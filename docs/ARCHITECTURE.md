# BARQ-NOSQL Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Client                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     REST API Server                         │
│                      (axum :7070)                          │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Query Engine   │ │  Index Manager  │ │  Graph Engine   │
│   (BarqQL)     │ │ (BTree/HNSW)    │ │   (BFS/DFS)     │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                  │                  │
         └──────────────────┼──────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Storage Engine                           │
│  ┌──────────┐    ┌──────────┐    ┌──────────────┐       │
│  │   WAL    │───▶│ MemTable │───▶│   SSTable    │       │
│  └──────────┘    └──────────┘    └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

### Write Path
1. Document received via REST API
2. Written to WAL (Write-Ahead Log)
3. Inserted into MemTable (in-memory)
4. When threshold hit → flush to SSTable (disk)

### Read Path
1. Check MemTable first
2. On miss → query SSTable
3. Return merged result

## Storage Engine

### WAL (Write-Ahead Log)
- Append-only log for crash recovery
- CRC32 checksums for integrity
- fsync after each write

### MemTable
- Lock-free using DashMap
- In-memory hash map
- Flush threshold: 10k docs or 64MB

### SSTable
- Sorted disk segments
- LZ4 compression
- CRC32 checksums
- Sparse index every 128 entries

## Index Types

### B-Tree Index
- For scalar fields (Int, Float, String)
- Range queries
- O(log n) lookup

### Inverted Index
- Full-text search
- Tokenized on whitespace
- AND/OR operators

### HNSW Vector Index
- Approximate nearest neighbor
- Multi-layer graph
- Supports Cosine, Dot, Euclidean metrics

## Graph Engine

- Documents ARE graph nodes
- Edges link DocumentIds
- BFS/DFS traversal
- Hybrid graph+vector queries

## Query Engine

### BarqQL (JSON-based DSL)

```json
{
  "collection": "users",
  "filter": {"age": {"$gt": 25}},
  "limit": 10,
  "sort": {"age": "desc"}
}
```

## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| port | 7070 | Server port |
| wal_enabled | true | Enable WAL |
| flush_threshold_docs | 10000 | MemTable flush |
| flush_threshold_bytes | 64MB | MemTable flush |
