use std::fmt;

/// What kind of knowledge a tile represents.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TileKind {
    Fact,
    Proof,
    Failure,
    Benchmark,
    Code,
    Observation,
}

impl fmt::Display for TileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fact => write!(f, "fact"),
            Self::Proof => write!(f, "proof"),
            Self::Failure => write!(f, "failure"),
            Self::Benchmark => write!(f, "benchmark"),
            Self::Code => write!(f, "code"),
            Self::Observation => write!(f, "observation"),
        }
    }
}

impl std::str::FromStr for TileKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fact" => Ok(Self::Fact),
            "proof" => Ok(Self::Proof),
            "failure" => Ok(Self::Failure),
            "benchmark" => Ok(Self::Benchmark),
            "code" => Ok(Self::Code),
            "observation" => Ok(Self::Observation),
            other => Err(format!("unknown tile kind: {other}")),
        }
    }
}

/// Verification status of a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TileStatus {
    Failed = 0,
    Retracted = 1,
    Partial = 2,
    Conjecture = 3,
    Verified = 4,
}

impl fmt::Display for TileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verified => write!(f, "verified"),
            Self::Partial => write!(f, "partial"),
            Self::Retracted => write!(f, "retracted"),
            Self::Failed => write!(f, "failed"),
            Self::Conjecture => write!(f, "conjecture"),
        }
    }
}

impl std::str::FromStr for TileStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "verified" => Ok(Self::Verified),
            "partial" => Ok(Self::Partial),
            "retracted" => Ok(Self::Retracted),
            "failed" => Ok(Self::Failed),
            "conjecture" => Ok(Self::Conjecture),
            other => Err(format!("unknown tile status: {other}")),
        }
    }
}

/// A single unit of knowledge within a room.
#[derive(Debug, Clone)]
pub struct Tile {
    pub id: String,
    pub room: String,
    pub kind: TileKind,
    pub content: String,
    pub dependencies: Vec<String>,
    pub confidence: f64,
    pub status: TileStatus,
}

impl Tile {
    /// Parse a tile from a markdown string.
    ///
    /// Expected format:
    /// ```md
    /// ---
    /// id: tile-id
    /// room: room-name
    /// kind: fact
    /// confidence: 0.95
    /// status: verified
    /// depends: tile-a, tile-b
    /// ---
    /// Content here
    /// ```
    pub fn parse(markdown: &str) -> Result<Self, String> {
        let mut front_matter = false;
        let mut id = None;
        let mut room = None;
        let mut kind = None;
        let mut confidence = 0.0;
        let mut status = None;
        let mut dependencies = Vec::new();
        let mut content_lines = Vec::new();

        for line in markdown.lines() {
            if line.trim() == "---" {
                if !front_matter {
                    front_matter = true;
                    continue;
                } else {
                    // end of front matter
                    front_matter = false;
                    continue;
                }
            }

            if front_matter {
                if let Some(rest) = line.strip_prefix("id:") {
                    id = Some(rest.trim().to_string());
                } else if let Some(rest) = line.strip_prefix("room:") {
                    room = Some(rest.trim().to_string());
                } else if let Some(rest) = line.strip_prefix("kind:") {
                    kind = Some(rest.trim().parse()?);
                } else if let Some(rest) = line.strip_prefix("confidence:") {
                    confidence = rest.trim().parse::<f64>().map_err(|e| e.to_string())?;
                } else if let Some(rest) = line.strip_prefix("status:") {
                    status = Some(rest.trim().parse()?);
                } else if let Some(rest) = line.strip_prefix("depends:") {
                    let deps = rest.trim();
                    if !deps.is_empty() {
                        dependencies = deps.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            } else {
                content_lines.push(line);
            }
        }

        // After parsing, everything after front matter is content
        let content = content_lines.join("\n").trim().to_string();

        Ok(Tile {
            id: id.unwrap_or_default(),
            room: room.unwrap_or_default(),
            kind: kind.unwrap_or(TileKind::Fact),
            content,
            dependencies,
            confidence: confidence.clamp(0.0, 1.0),
            status: status.unwrap_or(TileStatus::Conjecture),
        })
    }

    /// Parse a tile from a file path, deriving defaults from filename.
    pub fn from_file(path: &std::path::Path, room_name: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;

        let mut tile = Self::parse(&content)?;

        // If front matter didn't set id, use filename without extension
        if tile.id.is_empty() {
            tile.id = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
        }

        if tile.room.is_empty() {
            tile.room = room_name.to_string();
        }

        Ok(tile)
    }
}
