//! Renderer-neutral data and deterministic generation for local spatial maps.
//!
//! This module intentionally contains no Bevy types so map generation and state
//! can be tested independently from presentation.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::rng::DeterministicRng;

pub const LOCAL_MAP_GENERATOR_VERSION: u32 = 2;
pub const DEFAULT_MAP_WIDTH: u16 = 24;
pub const DEFAULT_MAP_HEIGHT: u16 = 16;
const MIN_MAP_DIMENSION: u16 = 8;
const MAX_MAP_DIMENSION: u16 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    Floor,
    Ash,
    Rubble,
    Wall,
}

impl Terrain {
    pub fn is_walkable(self) -> bool {
        !matches!(self, Self::Wall)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointOfInterest {
    /// Stable content identifier, independent of display text.
    pub id: String,
    pub position: GridPosition,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub discovered: bool,
    #[serde(default)]
    pub interacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainChange {
    pub position: GridPosition,
    pub terrain: Terrain,
}

/// Static environmental props supported by the first spatial-rendering slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialPropKind {
    Pillar,
    Table,
}

impl SpatialPropKind {
    /// Footprint width measured in local-map cells; props occupy one grid row.
    pub fn footprint_width(self) -> u16 {
        match self {
            Self::Pillar => 1,
            Self::Table => 2,
        }
    }
}

/// Persistent prop anchor. A table's position is the left-most cell of its
/// two-cell footprint; a pillar occupies its single anchor cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialProp {
    pub id: String,
    pub kind: SpatialPropKind,
    pub position: GridPosition,
}

impl SpatialProp {
    pub fn occupies(&self, position: GridPosition) -> bool {
        position.y == self.position.y
            && position.x >= self.position.x
            && position.x < self.position.x.saturating_add(self.kind.footprint_width())
    }

    pub fn footprint_contains(&self, position: GridPosition, map_width: u16) -> bool {
        self.occupies(position) && position.x < map_width
    }
}

/// A physical passage at the edge of a local map. The destination uses the
/// persistent world location ID so it can later connect directly to the world
/// graph without relying on display names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapExit {
    /// Stable within this local map and independent of destination display text.
    pub id: String,
    pub position: GridPosition,
    pub target_location_id: u64,
}

/// Persistent local-map state. Once generated, the stored terrain is the source
/// of truth; loading a map must not regenerate it with a newer generator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMap {
    /// Stable campaign/content location ID, not its display name.
    pub location_id: String,
    pub generator_version: u32,
    pub seed: u64,
    pub width: u16,
    pub height: u16,
    /// Row-major terrain, with exactly width * height entries.
    pub terrain: Vec<Terrain>,
    pub entry: GridPosition,
    pub points_of_interest: Vec<PointOfInterest>,
    /// Connected world locations reachable through physical map-edge passages.
    /// Defaults empty so maps in older saves remain loadable.
    #[serde(default)]
    pub exits: Vec<MapExit>,
    #[serde(default)]
    pub discovered_tiles: Vec<bool>,
    #[serde(default)]
    pub terrain_changes: Vec<TerrainChange>,
    /// Persistent static props and their collision footprints. Missing in older saves.
    #[serde(default)]
    pub props: Vec<SpatialProp>,
}

impl LocalMap {
    pub fn contains(&self, position: GridPosition) -> bool {
        position.x < self.width && position.y < self.height
    }

    pub fn terrain_at(&self, position: GridPosition) -> Option<Terrain> {
        if !self.contains(position) {
            return None;
        }
        self.terrain
            .get(usize::from(position.y) * usize::from(self.width) + usize::from(position.x))
            .copied()
    }

    pub fn is_walkable(&self, position: GridPosition) -> bool {
        self.terrain_at(position)
            .map(Terrain::is_walkable)
            .unwrap_or(false)
            && !self.props.iter().any(|prop| prop.footprint_contains(position, self.width))
    }

    pub fn set_terrain(&mut self, position: GridPosition, terrain: Terrain) -> bool {
        if !self.contains(position) {
            return false;
        }
        let index = usize::from(position.y) * usize::from(self.width) + usize::from(position.x);
        let Some(cell) = self.terrain.get_mut(index) else {
            return false;
        };
        *cell = terrain;
        if let Some(change) = self
            .terrain_changes
            .iter_mut()
            .find(|change| change.position == position)
        {
            change.terrain = terrain;
        } else {
            self.terrain_changes
                .push(TerrainChange { position, terrain });
        }
        true
    }

