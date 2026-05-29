# plato-loader — The Loading Program

Agents walk into a room, load knowledge, walk out knowing things. Parses knowledge tiles from [plato-room](https://github.com/SuperInstance/plato-room), builds dependency graphs, diffs against previous loads, and produces minimal updates.

**Part of the [Plato](https://github.com/SuperInstance/plato-shell) ecosystem.**

## What This Gives You

- **Parse tiles** — read knowledge units from a room (concepts, procedures, facts, code patterns)
- **Dependency DAGs** — resolve prerequisites before dependents
- **Incremental loads** — diff against previous loads, get only what changed
- **Bootstrap targets** — load only what's needed: `SpectralAnalysis`, `DomainSpecific`, or `Full`
- **Honest reports** — `LoadReport` with successes, failures, and missing prerequisites

## Quick Start

```rust
use plato_loader::{Loader, BootstrapTarget, Room};

let room = Room::from_path("./rooms/graph-theory/")?;
let loader = Loader::new(&room);

// Load only what's needed for spectral analysis
let digest = loader.bootstrap(BootstrapTarget::SpectralAnalysis)?;
println!("Loaded {} tiles ({} new)", digest.total(), digest.new_count());
```

### LoadReport

Every load produces a full accounting:

```rust
pub struct LoadReport {
    pub loaded: Vec<Tile>,        // successfully loaded
    pub new: Vec<Tile>,           // not seen before
    pub updated: Vec<Tile>,       // changed since last load
    pub failed: Vec<LoadFailure>, // couldn't load, with reasons
    pub missing: Vec<Dependency>, // prerequisites that weren't available
}
```

## How It Fits

Reads what [plato-room](https://github.com/SuperInstance/plato-room) writes. When an agent enters a knowledge room, the loader figures out what it needs, loads it in dependency order, and reports exactly what happened.

## Installation

```toml
[dependencies]
plato-loader = "0.1"
```

## Testing

```bash
cargo test
```

## License

MIT
