# rsomics-clustering-coefficient

Triangle-based clustering metrics for undirected graphs — a value-exact, much faster port of **networkx**'s `triangles`, `clustering`, `average_clustering`, and `transitivity` for unweighted, undirected graphs.

## Usage

```
rsomics-clustering-coefficient [OPTIONS] [EDGELIST]
```

Read an edge list (one `u v` pair per line, whitespace-separated) from a file or stdin (`-`).
Comment lines starting with `#` and blank lines are ignored.
Self-loops are silently dropped. Duplicate edges collapse to a simple graph.

### Options

```
--metric <METRIC>   triangles | local | average | transitivity  [default: triangles]
--json              emit rsomics-common JSON envelope
-q, --quiet         suppress progress output
```

### Metrics

| `--metric`      | Output                                      |
|-----------------|---------------------------------------------|
| `triangles`     | Per-node TSV: `node\tcount` (insertion order) |
| `local`         | Per-node TSV: `node\tcoefficient`           |
| `average`       | Single scalar: average clustering coefficient |
| `transitivity`  | Single scalar: global transitivity          |

Output order matches networkx node-insertion order — the order each node label first appears in the edge list.

### Examples

```bash
# Triangle count per node
rsomics-clustering-coefficient --metric triangles network.el

# Average clustering coefficient
rsomics-clustering-coefficient --metric average network.el

# From stdin
cat network.el | rsomics-clustering-coefficient --metric transitivity -
```

## Accuracy

All four metrics are **bit-identical** to networkx 3.6.1 (Python 3.12):

- Triangle counts are exact integers — no floating point involved.
- Local clustering coefficients are one float division of exact integers — 0 ULP.
- Average clustering uses CPython 3.12's pairwise-blocked summation (WIDTH=8) to reproduce `sum()`'s bit-exact result.
- Transitivity is an exact integer ratio — 0 ULP.

No transcendental operations are involved, so results are fully reproducible across architectures.

## Origin

This crate is an independent Rust reimplementation of networkx's clustering metrics based on:

- The networkx 3.6.1 source (`networkx/algorithms/cluster.py`, BSD-3-Clause), which was **read and cited** (MIT/Apache-2.0 upstream — reading allowed and required per project policy). The `triangles()`, `_triangles_and_degree_iter()`, `clustering()`, `average_clustering()`, and `transitivity()` functions were read directly to match semantics exactly.
- The standard graph-theoretic definitions of local clustering coefficient and global transitivity.

Test fixtures (golden edge lists) are generated from networkx 3.6.1 with a fixed seed and frozen; tests do not invoke Python at runtime.

License: MIT OR Apache-2.0  
Upstream credit: NetworkX <https://networkx.org> (BSD-3-Clause)
