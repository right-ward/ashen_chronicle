//! Renderer-neutral data and deterministic generation for local spatial maps.
//!
//! This module intentionally contains no Bevy types so map generation and state
//! can be tested independently from presentation.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::rng::DeterministicRng;

pub const LOCAL_MAP_GENERATOR_VERSION: u32 = 1;
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
    #[serde(default)]
    pub discovered_tiles: Vec<bool>,
    #[serde(default)]
    pub terrain_changes: Vec<TerrainChange>,
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
            self.terrain_changes.push(TerrainChange { position, terrain });
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
    let entry = GridPosition { x: 1, y: height / 2 };
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
    LocalMap {
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
        discovered_tiles,
        terrain_changes: Vec::new(),
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
            position.x.checked_sub(1).map(|x| GridPosition { x, y: position.y }),
            position.x.checked_add(1).filter(|x| *x < map.width).map(|x| GridPosition { x, y: position.y }),
            position.y.checked_sub(1).map(|y| GridPosition { x: position.x, y }),
            position.y.checked_add(1).filter(|y| *y < map.height).map(|y| GridPosition { x: position.x, y }),
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
    fn generated_map_has_valid_bounds_and_reachable_point_of_interest() {
        for seed in 0..64 {
            let map = generate_local_map(seed, "location.old_shrine", LocalMapConfig::default());
            assert_eq!(map.terrain.len(), usize::from(map.width) * usize::from(map.height));
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
            LocalMapConfig { width: 1, height: u16::MAX },
        );
        assert_eq!(map.width, MIN_MAP_DIMENSION);
        assert_eq!(map.height, MAX_MAP_DIMENSION);
        assert_eq!(map.terrain.len(), usize::from(map.width) * usize::from(map.height));
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
