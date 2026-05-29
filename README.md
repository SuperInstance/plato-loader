# plato-loader

**The loading program.**

Agents walk into a room, load knowledge, walk out knowing things.

```text
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  plato-room  │────▶│ plato-loader  │────▶│   agent     │
│  (the tiles) │     │ (the program) │     │ (knows now) │
└─────────────┘     └──────────────┘     └─────────────┘
```

plato-loader reads what [plato-room](https://github.com/SuperInstance/plato-room) writes: knowledge tiles organized into rooms. It parses them, builds dependency graphs, diffs against previous loads, and produces minimal updates. An agent loading the same room twice gets only what changed.

## What It Does

1. **Parse tiles** — Read knowledge units from a room. Each tile is a self-contained chunk (concept, procedure, fact, code pattern).
2. **Build dependency graphs** — Tiles reference each other. Loader resolves the DAG so you load prerequisites before dependents.
3. **Diff against previous loads** — If you loaded this room before, loader figures out what's new, what changed, and what was removed.
4. **Produce minimal updates** — The output is the smallest diff that brings the agent's knowledge up to date.

## The Bootstrap Protocol

Not every agent needs everything. The bootstrap protocol lets an agent declare what it's loading for:

| Target | What Gets Loaded |
|--------|-----------------|
| `spectral-analysis` | Graph theory, eigenvalue methods, CR computation |
| `domain-specific` | Tiles tagged for a particular domain |
| `full` | Everything in the room |

```rust
use plato_loader::{Loader, BootstrapTarget, Room};

let room = Room::from_path("./rooms/graph-theory/")?;
let loader = Loader::new(&room);

// Load only what's needed for spectral analysis
let digest = loader.bootstrap(BootstrapTarget::SpectralAnalysis)?;

println!("Loaded {} tiles ({} new)", digest.total(), digest.new_count());
```

## LoadReport

Every load produces a `LoadReport` — an honest accounting of what happened:

```rust
pub struct LoadReport {
    pub loaded: Vec<Tile>,        // successfully loaded
    pub new: Vec<Tile>,           // not seen in previous loads
    pub updated: Vec<Tile>,       // changed since last load
    pub failed: Vec<LoadFailure>, // couldn't load, with reasons
    pub missing: Vec<Dependency>, // prerequisites that weren't available
}
```

No silent failures. If something went wrong, it's in the report.

```rust
let report = loader.load(&room)?;

if !report.failed.is_empty() {
    for fail in &report.failed {
        eprintln!("FAIL: {} — {}", fail.tile_id, fail.reason);
    }
}

if !report.missing.is_empty() {
    eprintln!("Missing dependencies:");
    for dep in &report.missing {
        eprintln!("  {} (required by {})", dep.id, dep.required_by);
    }
}
```

## The Failure-First Principle

Errors are not afterthoughts. The loader's design assumes things will fail:

- **Missing tiles** are reported, not silently skipped.
- **Circular dependencies** are detected and flagged before loading starts.
- **Corrupt tiles** fail individually — one bad tile doesn't abort the whole load.
- **Version mismatches** between loader and room format are caught early.

Every code path that can fail produces a `LoadFailure` with enough context to debug it.

## Connection to plato-room

plato-loader is the read side. [plato-room](https://github.com/SuperInstance/plato-room) is the write side. Rooms define the knowledge; loaders consume it.

```text
plato-room ──writes──▶ tiles/ ──reads──▶ plato-loader ──produces──▶ LoadReport
```

The two are versioned independently. The room format has a schema version; the loader checks it and fails cleanly if incompatible.

## Full Example

```rust
use plato_loader::{Loader, BootstrapTarget, Room};

// Point at a room (directory of tiles)
let room = Room::from_path("./rooms/graph-theory/")?;

// Create a loader with previous state (or empty for first load)
let loader = Loader::new(&room)
    .with_previous_state(".cache/graph-theory-state.json");

// Bootstrap for a specific purpose
let report = loader.bootstrap(BootstrapTarget::SpectralAnalysis)?;

// What happened?
println!("=== Load Report ===");
println!("Total loaded: {}", report.loaded.len());
println!("New knowledge: {}", report.new.len());
println!("Updated: {}", report.updated.len());

if !report.failed.is_empty() {
    println!("\nFailures:");
    for f in &report.failed {
        println!("  ✗ {} — {}", f.tile_id, f.reason);
    }
}

if !report.missing.is_empty() {
    println!("\nMissing dependencies:");
    for d in &report.missing {
        println!("  ? {} (needed by {})", d.id, d.required_by);
    }
}

// Digest: a compact summary the agent can carry
let digest = report.digest();
println!("\nDigest: {} tiles, {} bytes", digest.tile_count(), digest.size());
```

## Repository

[github.com/SuperInstance/plato-loader](https://github.com/SuperInstance/plato-loader)

## License

MIT

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance/OpenConstruct) ecosystem.
