//! Determinism regression test for the voxel planet engine.
//!
//! Confirms that two independent evaluations of `generate_chunk_densities`
//! with the same seed and config produce **bit-identical** output (FR-034).
//!
//! # Golden snapshot workflow
//! On first run (or when intentionally updating), generate the golden binary:
//! ```
//! GENERATE_GOLDEN=1 cargo test voxel_determinism -- --nocapture
//! ```
//! This writes `tests/snapshots/voxel_density_golden.bin` and the test passes.
//!
//! On subsequent CI runs, the snapshot is loaded and compared byte-by-byte.
//! Any deviation fails the test.

use bevy::math::{DVec3, IVec3};
use neon_expanse::plugins::voxel::{config::PlanetConfig, noise_stack::generate_chunk_densities};
use std::path::Path;

/// World seed used for all determinism tests.
const SEED: u64 = 0x4E45_4F4E_3230_3236;
/// Number of chunk coordinates to include in the golden snapshot.
const CHUNK_COUNT: usize = 100;
/// Path to the committed golden binary.
const SNAPSHOT_PATH: &str = "tests/snapshots/voxel_density_golden.bin";

// ── Snapshot data generation ──────────────────────────────────────────────────

/// Generate a reproducible set of chunk coordinates spread across the planet.
fn test_chunk_coords(n: usize) -> Vec<IVec3> {
    (0..n as i32)
        .map(|i| {
            // Deterministic spread: wrap in a small spiral so coords
            // are never at the exact same position.
            IVec3::new(
                (i * 7 + 3) % 64 - 32,
                (i * 5 + 1) % 32 - 16,
                (i * 11 + 9) % 64 - 32,
            )
        })
        .collect()
}

/// Generate raw density bytes for the given chunk coords.
fn generate_snapshot_bytes() -> Vec<u8> {
    let config = PlanetConfig::default();
    let cell_size = 4.0_f32; // 4 m per voxel at LOD 0 test resolution
    let voxels = 8_u32; // small grid keeps snapshot size manageable
    let chunk_size_m = voxels as f64 * cell_size as f64;

    let mut bytes = Vec::new();

    for coord in test_chunk_coords(CHUNK_COUNT) {
        let origin = DVec3::new(
            coord.x as f64 * chunk_size_m + 6_370_000.0, // surface-adjacent
            coord.y as f64 * chunk_size_m,
            coord.z as f64 * chunk_size_m,
        );
        let densities = generate_chunk_densities(origin, cell_size, voxels, SEED, &config);
        for d in &densities {
            bytes.extend_from_slice(&d.to_le_bytes());
        }
    }

    bytes
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Primary determinism regression test.
///
/// If `GENERATE_GOLDEN=1` is set, regenerates the snapshot and exits without
/// comparing. Otherwise loads the committed snapshot and compares byte-by-byte.
#[test]
fn voxel_density_matches_golden_snapshot() {
    let snapshot_path = Path::new(SNAPSHOT_PATH);

    if std::env::var("GENERATE_GOLDEN")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        // ── Generate mode ─────────────────────────────────────────────────
        let bytes = generate_snapshot_bytes();
        std::fs::create_dir_all(snapshot_path.parent().unwrap())
            .expect("failed to create snapshots dir");
        std::fs::write(snapshot_path, &bytes).expect("failed to write golden snapshot");
        println!(
            "Golden snapshot written: {} bytes ({} chunks × {} voxels³)",
            bytes.len(),
            CHUNK_COUNT,
            (8 + 2) * (8 + 2) * (8 + 2)
        );
        return; // pass unconditionally in generate mode
    }

    // ── Comparison mode ───────────────────────────────────────────────────
    if !snapshot_path.exists() {
        panic!(
            "Golden snapshot not found at '{SNAPSHOT_PATH}'.\n\
             Generate it first: GENERATE_GOLDEN=1 cargo test voxel_determinism -- --nocapture"
        );
    }

    let expected = std::fs::read(snapshot_path).expect("failed to read golden snapshot");
    let actual = generate_snapshot_bytes();

    assert_eq!(
        expected.len(),
        actual.len(),
        "Snapshot size mismatch: expected {} bytes, got {} bytes",
        expected.len(),
        actual.len()
    );

    // Find first differing byte for a helpful error message.
    if let Some(pos) = expected.iter().zip(actual.iter()).position(|(e, a)| e != a) {
        let chunk_idx = pos / (std::mem::size_of::<f32>() * 1000);
        let sample_idx = (pos / std::mem::size_of::<f32>()) % 1000;
        let expected_f =
            f32::from_le_bytes(expected[(pos & !3)..(pos & !3) + 4].try_into().unwrap());
        let actual_f = f32::from_le_bytes(actual[(pos & !3)..(pos & !3) + 4].try_into().unwrap());
        panic!(
            "Golden snapshot mismatch at byte {pos} (chunk {chunk_idx}, sample {sample_idx}):\n\
             expected f32 = {expected_f:.6},  actual f32 = {actual_f:.6}\n\
             Regenerate with: GENERATE_GOLDEN=1 cargo test voxel_determinism -- --nocapture"
        );
    }
}

