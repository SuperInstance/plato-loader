use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::tile::{Tile, TileStatus};

/// Adjacency list representation of the tile dependency graph.
#[derive(Debug, Clone, Default)]
pub struct TileGraph {
    /// tile_id -> set of tile IDs it depends on
    pub edges: HashMap<String, HashSet<String>>,
}

impl TileGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_tile(&mut self, id: &str, deps: &[String]) {
        self.edges.insert(id.to_string(), deps.iter().cloned().collect());
    }

    /// Compute the graph Laplacian as a flattened matrix (for spectral analysis).
    /// Returns (n, flattened_laplacian) where n is the number of nodes.
    pub fn laplacian(&self) -> (usize, Vec<f64>) {
        let nodes: Vec<&str> = self.edges.keys().map(|s| s.as_str()).collect();
        let n = nodes.len();
        let mut l = vec![0.0f64; n * n];

        for (i, node) in nodes.iter().enumerate() {
            let deps = self.edges.get(*node).cloned().unwrap_or_default();
            let degree = deps.len() as f64;
            l[i * n + i] = degree;
            for dep in &deps {
                if let Some(j) = nodes.iter().position(|&x| x == dep) {
                    l[i * n + j] -= 1.0;
                    l[j * n + i] -= 1.0;
                    l[j * n + j] += 1.0;
                }
            }
        }

        (n, l)
    }
}

/// A room: a collection of tiles forming a knowledge domain.
#[derive(Debug, Clone)]
pub struct Room {
    pub name: String,
    pub tiles: Vec<Tile>,
    pub graph: TileGraph,
}

impl Room {
    /// Load a room from a directory of .md tile files.
    pub fn load(path: &Path) -> Result<Self, String> {
        let name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        if !path.exists() {
            return Err(format!("room path does not exist: {}", path.display()));
        }

        let mut tiles = Vec::new();

        if path.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(path)
                .map_err(|e| format!("reading dir: {e}"))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().and_then(|s| s.to_str()) == Some("md")
                })
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let tile = Tile::from_file(&entry.path(), &name)?;
                tiles.push(tile);
            }
        }

        let graph = Self::build_graph(&tiles);
        Ok(Room { name, tiles, graph })
    }

    /// Compute dependency graph for a set of tiles.
    fn build_graph(tiles: &[Tile]) -> TileGraph {
        let mut graph = TileGraph::new();
        for tile in tiles {
            graph.add_tile(&tile.id, &tile.dependencies);
        }
        graph
    }

    /// Get the knowledge graph structure.
    pub fn knowledge_graph(&self) -> &TileGraph {
        &self.graph
    }

    /// Return tiles that are new compared to a previous version of the room.
    pub fn what_changed(&self, previous: &Room) -> Vec<&Tile> {
        let prev_ids: HashSet<&str> = previous.tiles.iter().map(|t| t.id.as_str()).collect();
        self.tiles.iter()
            .filter(|t| !prev_ids.contains(t.id.as_str()))
            .collect()
    }

    /// All retracted and failed tiles (failures first — read these first!).
    pub fn failures(&self) -> Vec<&Tile> {
        let mut fails: Vec<&Tile> = self.tiles.iter()
            .filter(|t| t.status == TileStatus::Retracted || t.status == TileStatus::Failed)
            .collect();
        fails.sort_by_key(|t| t.status);
        fails
    }

    /// Only verified tiles.
    pub fn verified(&self) -> Vec<&Tile> {
        self.tiles.iter()
            .filter(|t| t.status == TileStatus::Verified)
            .collect()
    }

    /// Tiles whose dependencies include at least one non-verified tile (conjectures, partial).
    pub fn frontier(&self) -> Vec<&Tile> {
        let status_map: HashMap<&str, TileStatus> = self.tiles.iter()
            .map(|t| (t.id.as_str(), t.status))
            .collect();

        self.tiles.iter()
            .filter(|t| {
                t.dependencies.iter().any(|dep| {
                    status_map.get(dep.as_str())
                        .map_or(true, |s| *s != TileStatus::Verified)
                })
            })
            .collect()
    }
}
