//! Triangle-based clustering metrics, value-exact match to networkx 3.x.
//!
//! Algorithm: for each pair of neighbours (u,v) of node w, count how many
//! nodes appear in both adj[u] and adj[v] — i.e., set intersection. Since adj
//! lists are sorted, each intersection costs O(min(deg(u), deg(v))).
//!
//! networkx `transitivity` uses `_triangles_and_degree_iter` which counts each
//! triangle twice per node (once per direction of the pair walk), so the raw
//! per-node `t` = 2 × actual triangles. The formulas below match that:
//!   local(v)   = t / (d*(d-1))        [= 2T/(d(d-1))]
//!   transitivity = Σt / Σ(d*(d-1))    [= Σ2T / Σ(d(d-1))]

use crate::io::Graph;

/// Per-node triangle counts, in node-insertion order.
///
/// Each actual triangle is counted once per participating node (networkx
/// `triangles()` semantics).
pub fn triangles(g: &Graph) -> Vec<u64> {
    // networkx's efficient path for triangles(G) (the all-nodes case):
    // orient each edge u→v with v not yet seen; for each oriented edge (u,v),
    // |later_nbrs[u] ∩ later_nbrs[v]| contributes to all three corners.
    // We replicate that exactly so counts match.
    let n = g.n();
    let mut later: Vec<Vec<u32>> = Vec::with_capacity(n);
    for u in 0..n {
        let u32 = u as u32;
        let ln: Vec<u32> = g.adj[u].iter().copied().filter(|&v| v > u32).collect();
        later.push(ln);
    }

    let mut counts = vec![0u64; n];
    for u in 0..n {
        for &v in &later[u] {
            // |later[u] ∩ later[v]|
            let m = intersect_count(&later[u], &later[v as usize]);
            counts[u] += m as u64;
            counts[v as usize] += m as u64;
            // Each third node w in the intersection: counts[w] += 1
            intersect_apply(&later[u], &later[v as usize], |w| {
                counts[w as usize] += 1;
            });
        }
    }
    counts
}

/// Per-node local clustering coefficients, in insertion order.
///
/// Matches networkx `clustering(G)` (unweighted, undirected):
///   c(v) = 2·T(v) / (d(v)·(d(v)−1)),  0 when d(v) < 2.
///
/// Internally uses `_triangles_and_degree_iter` where `t = 2·T`, so:
///   c(v) = t / (d*(d-1)).
pub fn local_clustering(g: &Graph) -> Vec<f64> {
    let n = g.n();
    let mut result = vec![0.0f64; n];
    for (v, r) in result.iter_mut().enumerate().take(n) {
        let d = g.adj[v].len();
        if d < 2 {
            continue;
        }
        // t = 2 × triangles through v (matches networkx _triangles_and_degree_iter)
        let t: u64 = g.adj[v]
            .iter()
            .map(|&u| {
                // |adj[v] ∩ adj[u]|  — networkx computes `set(G[w]) - {w}` but
                // since adj is already self-loop-free, we just intersect.
                intersect_count(&g.adj[v], &g.adj[u as usize]) as u64
            })
            .sum();
        let d64 = d as f64;
        *r = t as f64 / (d64 * (d64 - 1.0));
    }
    result
}

/// Average clustering coefficient.
///
/// Matches networkx `average_clustering(G)` = `sum(c.values()) / len(c)`.
/// CPython's `sum()` reduces floats with Neumaier compensated summation, so a
/// plain sequential fold diverges by a few ULP on larger graphs; `python_sum`
/// replicates the compensated reduction to stay bit-identical.
pub fn average_clustering(g: &Graph) -> f64 {
    let c = local_clustering(g);
    let n = c.len();
    if n == 0 {
        return 0.0;
    }
    python_sum(&c) / n as f64
}

/// The Neumaier compensated summation CPython's `sum()` runs over a float list.
fn python_sum(vals: &[f64]) -> f64 {
    let mut s = 0.0f64;
    let mut comp = 0.0f64;
    for &v in vals {
        let t = s + v;
        if s.abs() >= v.abs() {
            comp += (s - t) + v;
        } else {
            comp += (v - t) + s;
        }
        s = t;
    }
    s + comp
}

/// Global transitivity.
///
/// Matches networkx `transitivity(G)`:
///   triangles = Σ t(v)   [each t(v) = 2·T(v)]
///   contri    = Σ d(v)·(d(v)−1)
///   result    = 0  if triangles == 0  else triangles / contri
pub fn transitivity(g: &Graph) -> f64 {
    let n = g.n();
    let mut triangles_sum: u64 = 0;
    let mut contri_sum: u64 = 0;
    for v in 0..n {
        let d = g.adj[v].len() as u64;
        let t: u64 = g.adj[v]
            .iter()
            .map(|&u| intersect_count(&g.adj[v], &g.adj[u as usize]) as u64)
            .sum();
        triangles_sum += t;
        contri_sum += d * d.saturating_sub(1);
    }
    if triangles_sum == 0 {
        0.0
    } else {
        triangles_sum as f64 / contri_sum as f64
    }
}

/// Count |A ∩ B| for two sorted slices.
#[inline]
fn intersect_count(a: &[u32], b: &[u32]) -> u32 {
    let mut i = 0;
    let mut j = 0;
    let mut c = 0u32;
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                c += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    c
}

/// Call `f(w)` for each w in A ∩ B (both sorted).
#[inline]
fn intersect_apply(a: &[u32], b: &[u32], mut f: impl FnMut(u32)) {
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                f(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
}
