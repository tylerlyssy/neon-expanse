//! Systems for the floating-origin coordinate system.

use avian3d::prelude::*;
use bevy::{math::DVec3, prelude::*};

use super::{
    components::{FloatingOrigin, GlobalPosition, TestOriginMarker},
    resources::OriginFrame,
};
use crate::plugins::lod::components::LodLevel;

/// The world-space position of the test sphere (and camera anchor).
///
/// ~6,000 km from the planet centre on the X axis.
/// Chosen so the camera is placed at `TEST_ENTITY_POS + CAMERA_OFFSET`,
/// keeping the sphere visible at a comfortable distance.
pub const TEST_ENTITY_POS: DVec3 = DVec3::new(6_000_000.0, 0.0, 0.0);

/// Camera offset from the test entity: 100 metres on the Z axis.
///
/// This ensures the camera's local `Transform` stays near `Vec3::ZERO`
/// while the global coordinate is ~6,000,000 m — proving DVec3→f32 sync.
pub const CAMERA_OFFSET: DVec3 = DVec3::new(0.0, 0.0, 100.0);

/// Startup system: spawns the camera, test sphere, and a point light
/// at planetary distance from the world origin.
///
/// Called once by [`FloatingOriginPlugin`] in the `Startup` schedule.
pub fn spawn_test_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera_pos = TEST_ENTITY_POS + CAMERA_OFFSET;

    // ── Camera ────────────────────────────────────────────────────────────────
    // Spawned with FloatingOrigin so its GlobalPosition defines the
    // current origin frame. Its Transform.translation will be Vec3::ZERO
    // after the first sync frame.
    // AmbientLight is a Component in Bevy 0.18 — attach to the camera entity.
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::ZERO).looking_at(-Vec3::Z * 100.0, Vec3::Y),
        GlobalPosition(camera_pos),
        FloatingOrigin,
        AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            affects_lightmapped_meshes: true,
        },
    ));

    // ── Point light near the sphere ───────────────────────────────────────────
    let light_pos = TEST_ENTITY_POS + DVec3::new(-150.0, 100.0, 50.0);
    commands.spawn((
        PointLight {
            intensity: 150_000.0,
            range: 500.0,
            shadows_enabled: false,
            ..default()
        },
        GlobalPosition(light_pos),
        Transform::default(),
    ));

    // ── Test origin marker sphere ─────────────────────────────────────────────
    // Radius 50 m — large enough to be clearly visible at 100 m range.
    // Emissive cyan so it's visible even without perfect lighting.
    let sphere_mesh = meshes.add(Sphere::new(50.0));
    let sphere_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.8, 0.9),
        emissive: LinearRgba::new(0.0, 0.5, 0.6, 1.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(sphere_mat),
        Transform::default(), // will be overwritten by sync_transforms
        GlobalPosition(TEST_ENTITY_POS),
        RigidBody::Static, // avian3d: immune to gravity; zero sim cost
        LodLevel(3),
        TestOriginMarker,
    ));

    info!("Neon Expanse core initialized");
    info!(
        "Test entity spawned at global position {} m from world origin",
        TEST_ENTITY_POS.length() as u64
    );
}

/// `PreUpdate` system: copies the [`FloatingOrigin`] entity's
/// [`GlobalPosition`] into the [`OriginFrame`] resource.
///
/// Must run every frame before any system reads [`OriginFrame`].
pub fn update_origin_frame(
    origin_query: Query<&GlobalPosition, With<FloatingOrigin>>,
    mut origin_frame: ResMut<OriginFrame>,
) {
    // There is always exactly one FloatingOrigin entity (the camera).
    // If for any reason it is missing, keep the previous frame's value.
    if let Ok(pos) = origin_query.single() {
        debug_assert!(
            pos.0.is_finite(),
            "FloatingOrigin GlobalPosition contains NaN or infinity"
        );
        origin_frame.position = pos.0;
    }
}

/// `PostUpdate` system: recomputes `Transform.translation` for every
/// entity with a [`GlobalPosition`] by subtracting the current
/// [`OriginFrame`].
///
/// Must run `.before(TransformSystem::TransformPropagate)` so that
/// `GlobalTransform` is computed from the freshly-synced `Transform`.
pub fn sync_transforms(
    origin_frame: Res<OriginFrame>,
    mut query: Query<(&GlobalPosition, &mut Transform)>,
) {
    let origin = origin_frame.position;
    for (gpos, mut transform) in query.iter_mut() {
        debug_assert!(
            gpos.0.is_finite(),
            "GlobalPosition contains NaN or infinity on entity"
        );
        // Safe narrowing: offset is at most a few hundred metres —
        // well within f32 precision range (~16M metres before precision loss).
        transform.translation = (gpos.0 - origin).as_vec3();
    }
}
