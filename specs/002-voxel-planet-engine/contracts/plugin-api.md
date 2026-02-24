# Plugin API Contract: VoxelWorldPlugin

**Feature**: `002-voxel-planet-engine` | **Date**: 2026-02-20

This document defines the public surface of `VoxelWorldPlugin` — what other plugins
may safely read, what they must not write, and what hooks are reserved for Feature 003.

---

## Public Resources (read-only for other plugins)

### `PlanetConfig` (Resource)

```rust
// crate::plugins::voxel::config::PlanetConfig
pub struct PlanetConfig {
    pub radius_km:        f64,
    pub seed_override:    Option<u64>,
    pub memory_budget_mb: u32,
    pub noise_layers:     Vec<NoiseLayer>,
    pub erosion:          ErosionConfig,
    pub biome:            BiomeConfig,
}
```

**Contract**: Present from `PostStartup` onwards. Inserted by `load_planet_config`
in `Startup`. Other plugins MAY read this resource. No plugin other than
`VoxelWorldPlugin` may mutate it after insertion.

**Env override**: `NEON_PLANET_CONFIG=<path>` at process launch.

---

### `VoxelStats` (Resource)

```rust
pub struct VoxelStats {
    pub loaded_chunks:   u32,
    pub queued_chunks:   u32,
    pub in_flight_tasks: u8,    // max 4
    pub memory_used_mb:  f32,
}
```

**Contract**: Updated every `Update` frame by `chunk_streamer`. Read-only for
all other plugins. `TraversalPlugin` may read `loaded_chunks` to throttle
physics spawns.

---

### `ChunkPool` (Resource)

```rust
pub struct ChunkPool {
    pub loaded:     HashMap<IVec3, Entity>,
    pub bytes_used: usize,
}
```

**Contract**: Authoritative mapping of loaded chunk coordinates to entities.
Feature 003 MAY call `chunk_pool.loaded.get(&coord)` to check if a chunk is
resident before spawning physics actors on it. Must not be mutated externally.

---

## Public Components (queryable by other plugins)

### `VoxelChunk` (Component)

```rust
pub struct VoxelChunk {
    pub chunk_coord: IVec3,  // chunk-space address
    pub lod:         u8,     // 0 = highest detail
}
```

**Contract**: Present on all resident chunk entities from the frame they are spawned.
Feature 003 may query `With<VoxelChunk>` to iterate terrain entities.

### `ChunkCollider` (Component, marker)

**Contract**: Present on a chunk entity once its Avian3d `Collider` is active.
Feature 003 character controller SHOULD require `With<ChunkCollider>` before
allowing the player to stand on a chunk, preventing tunnelling during load.

---

## Invariants Other Plugins Must Respect

1. **Do not write `Transform`** on any entity that carries `GlobalPosition`.
   `FloatingOriginPlugin` owns `Transform` on those entities.

2. **Do not insert `RigidBody`** on chunk entities. `ColliderSyncer` inserts
   `RigidBody::Static`; a second insert would conflict.

3. **Do not call `despawn`** on chunk entities directly. Use a future
   `ChunkEvictRequest` event (not in this sprint) or rely on `ChunkPool` eviction.

4. **Do not modify `PlanetConfig`** at runtime. It is read by async worker tasks;
   mutation after startup is a data race.

---

## System Ordering Guarantees

```
Startup:
  load_planet_config  →  spawn_planet_on_startup

Update (within VoxelWorldPlugin):
  scaled_space_switcher
  chunk_streamer
  chunk_task_poller    (after chunk_streamer)
  collider_syncer      (after chunk_task_poller)

PostUpdate (external, unchanged):
  LodPlugin::update_lod_levels  →  FloatingOriginPlugin::sync_transforms
```

Feature 003 systems that depend on chunk collision being ready should run in
`Update` after `collider_syncer` or in `PostUpdate`.

---

## Reserved Hooks for Feature 003

| Hook | Purpose | Implementation |
|------|---------|----------------|
| `ChunkReadyEvent { entity, coord, lod }` | Notify traversal when a new LOD-0 chunk is ready | Add to `chunk_task_poller` in 003 sprint |
| `VoxelLayerRegistry` (Resource) | Mod-contributed additional noise layers | Register in `VoxelWorldPlugin::build` in 003 or mod sprint |
| `BiomeQuery::at(GlobalPosition) -> BiomeBand` | Query terrain type at a world position (for vehicle spawn, footstep sounds) | Inline noise_stack call, add helper in 003 |

---

## Breaking Change Policy

Any change to the public types in this document that removes or renames a field
requires a +1 on the Constitution amendment process (§ Governance). Additive
changes (new optional fields, new resources) do not require an amendment.
