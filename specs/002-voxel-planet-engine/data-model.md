# Data Model: Procedural Voxel Planet Engine

**Feature**: `002-voxel-planet-engine` | **Date**: 2026-02-20

---

## Entity Map

```
PlanetCentre entity
│  GlobalPosition(DVec3::ZERO)
│  PlanetCentre (marker)
│
├── ScaledSpaceMarker entity (1 per planet)
│   GlobalPosition(DVec3::ZERO)
│   ScaledSpaceMarker
│   Mesh3d(Handle<Mesh>)          ← UV sphere, radius = planet_radius_m
│   MeshMaterial3d(Handle<Mat>)   ← StandardMaterial, biome-average colour
│   Visibility
│   Transform                     ← managed by FloatingOriginPlugin
│
└── VoxelChunk entities (N, streaming lifecycle)
    VoxelChunk { chunk_coord: IVec3, lod: u8 }
    VoxelData  { densities: Vec<f32>, voxels_per_edge: u32 }
    ChunkMesh  { mesh_handle: Handle<Mesh>, vertex_count: u32 }
    ChunkCollider (marker, present when collider inserted)
    Collider                     ← avian3d; trimesh at LOD 0, heightfield at LOD 1-2
    RigidBody::Static            ← avian3d
    GlobalPosition(DVec3)        ← chunk centre in world space
    LodLevel(u8)                 ← managed by LodPlugin
    Transform                    ← managed by FloatingOriginPlugin
    Visibility

ChunkGenTask entities (transient, 0–4 at any time)
    ChunkGenTask(Task<ChunkGenOutput>)  ← despawned when task completes
```

---

## Resource Map

| Resource | Owner | Purpose |
|----------|-------|---------|
| `PlanetConfig` | VoxelWorldPlugin | All planet parameters from RON |
| `ChunkPool` | VoxelWorldPlugin | `HashMap<IVec3, Entity>` + byte budget tracker |
| `StreamingQueue` | VoxelWorldPlugin | `BinaryHeap<StreamJob>` + in-flight dedup set |
| `ScaledSpaceState` | VoxelWorldPlugin | Whether sphere is currently visible |
| `VoxelStats` | VoxelWorldPlugin | Live diagnostics (chunk count, MB, tasks) |
| `WorldSeed` | CorePlugin (pre-existing) | `u64` seed consumed by noise stack |
| `OriginFrame` | FloatingOriginPlugin (pre-existing) | `DVec3` viewer position |
| `LodSettings` | LodPlugin (pre-existing) | `[f32; 3]` distance thresholds |

---

## Component Ownership Rules

| Component | Who writes | Who reads |
|-----------|-----------|-----------|
| `GlobalPosition` | Spawning system (planet_init/streaming) | FloatingOriginPlugin → Transform |
| `LodLevel` | LodPlugin (update_lod_levels, PostUpdate) | ColliderSyncer, ScaledSpaceSwitcher |
| `Transform` | FloatingOriginPlugin (sync_transforms, PostUpdate) | Bevy render |
| `VoxelData` | chunk_task_poller (from async output) | ColliderSyncer, DMC mesher |
| `Collider` | ColliderSyncer | avian3d physics |
| `Visibility` | ScaledSpaceSwitcher (sphere), chunk_task_poller (chunks) | Bevy render |

---

## Data Flows

```
Startup
  load_planet_config ──► PlanetConfig (Resource)
  spawn_planet_on_startup ──► ScaledSpaceMarker entity + PlanetCentre entity

Per-Update-frame
  OriginFrame (FloatingOriginPlugin, PreUpdate)
        │
        ▼
  chunk_streamer
        │ enqueues StreamJob items into StreamingQueue
        │ dispatches AsyncComputeTaskPool tasks (≤ 4)
        │ evicts over-budget chunks from ChunkPool
        ▼
  chunk_task_poller
        │ polls Task<ChunkGenOutput>
        │ spawns VoxelChunk entity with VoxelData + ChunkMesh + Mesh3d
        │ registers entity in ChunkPool
        ▼
  collider_syncer
        │ queries (VoxelChunk, ChunkMesh, VoxelData) WITHOUT ChunkCollider
        │ inserts Collider + RigidBody::Static + ChunkCollider marker
        ▼
  scaled_space_switcher
        │ reads OriginFrame.position distance vs PlanetConfig.radius_km × 10
        └ sets Visibility on ScaledSpaceMarker entity

  LodPlugin (PostUpdate)
        └ updates LodLevel on all entities with GlobalPosition
```

---

## State Transitions: Chunk Lifecycle

```
[Not Exists]
     │  StreamingQueue enqueues coord
     ▼
[Queued]  (StreamJob in BinaryHeap)
     │  task_count < 4 → dispatch AsyncComputeTaskPool task
     ▼
[In Flight]  (ChunkGenTask entity; coord in StreamingQueue::in_flight)
     │  task completes → poll_once returns ChunkGenOutput
     ▼
[Loaded]  (VoxelChunk entity with VoxelData + ChunkMesh + Collider)
     │  coord in ChunkPool::loaded
     │  budget exceeded OR viewer moves out of streaming radius
     ▼
[Despawned]  (entity despawn_recursive; coord removed from ChunkPool)
     └─► [Not Exists]
```

---

## Config Schema

### `PlanetConfig` (deserialised from RON)

| Field | Type | Unit | Default |
|-------|------|------|---------|
| `radius_km` | `f64` | km | 6371.0 |
| `seed_override` | `Option<u64>` | — | None |
| `memory_budget_mb` | `u32` | MB | 512 |
| `noise_layers` | `Vec<NoiseLayer>` | — | 1 FBM layer |
| `erosion` | `ErosionConfig` | — | 1 pass, 0.3 rate |
| `biome` | `BiomeConfig` | — | 2-band green→white |

### `NoiseLayer`

| Field | Type | Typical Range |
|-------|------|--------------|
| `kind` | `NoiseKind` | Fbm / Ridged / Billow |
| `frequency` | `f32` | 1e-7 – 1e-4 |
| `amplitude` | `f32` | 100 – 10,000 m |
| `octaves` | `u32` | 3–8 |
| `persistence` | `f32` | 0.3–0.7 |
| `lacunarity` | `f32` | 1.8–2.5 |

### `VoxelData` grid dimensions by LOD

| LOD | Chunk edge (m) | Voxels/edge | Grid size | Density bytes |
|-----|---------------|-------------|-----------|--------------|
| 0 | 128 | 32 | 33³ = 35,937 | 144 KB |
| 1 | 256 | 16 | 17³ = 4,913 | ~20 KB |
| 2 | 512 | 8 | 9³ = 729 | ~3 KB |
| 3 | 1024 | 8 | 9³ = 729 | ~3 KB |
