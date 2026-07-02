//! Value-exact compat tests against frozen networkx 3.6.1 goldens.
//!
//! Golden files live in `tests/golden/` and were generated once from networkx
//! 3.6.1 (BSD-3-Clause) in node-insertion-order (= first-seen from the edge
//! list). Tests do NOT invoke Python at runtime.
//!
//! Triangle counts: exact integer equality.
//! Local / average / transitivity: bit-exact (0 ULP) float equality via
//! `f64::from_bits(u64::from_str_radix(hex, 16))`.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-clustering-coefficient"))
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn run_metric(edgelist: &str, metric: &str) -> String {
    let path = golden_dir().join(edgelist);
    let out = Command::new(bin())
        .args(["--metric", metric, path.to_str().unwrap()])
        .output()
        .expect("run binary");
    assert!(
        out.status.success(),
        "binary failed (metric={metric}, file={edgelist}): {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn parse_per_node_ints(s: &str) -> Vec<(String, u64)> {
    s.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut it = l.splitn(2, '\t');
            let node = it.next().unwrap().to_owned();
            let val: u64 = it.next().unwrap().trim().parse().expect("integer");
            (node, val)
        })
        .collect()
}

fn parse_per_node_floats(s: &str) -> Vec<(String, f64)> {
    s.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut it = l.splitn(2, '\t');
            let node = it.next().unwrap().to_owned();
            let val: f64 = it.next().unwrap().trim().parse().expect("float");
            (node, val)
        })
        .collect()
}

fn parse_scalar_float(s: &str) -> f64 {
    s.trim().parse().expect("scalar float")
}

fn from_hexbits(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h, 16).expect("hex bits"))
}

fn load_expected() -> Vec<(String, String, String)> {
    let text = std::fs::read_to_string(golden_dir().join("expected.tsv")).unwrap();
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.splitn(3, '\t').collect();
            (c[0].to_owned(), c[1].to_owned(), c[2].to_owned())
        })
        .collect()
}

#[test]
fn value_exact_all_goldens() {
    let expected = load_expected();
    let mut checked = 0usize;

    for (graph, metric, want_str) in &expected {
        let edgelist = format!("{graph}.el");

        match metric.as_str() {
            "triangles" => {
                let got = parse_per_node_ints(&run_metric(&edgelist, "triangles"));
                let want: Vec<(String, u64)> = want_str
                    .split('|')
                    .map(|pair| {
                        let mut it = pair.splitn(2, ':');
                        let node = it.next().unwrap().to_owned();
                        let val: u64 = it.next().unwrap().parse().unwrap();
                        (node, val)
                    })
                    .collect();
                assert_eq!(
                    got.len(),
                    want.len(),
                    "{graph}/triangles: node count mismatch"
                );
                for ((gn, gv), (wn, wv)) in got.iter().zip(&want) {
                    assert_eq!(gn, wn, "{graph}/triangles: node label mismatch");
                    assert_eq!(gv, wv, "{graph}/triangles[{gn}]: got {gv} want {wv}");
                }
            }
            "local" => {
                let got = parse_per_node_floats(&run_metric(&edgelist, "local"));
                let want: Vec<(String, f64)> = want_str
                    .split('|')
                    .map(|pair| {
                        let mut it = pair.splitn(2, ':');
                        let node = it.next().unwrap().to_owned();
                        let val = from_hexbits(it.next().unwrap());
                        (node, val)
                    })
                    .collect();
                assert_eq!(got.len(), want.len(), "{graph}/local: node count mismatch");
                for ((gn, gv), (wn, wv)) in got.iter().zip(&want) {
                    assert_eq!(gn, wn, "{graph}/local: node label mismatch");
                    assert_eq!(
                        gv.to_bits(),
                        wv.to_bits(),
                        "{graph}/local[{gn}]: got {gv} ({:016x}) want {wv} ({:016x})",
                        gv.to_bits(),
                        wv.to_bits()
                    );
                }
            }
            "average" => {
                let got = parse_scalar_float(&run_metric(&edgelist, "average"));
                let want = from_hexbits(want_str);
                assert_eq!(
                    got.to_bits(),
                    want.to_bits(),
                    "{graph}/average: got {got} ({:016x}) want {want} ({:016x})",
                    got.to_bits(),
                    want.to_bits()
                );
            }
            "transitivity" => {
                let got = parse_scalar_float(&run_metric(&edgelist, "transitivity"));
                let want = from_hexbits(want_str);
                assert_eq!(
                    got.to_bits(),
                    want.to_bits(),
                    "{graph}/transitivity: got {got} ({:016x}) want {want} ({:016x})",
                    got.to_bits(),
                    want.to_bits()
                );
            }
            _ => panic!("unknown metric {metric}"),
        }
        checked += 1;
    }

    assert!(
        checked >= 32,
        "expected at least 32 golden checks, got {checked}"
    );
}
