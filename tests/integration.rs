use std::fs;
use std::path::PathBuf;

use plato_loader::*;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("plato-loader-test").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_tile(dir: &PathBuf, filename: &str, content: &str) {
    fs::write(dir.join(filename), content).unwrap();
}

#[test]
fn load_room_from_disk() {
    let dir = temp_dir("load_room");
    write_tile(&dir, "tile-a.md", "---\nid: tile-a\nroom: test\nkind: fact\nstatus: verified\nconfidence: 0.9\n---\nGravity exists.");
    write_tile(&dir, "tile-b.md", "---\nid: tile-b\nroom: test\nkind: proof\nstatus: verified\nconfidence: 1.0\ndepends: tile-a\n---\nProved via experiment.");

    let room = Room::load(&dir).unwrap();
    assert_eq!(room.name, "load_room");
    assert_eq!(room.tiles.len(), 2);
    assert_eq!(room.tiles[0].id, "tile-a");
    assert_eq!(room.tiles[1].id, "tile-b");
}

#[test]
fn tile_parsing_all_kinds_and_statuses() {
    let cases = vec![
        ("fact", TileKind::Fact, "verified", TileStatus::Verified),
        ("proof", TileKind::Proof, "partial", TileStatus::Partial),
        ("failure", TileKind::Failure, "retracted", TileStatus::Retracted),
        ("benchmark", TileKind::Benchmark, "failed", TileStatus::Failed),
        ("code", TileKind::Code, "conjecture", TileStatus::Conjecture),
        ("observation", TileKind::Observation, "verified", TileStatus::Verified),
    ];

    for (kind_str, expected_kind, status_str, expected_status) in cases {
        let md = format!(
            "---\nid: t\nroom: r\nkind: {kind_str}\nstatus: {status_str}\nconfidence: 0.5\n---\ncontent"
        );
        let tile = Tile::parse(&md).unwrap();
        assert_eq!(tile.kind, expected_kind, "kind mismatch for {kind_str}");
        assert_eq!(tile.status, expected_status, "status mismatch for {status_str}");
    }
}

#[test]
fn dependency_graph_edges() {
    let dir = temp_dir("dep_graph");
    write_tile(&dir, "a.md", "---\nid: a\nroom: g\nkind: fact\nstatus: verified\nconfidence: 1.0\n---\nA");
    write_tile(&dir, "b.md", "---\nid: b\nroom: g\nkind: proof\nstatus: verified\nconfidence: 1.0\ndepends: a\n---\nB");
    write_tile(&dir, "c.md", "---\nid: c\nroom: g\nkind: proof\nstatus: verified\nconfidence: 1.0\ndepends: a, b\n---\nC");

    let room = Room::load(&dir).unwrap();
    let graph = room.knowledge_graph();

    assert!(graph.edges["b"].contains("a"));
    assert!(graph.edges["c"].contains("a"));
    assert!(graph.edges["c"].contains("b"));
    assert!(graph.edges["a"].is_empty());
}

#[test]
fn what_changed_returns_only_new_tiles() {
    let dir = temp_dir("diff");
    write_tile(&dir, "a.md", "---\nid: a\nroom: d\nkind: fact\nstatus: verified\nconfidence: 1.0\n---\nA");

    let v1 = Room::load(&dir).unwrap();
    write_tile(&dir, "b.md", "---\nid: b\nroom: d\nkind: fact\nstatus: verified\nconfidence: 1.0\n---\nB");
    let v2 = Room::load(&dir).unwrap();

    let changed = v2.what_changed(&v1);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].id, "b");
}

#[test]
fn failures_returns_retracted_and_failed() {
    let dir = temp_dir("failures");
    write_tile(&dir, "bad.md", "---\nid: bad\nroom: f\nkind: failure\nstatus: failed\nconfidence: 0.1\n---\nNope");
    write_tile(&dir, "old.md", "---\nid: old\nroom: f\nkind: fact\nstatus: retracted\nconfidence: 0.3\n---\nRetracted");
    write_tile(&dir, "good.md", "---\nid: good\nroom: f\nkind: fact\nstatus: verified\nconfidence: 1.0\n---\nGood");

    let room = Room::load(&dir).unwrap();
    let fails = room.failures();

    assert_eq!(fails.len(), 2);
    // Failed < Retracted in Ord, so Failed comes first
    assert_eq!(fails[0].status, TileStatus::Failed);
    assert_eq!(fails[1].status, TileStatus::Retracted);
}

