use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::room::Room;
use crate::tile::Tile;

/// The loading program: tracks rooms and known tile IDs.
#[derive(Debug)]
pub struct Loader {
    pub rooms: HashMap<String, Room>,
    pub known: HashSet<String>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            known: HashSet::new(),
        }
    }

    /// Load a room from disk, recording tile IDs.
    pub fn load_room(&mut self, path: &Path) -> Result<&Room, String> {
        let room = Room::load(path)?;
        for tile in &room.tiles {
            self.known.insert(tile.id.clone());
        }
        let name = room.name.clone();
        self.rooms.insert(name.clone(), room);
        Ok(self.rooms.get(&name).unwrap())
    }

    /// Returns only tiles that are NEW since the last load (not in `known` before this call).
    pub fn update(&mut self) -> Vec<&Tile> {
        let mut new_tiles = Vec::new();
        let mut new_ids = Vec::new();

        for room in self.rooms.values() {
            for tile in &room.tiles {
                if !self.known.contains(&tile.id) {
                    new_tiles.push(tile);
                    new_ids.push(tile.id.clone());
                }
            }
        }

        for id in new_ids {
            self.known.insert(id);
        }

        new_tiles
    }

    /// Full knowledge graph across all rooms.
    pub fn knowledge_base(&self) -> crate::room::TileGraph {
        let mut graph = crate::room::TileGraph::new();
        for room in self.rooms.values() {
            for (id, deps) in &room.graph.edges {
                graph.edges.insert(id.clone(), deps.clone());
            }
        }
        graph
    }

    /// Conservation ratio of the knowledge base: fraction of possible edges that exist.
    /// CR = actual_dependency_edges / (n * (n-1)) where n = total tiles.
    /// High CR = highly interconnected knowledge.
    pub fn conservation(&self) -> f64 {
        let mut total_tiles = 0usize;
        let mut total_deps = 0usize;

        for room in self.rooms.values() {
            total_tiles += room.tiles.len();
            for tile in &room.tiles {
                total_deps += tile.dependencies.len();
            }
        }

        if total_tiles <= 1 {
            return 1.0;
        }

        let possible = total_tiles * (total_tiles - 1);
        total_deps as f64 / possible as f64
    }

    /// Missing dependencies: tile IDs referenced but not present in any room.
    pub fn gaps(&self) -> Vec<String> {
        let all_ids: HashSet<&str> = self.rooms.values()
            .flat_map(|r| r.tiles.iter())
            .map(|t| t.id.as_str())
            .collect();

        let mut missing: HashSet<String> = HashSet::new();
        for room in self.rooms.values() {
            for tile in &room.tiles {
                for dep in &tile.dependencies {
                    if !all_ids.contains(dep.as_str()) {
                        missing.insert(dep.clone());
                    }
                }
            }
        }

        missing.into_iter().collect()
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}