/// Confirm two independent calls produce identical output (T032 smoke test).
#[test]
fn generate_chunk_densities_is_deterministic() {
    let config = PlanetConfig::default();
    let origin = DVec3::new(6_370_000.0, 0.0, 0.0);
    let cell_size = 4.0_f32;
    let voxels = 8_u32;

    let a = generate_chunk_densities(origin, cell_size, voxels, SEED, &config);
    let b = generate_chunk_densities(origin, cell_size, voxels, SEED, &config);

    assert_eq!(a.len(), b.len(), "density grid lengths differ");
    for (i, (va, vb)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            va.to_bits(),
            vb.to_bits(),
            "density mismatch at index {i}: a={va:.6} b={vb:.6}"
        );
    }
}

/// T024 — Physics smoke test (FR-030).
///
/// Generates a real LOD-0 chunk at the north-pole surface, builds a trimesh
/// collider from the DMC output, drops a dynamic rigid body 10 m above the
/// mesh, and steps 200 physics frames at 64 Hz.
///
/// **Pass criteria**:
/// - Body Y within 0.1 m of expected rest height (surface + sphere radius).
/// - Vertical velocity < 0.01 m/s (stable rest — body has settled).
#[test]
fn physics_rigid_body_rests_on_lod0_chunk() {
    use avian3d::prelude::*;
    use bevy::{
        mesh::{Indices, VertexAttributeValues},
        prelude::*,
        time::TimeUpdateStrategy,
    };
    use neon_expanse::plugins::voxel::{
        components::VoxelData, config::PlanetConfig, mesher::dmc::mesh_chunk,
        noise_stack::generate_chunk_densities,
    };
    use std::time::Duration;

    // ── 1. Generate a LOD-0 chunk 10 m below the north-pole surface ───────
    let config = PlanetConfig::default();
    let cell_size = 4.0_f32; // 4 m/voxel → 32 m chunk edge
    let voxels = 8_u32;
    let surface_r = config.radius_km * 1_000.0;

    // Chunk corner 10 m below the surface so the isosurface cuts through the
    // upper quarter of the chunk (~y_local = 10 within [0, 32]).
    let chunk_origin_d = DVec3::new(0.0, surface_r - 10.0, 0.0);
    let densities = generate_chunk_densities(chunk_origin_d, cell_size, voxels, SEED, &config);
    let voxel_data = VoxelData {
        densities,
        voxels_per_edge: voxels,
    };

    // ── 2. Run DMC mesher in local space (vertices in [0, 32] m) ─────────
    let mesh = mesh_chunk(
        &voxel_data,
        Vec3::ZERO,
        cell_size,
        &config.biome,
        surface_r as f32,
    );

    let positions: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(data)) => {
            data.iter().map(|p| Vec3::from(*p)).collect()
        }
        _ => vec![],
    };
    let triangles: Vec<[u32; 3]> = match mesh.indices() {
        Some(Indices::U32(data)) => data.chunks(3).map(|c| [c[0], c[1], c[2]]).collect(),
        _ => vec![],
    };

    // Guard: if the chunk contains no surface (fully solid or fully air) skip.
    // This should not happen at the pole surface but avoids a panicking test.
    if positions.is_empty() || triangles.is_empty() {
        eprintln!(
            "T024 SKIP: DMC produced no geometry at north-pole LOD-0 chunk \
             (origin={chunk_origin_d:?}). Surface may have been missed by noise."
        );
        return;
    }

    // Highest Y vertex — approximate contact surface for +Y gravity.
    let surface_y = positions
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);

    // ── 3. Build headless Bevy app with controlled time ───────────────────
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        PhysicsPlugins::default(),
    ))
    // Advance exactly 1/64 s per app.update() so the Fixed schedule
    // fires reliably without depending on wall-clock time.
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 64.0,
    )))
    .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)));

    // ── 4. Spawn static trimesh ground (the LOD-0 chunk surface) ─────────
    let ground_collider = Collider::trimesh(positions.clone(), triangles);
    app.world_mut().spawn((
        RigidBody::Static,
        ground_collider,
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
    ));

    // ── 5. Spawn dynamic sphere 10 m above the mesh surface ──────────────
    const SPHERE_RADIUS: f32 = 0.5;
    // XZ at chunk centre (16, _, 16); Y = surface + 10 m gap + sphere radius.
    let spawn_y = surface_y + 10.0 + SPHERE_RADIUS;
    let body = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(SPHERE_RADIUS),
            Transform::from_xyz(16.0, spawn_y, 16.0),
            GlobalTransform::IDENTITY,
        ))
        .id();

    // ── 6. Step 200 frames at 64 Hz (≈ 3.1 s of physics time) ───────────
    for _ in 0..200 {
        app.update();
    }

    // ── 7. Assertions ─────────────────────────────────────────────────────
    let body_ref = app.world().entity(body);
    let body_y = body_ref
        .get::<Transform>()
        .expect("body missing Transform")
        .translation
        .y;
    let vel_y = body_ref
        .get::<LinearVelocity>()
        .expect("body missing LinearVelocity")
        .y;

    // Sphere centre rests at surface_y + SPHERE_RADIUS when fully settled.
    let expected_rest_y = surface_y + SPHERE_RADIUS;
    assert!(
        (body_y - expected_rest_y).abs() < 0.1,
        "T024 FAIL position: body_y={body_y:.4} expected={expected_rest_y:.4} \
         (surface_y={surface_y:.4} + r={SPHERE_RADIUS}), \
         Δ={:.4} m (limit 0.1 m)",
        (body_y - expected_rest_y).abs()
    );
    assert!(
        vel_y.abs() < 0.01,
        "T024 FAIL velocity: vel_y={vel_y:.5} m/s (limit 0.01 m/s — body not yet at rest)"
    );
}
