use std::path::Path;
use std::time::Instant;

use crate::loader::Loader;

/// What to load.
#[derive(Debug, Clone)]
pub enum BootstrapTarget {
    /// Load everything about Laplacians and conservation.
    SpectralAnalysis,
    /// Load a specific named room.
    DomainExpertise(String),
    /// Load everything under the base path.
    FullKnowledge,
}

/// Summary of a load operation.
#[derive(Debug, Clone)]
pub struct LoadReport {
    pub tiles_loaded: usize,
    pub new_tiles: usize,
    pub failures_found: usize,
    pub gaps_identified: usize,
    pub conservation: f64,
    pub time_ms: u64,
}

/// Bootstrap orchestrator: load rooms for a specific purpose.
#[derive(Debug)]
pub struct Bootstrap {
    pub loader: Loader,
    pub target: BootstrapTarget,
}

impl Bootstrap {
    pub fn for_target(target: BootstrapTarget) -> Self {
        Self {
            loader: Loader::new(),
            target,
        }
    }

    /// Load rooms from disk according to the target.
    ///
    /// For `FullKnowledge`, loads all subdirectories of `base_path`.
    /// For `DomainExpertise(name)`, loads only `base_path/name`.
    /// For `SpectralAnalysis`, loads rooms matching spectral/laplacian/conservation keywords.
    pub fn load(&mut self, base_path: &Path) -> Result<LoadReport, String> {
        let start = Instant::now();
        let tiles_before = self.loader.known.len();

        let dirs_to_load = self.resolve_dirs(base_path)?;

        for dir in &dirs_to_load {
            let _ = self.loader.load_room(dir);
        }

        let tiles_after: usize = self.loader.rooms.values()
            .map(|r| r.tiles.len())
            .sum();
        let tiles_loaded = tiles_after;
        let new_tiles = tiles_after.saturating_sub(tiles_before);
        let failures_found = self.loader.rooms.values()
            .flat_map(|r| r.failures())
            .count();
        let gaps_identified = self.loader.gaps().len();
        let conservation = self.loader.conservation();
        let time_ms = start.elapsed().as_millis() as u64;

        Ok(LoadReport {
            tiles_loaded,
            new_tiles,
            failures_found,
            gaps_identified,
            conservation,
            time_ms,
        })
    }

    /// Markdown summary of what was loaded.
    pub fn digest(&self) -> String {
        let mut out = String::new();
        out.push_str("# PLATO Load Digest\n\n");

        for (name, room) in &self.loader.rooms {
            out.push_str(&format!("## Room: {name}\n"));
            out.push_str(&format!("- Tiles: {}\n", room.tiles.len()));
            out.push_str(&format!("- Verified: {}\n", room.verified().len()));
            out.push_str(&format!("- Failures: {}\n", room.failures().len()));
            out.push_str(&format!("- Frontier: {}\n\n", room.frontier().len()));
        }

        let gaps = self.loader.gaps();
        if !gaps.is_empty() {
            out.push_str(&format!("## Gaps ({})\n", gaps.len()));
            for gap in &gaps {
                out.push_str(&format!("- {gap}\n"));
            }
            out.push('\n');
        }

        out.push_str(&format!("**Conservation ratio:** {:.4}\n", self.loader.conservation()));
        out
    }

    fn resolve_dirs(&self, base_path: &Path) -> Result<Vec<std::path::PathBuf>, String> {
        if !base_path.exists() {
            return Err(format!("base path does not exist: {}", base_path.display()));
        }

        let spectral_keywords = ["spectral", "laplacian", "conservation", "graph", "eigenvalue"];

        let mut dirs = Vec::new();
        if base_path.is_dir() {
            for entry in std::fs::read_dir(base_path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    match &self.target {
                        BootstrapTarget::FullKnowledge => {
                            dirs.push(path);
                        }
                        BootstrapTarget::DomainExpertise(domain) => {
                            if name.contains(&domain.to_lowercase()) {
                                dirs.push(path);
                            }
                        }
                        BootstrapTarget::SpectralAnalysis => {
                            if spectral_keywords.iter().any(|kw| name.contains(kw)) {
                                dirs.push(path);
                            }
                        }
                    }
                }
            }
        }

        Ok(dirs)
    }
}
