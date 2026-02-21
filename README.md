# BARQ-NOSQL

Lightning-fast multi-model NoSQL database written in Rust.

## Features

- **Document Store**: Flexible JSON-like documents with schema validation
- **Graph**: Native graph relationships with BFS/DFS traversal
- **Vector Search**: HNSW indexing for similarity search
- **Key-Value**: High-performance in-memory storage
- **MVCC**: Multi-Version Concurrency Control
- **WAL**: Write-Ahead Log for crash recovery
- **Async**: Built on tokio for high concurrency

## Quick Start

```bash
# Build
cargo build --release

# Run server
cargo run --release -p barq-server

# Or use CLI
cargo run --release -p barq-cli -- --help
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture.

## REST API

Server runs on `http://localhost:7070`

```bash
# Health check
curl http://localhost:7070/health

# Create collection
curl -X POST http://localhost:7070/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "users"}'

# Insert document
curl -X POST http://localhost:7070/collections/users/documents \
  -H "Content-Type: application/json" \
  -d '{"id": "uuid", "data": {"name": "Alice", "age": 30}}'
```

## CLI Commands

```bash
# Create collection
barq-cli create-collection --name users

# Insert document
barq-cli insert --collection users --doc '{"name": "Alice"}'

# Query
barq-cli query --collection users --filter '{"age": {"$gt": 25}}'

# Vector search
barq-cli vector --collection items --field embedding --query '[0.1,0.2]' --k 5
```

## Crates

| Crate | Description |
|-------|-------------|
| barq-core | Types, schema, errors |
| barq-storage | WAL, MemTable, SSTable |
| barq-index | BTree, Inverted, HNSW |
| barq-graph | Graph traversal |
| barq-query | Query engine |
| barq-server | REST API |
| barq-cli | CLI client |

## License

MIT
