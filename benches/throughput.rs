use barq_core::{Document, DocumentId, Value};
use barq_index::{DistanceMetric, HnswIndex};
use barq_vector::VectorOps;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use std::collections::HashMap;

fn bench_insert_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("insert_100k_single_threaded", |b| {
        b.to_async(&rt).iter(|| {
            let mut docs = Vec::with_capacity(100_000);
            for i in 0..100_000 {
                let mut doc = Document::new(DocumentId::new());
                doc.data.insert("index".to_string(), Value::Int(i as i64));
                doc.data
                    .insert("data".to_string(), Value::String(format!("data_{}", i)));
                docs.push(doc);
            }
            black_box(docs)
        });
    });
}

fn bench_point_lookup(c: &mut Criterion) {
    let mut index: HashMap<DocumentId, Document> = HashMap::new();
    let mut ids = Vec::new();

    for i in 0..50_000 {
        let id = DocumentId::new();
        let mut doc = Document::new(id.clone());
        doc.data.insert("index".to_string(), Value::Int(i as i64));
        ids.push(id.clone());
        index.insert(id, doc);
    }

    let mut rng = rand::thread_rng();

    c.bench_function("point_lookup_50k", |b| {
        b.iter(|| {
            let idx = rng.gen_range(0..ids.len());
            black_box(index.get(&ids[idx]).cloned())
        });
    });
}

fn bench_vector_search(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let dim = 128;

    let mut index = HnswIndex::new(dim, DistanceMetric::Cosine);

    for _ in 0..10_000 {
        let vector: Vec<f32> = (0..dim).map(|_| rng.gen()).collect();
        index.insert(DocumentId::new(), vector).unwrap();
    }

    let query: Vec<f32> = (0..dim).map(|_| rng.gen()).collect();

    c.bench_function("hnsw_search_k10_10k_vectors", |b| {
        b.iter(|| black_box(index.search(query.clone(), 10)));
    });
}

fn bench_vector_insert(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let dim = 128;

    c.bench_function("hnsw_insert_1k_vectors", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let mut index = HnswIndex::new(dim, DistanceMetric::Cosine);
                for _ in 0..1_000 {
                    let vector: Vec<f32> = (0..dim).map(|_| rng.gen()).collect();
                    index.insert(DocumentId::new(), vector).ok();
                }
                black_box(index)
            });
    });
}

criterion_group!(
    benches,
    bench_insert_throughput,
    bench_point_lookup,
    bench_vector_search,
    bench_vector_insert
);
criterion_main!(benches);
