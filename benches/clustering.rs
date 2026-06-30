use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};

use rsomics_clustering_coefficient::{cluster, io};

fn fixture() -> PathBuf {
    let p = PathBuf::from("/Volumes/KIOXIA/rsomics-fixtures/clustering-coefficient/gnm200.el");
    if p.exists() {
        return p;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/gnm200.el")
}

fn bench_clustering(c: &mut Criterion) {
    let g = io::read_edgelist(Some(fixture().as_path())).expect("load graph");

    c.bench_function("triangles/gnm200", |b| {
        b.iter(|| cluster::triangles(&g));
    });

    c.bench_function("local_clustering/gnm200", |b| {
        b.iter(|| cluster::local_clustering(&g));
    });

    c.bench_function("average_clustering/gnm200", |b| {
        b.iter(|| cluster::average_clustering(&g));
    });

    c.bench_function("transitivity/gnm200", |b| {
        b.iter(|| cluster::transitivity(&g));
    });
}

criterion_group!(benches, bench_clustering);
criterion_main!(benches);