#[test]
fn verified_returns_only_verified() {
    let dir = temp_dir("verified");
    write_tile(&dir, "v.md", "---\nid: v\nroom: t\nkind: fact\nstatus: verified\nconfidence: 1.0\n---\nV");
    write_tile(&dir, "p.md", "---\nid: p\nroom: t\nkind: fact\nstatus: partial\nconfidence: 0.5\n---\nP");

    let room = Room::load(&dir).unwrap();
    let verified = room.verified();
    assert_eq!(verified.len(), 1);
    assert_eq!(verified[0].id, "v");
}

#[test]
fn frontier_returns_tiles_with_unverified_deps() {
    let dir = temp_dir("frontier");
    write_tile(&dir, "base.md", "---\nid: base\nroom: fr\nkind: fact\nstatus: conjecture\nconfidence: 0.5\n---\nMaybe");
    write_tile(&dir, "built.md", "---\nid: built\nroom: fr\nkind: proof\nstatus: verified\nconfidence: 1.0\ndepends: base\n---\nBuilt on conjecture");
    write_tile(&dir, "solo.md", "---\nid: solo\nroom: fr\nkind: fact\nstatus: verified\nconfidence: 1.0\n---\nStandalone");

    let room = Room::load(&dir).unwrap();
    let frontier = room.frontier();

    // "built" depends on "base" which is a conjecture
    assert!(frontier.iter().any(|t| t.id == "built"));
    // "base" has no deps, so its dependencies are empty — not frontier
    // "solo" has no deps — not frontier
}

#[test]
fn conservation_fully_connected() {
    let dir = temp_dir("conservation");
    write_tile(&dir, "a.md", "---\nid: a\nroom: c\nkind: fact\nstatus: verified\nconfidence: 1.0\ndepends: b, c\n---\nA");
    write_tile(&dir, "b.md", "---\nid: b\nroom: c\nkind: fact\nstatus: verified\nconfidence: 1.0\ndepends: a, c\n---\nB");
    write_tile(&dir, "c.md", "---\nid: c\nroom: c\nkind: fact\nstatus: verified\nconfidence: 1.0\ndepends: a, b\n---\nC");

    let mut loader = Loader::new();
    loader.load_room(&dir).unwrap();
    let cr = loader.conservation();
    // 3 tiles, each depends on 2 others = 6 deps, possible = 3*2 = 6, CR = 1.0
    assert!((cr - 1.0).abs() < 1e-9);
}

#[test]
fn gaps_identifies_missing_dependencies() {
    let dir = temp_dir("gaps");
    write_tile(&dir, "a.md", "---\nid: a\nroom: g\nkind: fact\nstatus: verified\nconfidence: 1.0\ndepends: missing-tile, also-missing\n---\nA");

    let mut loader = Loader::new();
    loader.load_room(&dir).unwrap();
    let gaps = loader.gaps();

    assert_eq!(gaps.len(), 2);
    assert!(gaps.contains(&"missing-tile".to_string()));
    assert!(gaps.contains(&"also-missing".to_string()));
}

#[test]
fn bootstrap_digest() {
    let base = temp_dir("bootstrap");
    let room_dir = base.join("bs");
    fs::create_dir_all(&room_dir).unwrap();
    write_tile(&room_dir, "t1.md", "---\nid: t1\nroom: bs\nkind: fact\nstatus: verified\nconfidence: 0.9\n---\nFact");

    let mut bs = Bootstrap::for_target(BootstrapTarget::FullKnowledge);
    let report = bs.load(&base).unwrap();

    assert_eq!(report.tiles_loaded, 1);
    assert_eq!(report.new_tiles, 1);

    let digest = bs.digest();
    assert!(digest.contains("# PLATO Load Digest"));
    assert!(digest.contains("bs"));
}