    pub fn mark_tile_discovered(&mut self, position: GridPosition) -> bool {
        if !self.contains(position) {
            return false;
        }
        let expected_len = usize::from(self.width) * usize::from(self.height);
        if self.discovered_tiles.len() != expected_len {
            self.discovered_tiles.resize(expected_len, false);
        }
        let index = usize::from(position.y) * usize::from(self.width) + usize::from(position.x);
        self.discovered_tiles[index] = true;
        true
    }

    pub fn is_tile_discovered(&self, position: GridPosition) -> bool {
        if !self.contains(position) {
            return false;
        }
        let index = usize::from(position.y) * usize::from(self.width) + usize::from(position.x);
        self.discovered_tiles.get(index).copied().unwrap_or(false)
    }

    pub fn point_of_interest(&self, id: &str) -> Option<&PointOfInterest> {
        self.points_of_interest.iter().find(|poi| poi.id == id)
    }

    pub fn point_of_interest_mut(&mut self, id: &str) -> Option<&mut PointOfInterest> {
        self.points_of_interest.iter_mut().find(|poi| poi.id == id)
    }
}

/// Character-specific state for the currently active local exploration session.
/// This is deliberately separate from LocalMap, which belongs to the world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplorationSession {
    pub location_id: String,
    pub position: GridPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalMapConfig {
    pub width: u16,
    pub height: u16,
}

impl Default for LocalMapConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_MAP_WIDTH,
            height: DEFAULT_MAP_HEIGHT,
        }
    }
}

impl LocalMapConfig {
    fn normalized(self) -> Self {
        Self {
            width: self.width.clamp(MIN_MAP_DIMENSION, MAX_MAP_DIMENSION),
            height: self.height.clamp(MIN_MAP_DIMENSION, MAX_MAP_DIMENSION),
        }
    }
}

/// Generate a deterministic bounded map from a world seed and stable location
/// identity. A guaranteed corridor connects the entry and the initial point of
/// interest, even when random terrain would otherwise block the route.
pub fn generate_local_map(
    world_seed: u64,
    location_id: impl Into<String>,
    config: LocalMapConfig,
) -> LocalMap {
    let location_id = location_id.into();
    let config = config.normalized();
    let seed = local_map_seed(world_seed, &location_id);
    let mut rng = DeterministicRng::new(seed);
    let width = config.width;
    let height = config.height;
    let entry = GridPosition {
        x: 1,
        y: height / 2,
    };
    let poi_position = GridPosition {
        x: width - 2,
        y: 1 + rng.gen_range(0..usize::from(height - 2)) as u16,
    };
    let mut terrain = Vec::with_capacity(usize::from(width) * usize::from(height));

    for y in 0..height {
        for x in 0..width {
            let position = GridPosition { x, y };
            let tile = if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                Terrain::Wall
            } else {
                match rng.gen_range(0..12) {
                    0 | 1 => Terrain::Wall,
                    2 | 3 => Terrain::Rubble,
                    4 | 5 => Terrain::Ash,
                    _ => Terrain::Floor,
                }
            };
            let index = usize::from(y) * usize::from(width) + usize::from(x);
            debug_assert_eq!(terrain.len(), index);
            terrain.push(if position == entry || position == poi_position {
                Terrain::Floor
            } else {
                tile
            });
        }
    }

    // Carve an L-shaped corridor, making the first discovery reachable for all
    // generated dimensions and seeds.
    let corridor_y = entry.y;
    for x in entry.x..=poi_position.x {
        let index = usize::from(corridor_y) * usize::from(width) + usize::from(x);
        terrain[index] = Terrain::Floor;
    }
    let (from_y, to_y) = if corridor_y <= poi_position.y {
        (corridor_y, poi_position.y)
    } else {
        (poi_position.y, corridor_y)
    };
    for y in from_y..=to_y {
        let index = usize::from(y) * usize::from(width) + usize::from(poi_position.x);
        terrain[index] = Terrain::Floor;
    }

    let discovered_tiles = vec![false; terrain.len()];
    let mut map = LocalMap {
        location_id: location_id.clone(),
        generator_version: LOCAL_MAP_GENERATOR_VERSION,
        seed,
        width,
        height,
        terrain,
        entry,
        points_of_interest: vec![PointOfInterest {
            id: format!("{location_id}.poi.initial_discovery"),
            position: poi_position,
            title: "Unsettling Remains".to_string(),
            description: "Something here resists the passage of time.".to_string(),
            discovered: false,
            interacted: false,
        }],
        exits: Vec::new(),
        discovered_tiles,
        terrain_changes: Vec::new(),
        props: Vec::new(),
    };
    populate_generated_props(&mut map, &mut rng);
    map
}

