//! A `#` comment anywhere on a line must be stripped before tokenising, so an
//! inline-commented edge list yields the same graph as its comment-free twin
//! (networkx `parse_edgelist` semantics).

use std::io::Write;

use rsomics_clustering_coefficient::cluster::{local_clustering, transitivity, triangles};
use rsomics_clustering_coefficient::io::read_edgelist;

fn write_el(text: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(text.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn inline_comment_matches_comment_free() {
    let commented = write_el("0 1\n1 2#note\n# whole-line comment\n2 3\n0 2   # close triangle\n");
    let clean = write_el("0 1\n1 2\n2 3\n0 2\n");

    let gc = read_edgelist(Some(commented.path())).unwrap();
    let gk = read_edgelist(Some(clean.path())).unwrap();

    assert_eq!(gc.labels, gk.labels, "node labels diverge");
    assert_eq!(gc.adj, gk.adj, "adjacency diverges");

    assert_eq!(triangles(&gc), triangles(&gk));
    assert_eq!(local_clustering(&gc), local_clustering(&gk));
    assert_eq!(transitivity(&gc).to_bits(), transitivity(&gk).to_bits());
}
