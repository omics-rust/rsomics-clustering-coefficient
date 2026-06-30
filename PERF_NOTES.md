# Performance Notes

## Fixture

`gnm_random_graph(50_000 nodes, 1_000_000 edges, seed=7)` written to
`/Volumes/KIOXIA/rsomics-fixtures/clustering-coefficient/gnm50k.el` (11.6 MB).

## Machine

- mini_m2 (Apple M2), macOS 25.5.0
- Single thread (both sides)
- hyperfine 1.x, 5 warmup 1 run 5 (compute-only) / 3 (end-to-end)

## Upstream reference

networkx 3.6.1, Python 3.12.13
`/opt/homebrew/Caskroom/miniforge/base/envs/scanpy/bin/python3`

## Compute-only (upstream: pre-built Graph; ours: I/O + compute)

| Metric            | Ours (mean)  | networkx (mean) | Ratio   |
|-------------------|-------------|-----------------|---------|
| triangles         | 384.7 ms    | 1 699 ms        | 4.42×   |
| local_clustering  | 882.5 ms    | 8 060 ms        | 9.13×   |
| average_clustering| 937.0 ms    | 8 977 ms        | 9.58×   |
| transitivity      | 879.1 ms    | 9 342 ms        | 10.63×  |

Note: networkx compute-only times exclude graph construction (~2.5s) and file
I/O (~0.5s). Our times include both I/O and compute.

## End-to-end (both read edgelist, build graph, compute)

| Metric            | Ours (mean) | networkx (mean) | Ratio   |
|-------------------|-------------|-----------------|---------|
| triangles         | 417.5 ms    | 4 861 ms        | 11.64×  |
| average_clustering| 988.3 ms    | 11 610 ms       | 11.75×  |
| transitivity      | 999.4 ms    | 13 153 ms       | 13.16×  |

All ratios > 1.0× — release gate PASSED.