/// Add a sparse deterministic set of props outside the guaranteed
/// entry-to-discovery route. Props are persisted and are not regenerated for
/// an already established local map.
fn populate_generated_props(map: &mut LocalMap, rng: &mut DeterministicRng) {
    let target_count =
        (usize::from(map.width) * usize::from(map.height) / 96).clamp(1, 5);

    for index in 0..target_count {
        let mut kind = if index % 2 == 0 {
            SpatialPropKind::Pillar
        } else {
            SpatialPropKind::Table
        };
        let mut candidates = prop_candidates(map, kind);
        if candidates.is_empty() && kind == SpatialPropKind::Table {
            kind = SpatialPropKind::Pillar;
            candidates = prop_candidates(map, kind);
        }
        if candidates.is_empty() {
            break;
        }

        let position = candidates[rng.gen_range(0..candidates.len())];
        map.props.push(SpatialProp {
            id: format!("{}.prop.{index}", map.location_id),
            kind,
            position,
        });
    }
}

fn prop_candidates(map: &LocalMap, kind: SpatialPropKind) -> Vec<GridPosition> {
    let poi = map.points_of_interest.first().map(|poi| poi.position);
    let mut candidates = Vec::new();

    for y in 1..map.height.saturating_sub(1) {
        for x in 1..map.width.saturating_sub(1) {
            let last_x = x.saturating_add(kind.footprint_width().saturating_sub(1));
            if last_x >= map.width.saturating_sub(1) {
                continue;
            }

            let footprint = (x..=last_x)
                .map(|cell_x| GridPosition { x: cell_x, y })
                .collect::<Vec<_>>();
            let conflicts_with_route = footprint
                .iter()
                .any(|cell| reserved_discovery_corridor(map.entry, poi, *cell));
            let overlaps_prop = footprint.iter().any(|cell| {
                map.props
                    .iter()
                    .any(|prop| prop.footprint_contains(*cell, map.width))
            });
            let terrain_walkable = footprint.iter().all(|cell| {
                map.terrain_at(*cell).is_some_and(Terrain::is_walkable)
            });

            if terrain_walkable && !conflicts_with_route && !overlaps_prop {
                candidates.push(position);
            }
        }
    }
    candidates
}

fn reserved_discovery_corridor(
    entry: GridPosition,
    poi: Option<GridPosition>,
    position: GridPosition,
) -> bool {
    let Some(poi) = poi else {
        return false;
    };
    let horizontal =
        position.y == entry.y && position.x >= entry.x && position.x <= poi.x;
    let (first_y, last_y) = if entry.y <= poi.y {
        (entry.y, poi.y)
    } else {
        (poi.y, entry.y)
    };
    let vertical = position.x == poi.x && position.y >= first_y && position.y <= last_y;
    horizontal || vertical
}

/// Place a physical map-edge passage for each connected world location.
///
/// Passages are assigned deterministically to east, west, north, and south
/// edges. Each gets a clear walkable corridor to the entry area. Exit targets
/// are world IDs, so destination danger state does not affect passage identity.
pub fn configure_map_exits(map: &mut LocalMap, target_location_ids: &[u64]) {
    let mut targets = Vec::with_capacity(target_location_ids.len());
    for target in target_location_ids {
        if !targets.contains(target) {
            targets.push(*target);
        }
    }

    map.exits.clear();
    for (index, target_location_id) in targets.iter().copied().enumerate() {
        let side = index % 4;
        let slot = index / 4;
        let side_count = targets.iter().skip(side).step_by(4).count();
        let (x, y) = match side {
            // East and west exits are spaced along the vertical edge.
            0 => (
                map.width - 1,
                spaced_edge_coordinate(slot, side_count, map.height),
            ),
            1 => (0, spaced_edge_coordinate(slot, side_count, map.height)),
            // North and south exits are spaced along the horizontal edge.
            2 => (spaced_edge_coordinate(slot, side_count, map.width), 0),
            _ => (
                spaced_edge_coordinate(slot, side_count, map.width),
                map.height - 1,
            ),
        };
        let position = GridPosition { x, y };
        carve_exit_corridor(map, position, side);
        map.exits.push(MapExit {
            id: format!("{}.exit.{}", map.location_id, target_location_id),
            position,
            target_location_id,
        });
    }
}

fn spaced_edge_coordinate(slot: usize, count: usize, dimension: u16) -> u16 {
    let usable = usize::from(dimension.saturating_sub(2));
    (1 + (slot + 1) * usable / (count + 1)) as u16
}

