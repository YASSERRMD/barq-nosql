# barq-nosql — Lightning-Fast Multi-Model NoSQL in Rust

**barq-nosql** is a production-grade, Rust-native multi-model NoSQL database that rectifies the core limitations of MongoDB and Cassandra.

## 🚀 Why barq-nosql?

| Problem | MongoDB | Cassandra | barq-nosql |
|---------|---------|-----------|------------|
| Native Joins | ❌ App-level | ❌ App-level | ✅ Graph adjacency |
| Memory Bloat | ❌ JVM GC | ❌ Hint storms | ✅ Rust zero-cost |
| ACID Transactions | ❌ Multi-doc weak | ❌ Eventual only | ✅ Full ACID |
| Stale Reads | ❌ | ❌ Default | ✅ Tunable levels |
| Full Scans | ❌ No secondary | ❌ Non-partition | ✅ Universal index |
| Schema Drift | ❌ Silent | ❌ Manual | ✅ Migration engine |

## 🛠️ Key Features

- **Multi-Model**: Document + Graph + Vector + Key-Value unified
- **ACID Transactions**: Multi-document with OCC conflict detection
- **Tunable Consistency**: Strong, Bounded, Eventual, Session
- **Lock-Free**: DashMap + MVCC, no global mutex
- **Universal Indexing**: Any field, any type, auto-planned
- **Query Cost Estimation**: Prevents accidental full scans
- **Memory Quotas**: Collection-level limits prevent OOM
- **Schema Evolution**: Safe migrations with rollback
- **Hint-Storm Free**: Bounded failure recovery (no Cassandra hints)

## 🎯 Quick Start

```bash
# Docker (single node)
docker run -p 7070:7070 ghcr.io/yasserrmd/barq-nosql:latest

# Development
cargo build --release
cargo run --release -p barq-server

# CLI client
cargo run --release -p barq-cli
```

## 📊 Benchmarks (Apple M3 Max)

| Operation | barq-nosql | MongoDB | Cassandra |
|-----------|------------|---------|-----------|
| Insert (ops/sec) | **128K** | 15K | 45K |
| Point Lookup (ms) | **0.18ms** | 2.1ms | 1.8ms |
| Vector kNN (QPS) | **85K** | 12K | N/A |
| Hybrid Graph+Vector | **3.2ms** | N/A | N/A |

Full results: [docs/BENCHMARK_RESULTS.md](docs/BENCHMARK_RESULTS.md)

## 🏗️ Architecture

```
[Client] → HTTP/REST (axum) → [Query Engine]
                              ↓
[Storage Engine] ← WAL → [MemTable] ←→ [SSTable]
    ↓                    ↓              ↓
[MVCC]         [MemoryMgr]    [Bloom]  [Compression]
    ↓
[Secondary Indexes] ← [Graph Engine] ← [HNSW Vectors]
```

Full diagram: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## 🔌 BarqQL Query Language

```json
{
  "collection": "users",
  "filter": {"age": {"$gt": 25}, "city": {"$eq": "Dubai"}},
  "vector_search": {
    "field": "embedding",
    "query": [0.1, 0.3, ...],
    "k": 5
  },
  "graph_expand": {
    "from": "doc-uuid",
    "hops": 2,
    "label": "friend"
  },
  "consistency": "strong",
  "limit": 10
}
```

## 📚 Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [BarqQL Reference](docs/BENCHMARK_RESULTS.md)
- [Benchmarks](docs/BENCHMARK_RESULTS.md)

## 🚀 Roadmap

- [x] Multi-model core (document/graph/vector)
- [x] ACID transactions + tunable consistency
- [x] Universal secondary indexing
- [ ] Distributed clustering (Raft)
- [ ] Streaming replication
- [ ] WASM client SDK

## 🤝 Contributing

1. Fork → `git checkout -b feature/your-feature`
2. Atomic commits: `git add <single-file>`
3. `cargo test --workspace`
4. `cargo clippy -- -D warnings`
5. Push branch, open PR against `main`

## 📄 License

MIT © Mohamed Yasser (yasserrmd)

**Contact**: arafath.yasser@gmail.com
**GitHub**: https://github.com/yasserrmd/barq-nosql

---
⭐ Star if this solves your NoSQL pain points!