fn carve_exit_corridor(map: &mut LocalMap, exit: GridPosition, side: usize) {
    let mut position = exit;
    carve_generated_floor(map, position);
    match side {
        // Reach the entry column before joining the central entry corridor.
        0 | 1 => {
            while position.x != map.entry.x {
                position.x = if position.x > map.entry.x {
                    position.x - 1
                } else {
                    position.x + 1
                };
                carve_generated_floor(map, position);
            }
            while position.y != map.entry.y {
                position.y = if position.y > map.entry.y {
                    position.y - 1
                } else {
                    position.y + 1
                };
                carve_generated_floor(map, position);
            }
        }
        // North/south passages first move inward, then join the entry corridor.
        _ => {
            while position.y != map.entry.y {
                position.y = if position.y > map.entry.y {
                    position.y - 1
                } else {
                    position.y + 1
                };
                carve_generated_floor(map, position);
            }
            while position.x != map.entry.x {
                position.x = if position.x > map.entry.x {
                    position.x - 1
                } else {
                    position.x + 1
                };
                carve_generated_floor(map, position);
            }
        }
    }
}

fn carve_generated_floor(map: &mut LocalMap, position: GridPosition) {
    // Generated exit corridors are authoritative; remove any obstacle whose
    // footprint would overlap a required corridor cell.
    map.props
        .retain(|prop| !prop.footprint_contains(position, map.width));
    let index = usize::from(position.y) * usize::from(map.width) + usize::from(position.x);
    if let Some(terrain) = map.terrain.get_mut(index) {
        *terrain = Terrain::Floor;
    }
}

fn local_map_seed(world_seed: u64, location_id: &str) -> u64 {
    // FNV-1a over the stable location ID, mixed with the campaign seed. This
    // avoids dependence on process-randomized hashers or display names.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in location_id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash ^ world_seed.rotate_left(23)
}

/// Return whether a walkable path exists between two map positions.
pub fn has_walkable_path(map: &LocalMap, start: GridPosition, goal: GridPosition) -> bool {
    if !map.is_walkable(start) || !map.is_walkable(goal) {
        return false;
    }
    let mut visited = vec![false; usize::from(map.width) * usize::from(map.height)];
    let index_of = |position: GridPosition| {
        usize::from(position.y) * usize::from(map.width) + usize::from(position.x)
    };
    let mut queue = VecDeque::new();
    visited[index_of(start)] = true;
    queue.push_back(start);

    while let Some(position) = queue.pop_front() {
        if position == goal {
            return true;
        }
        let neighbors = [
            position
                .x
                .checked_sub(1)
                .map(|x| GridPosition { x, y: position.y }),
            position
                .x
                .checked_add(1)
                .filter(|x| *x < map.width)
                .map(|x| GridPosition { x, y: position.y }),
            position
                .y
                .checked_sub(1)
                .map(|y| GridPosition { x: position.x, y }),
            position
                .y
                .checked_add(1)
                .filter(|y| *y < map.height)
                .map(|y| GridPosition { x: position.x, y }),
        ];
        for neighbor in neighbors.into_iter().flatten() {
            let index = index_of(neighbor);
            if !visited[index] && map.is_walkable(neighbor) {
                visited[index] = true;
                queue.push_back(neighbor);
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_location_generate_identical_maps() {
        let a = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
        let b = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
        assert_eq!(a, b);
    }

    #[test]
    fn different_location_ids_generate_distinct_maps() {
        let a = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
        let b = generate_local_map(42, "location.ashen_gate", LocalMapConfig::default());
        assert_ne!(a.seed, b.seed);
        assert_ne!(a, b);
    }

    #[test]
    fn generated_props_are_deterministic_and_block_their_footprints() {
        let a = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
        let b = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
        assert_eq!(a.props, b.props);
        assert!(!a.props.is_empty());
        assert!(a.props.iter().any(|prop| prop.kind == SpatialPropKind::Pillar));
        assert!(a.props.iter().any(|prop| prop.kind == SpatialPropKind::Table));

        for prop in &a.props {
            for offset_x in 0..prop.kind.footprint_width() {
                let occupied = GridPosition {
                    x: prop.position.x + offset_x,
                    y: prop.position.y,
                };
                assert!(!a.is_walkable(occupied), "prop footprint must block {occupied:?}");
            }
        }
        assert!(has_walkable_path(&a, a.entry, a.points_of_interest[0].position));
    }

    #[test]
    fn props_round_trip_and_older_saves_default_to_no_props() {
        let map = generate_local_map(12, "location.old_shrine", LocalMapConfig::default());
        let json = serde_json::to_string(&map).expect("map with props should serialize");
        let restored: LocalMap =
            serde_json::from_str(&json).expect("map with props should deserialize");
        assert_eq!(restored.props, map.props);

        let mut old_value = serde_json::to_value(&map).expect("map should serialize");
        let object = old_value.as_object_mut().expect("map should be an object");
        object.remove("props");
        object.remove("exits");
        let restored_old: LocalMap =
            serde_json::from_value(old_value).expect("pre-prop map should remain loadable");
        assert!(restored_old.props.is_empty());
        assert!(restored_old.exits.is_empty());
    }

    #[test]
    fn generated_map_has_valid_bounds_and_reachable_point_of_interest() {
        for seed in 0..64 {
            let map = generate_local_map(seed, "location.old_shrine", LocalMapConfig::default());
            assert_eq!(
                map.terrain.len(),
                usize::from(map.width) * usize::from(map.height)
            );
            assert!(map.contains(map.entry));
            assert!(map.is_walkable(map.entry));
            let poi = &map.points_of_interest[0];
            assert!(map.contains(poi.position));
            assert!(map.is_walkable(poi.position));
            assert!(has_walkable_path(&map, map.entry, poi.position));
        }
    }

    #[test]
    fn dimensions_are_normalized_and_map_is_bounded() {
        let map = generate_local_map(
            1,
            "location.test",
            LocalMapConfig {
                width: 1,
                height: u16::MAX,
            },
        );
        assert_eq!(map.width, MIN_MAP_DIMENSION);
        assert_eq!(map.height, MAX_MAP_DIMENSION);
        assert_eq!(
            map.terrain.len(),
            usize::from(map.width) * usize::from(map.height)
        );
    }

    #[test]
    fn connected_world_locations_get_reachable_edge_passages() {
        let mut map = generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
        configure_map_exits(&mut map, &[11, 22, 33, 44, 22]);

        assert_eq!(map.exits.len(), 4);
        assert_eq!(
            map.exits
                .iter()
                .map(|exit| exit.target_location_id)
                .collect::<Vec<_>>(),
            vec![11, 22, 33, 44]
        );
        for exit in &map.exits {
            assert!(
                exit.position.x == 0
                    || exit.position.x == map.width - 1
                    || exit.position.y == 0
                    || exit.position.y == map.height - 1,
                "passage must be on a map edge"
            );
            assert!(map.is_walkable(exit.position));
            assert!(has_walkable_path(&map, map.entry, exit.position));
        }

        let repeated = {
            let mut repeated =
                generate_local_map(42, "location.old_shrine", LocalMapConfig::default());
            configure_map_exits(&mut repeated, &[11, 22, 33, 44, 22]);
            repeated
        };
        assert_eq!(map.exits, repeated.exits);
        assert_eq!(map.terrain, repeated.terrain);
    }

    #[test]
    fn generated_passages_remain_reachable_and_persistent_across_seeds() {
        for seed in 0..32 {
            for target_count in [1, 2, 3, 4, 6] {
                let targets = (0..target_count)
                    .map(|index| 100 + index as u64)
                    .collect::<Vec<_>>();
                let mut map = generate_local_map(
                    seed,
                    "location.spatial_exit_test",
                    LocalMapConfig::default(),
                );
                configure_map_exits(&mut map, &targets);

                assert_eq!(map.exits.len(), target_count);
                assert!(has_walkable_path(
                    &map,
                    map.entry,
                    map.points_of_interest[0].position
                ));
                for exit in &map.exits {
                    assert!(has_walkable_path(&map, map.entry, exit.position));
                }

                let json = serde_json::to_string(&map).expect("map with passages should serialize");
                let restored: LocalMap =
                    serde_json::from_str(&json).expect("map with passages should deserialize");
                assert_eq!(restored.exits, map.exits);
                assert_eq!(restored.terrain, map.terrain);
            }
        }
    }

    #[test]
    fn map_and_session_round_trip_through_serde() {
        let mut map = generate_local_map(123, "location.old_shrine", LocalMapConfig::default());
        let poi_id = map.points_of_interest[0].id.clone();
        map.point_of_interest_mut(&poi_id).unwrap().discovered = true;
        map.mark_tile_discovered(map.entry);
        map.set_terrain(map.entry, Terrain::Rubble);
        let json = serde_json::to_string(&map).expect("map should serialize");
        let restored: LocalMap = serde_json::from_str(&json).expect("map should deserialize");
        assert_eq!(restored, map);

        let session = ExplorationSession {
            location_id: map.location_id.clone(),
            position: map.entry,
        };
        let json = serde_json::to_string(&session).expect("session should serialize");
        let restored: ExplorationSession =
            serde_json::from_str(&json).expect("session should deserialize");
        assert_eq!(restored, session);
    }
}
