# Analytics cache architecture

Research and design for caching expensive DuckDB analytics queries in the ironstar template.

## Problem statement

DuckDB queries for dashboards are expensive (seconds, not milliseconds) and dashboards are hit frequently by multiple users.
The current architecture uses:
- SQLite + sqlx for the event store (append-only log)
- DuckDB for querying external scientific datasets (HuggingFace Hub, S3-compatible storage, DuckLake via httpfs)
- tokio::sync::broadcast for in-process pub/sub for SSE notifications

This document evaluates caching strategies to avoid redundant expensive DuckDB queries while maintaining consistency with the event-sourced architecture.

## Evaluation criteria

| Criterion | Weight | Description |
|-----------|--------|-------------|
| Async compatibility | High | Must integrate cleanly with tokio async runtime |
| Binary blob storage | High | Analytics results are serialized, not structured KV |
| Invalidation precision | Medium | Per-aggregate-type or time-based invalidation |
| Persistence | Low | Cache is rebuildable from DuckDB; nice-to-have |
| Memory footprint | Medium | Modest hardware target |
| Complexity | Medium | Prefer simpler solutions |

## Candidate evaluation

### 1. moka (in-memory cache)

[moka](https://github.com/moka-rs/moka) is a high-performance concurrent caching library for Rust inspired by Java's Caffeine.

**Strengths:**
- Async-native with `moka::future::Cache`
- TTL (time-to-live) and size-based eviction built in
- Lock-free concurrent access (no Arc<Mutex<...>> needed)
- LFU-based eviction maintains near-optimal hit ratio
- Eviction listener support for cleanup callbacks
- Actively maintained, MSRV 1.70+

**Weaknesses:**
- No persistence (cache lost on restart)
- Memory-only (limited by available RAM)

**API pattern:**

```rust
use moka::future::Cache;
use std::time::Duration;

// Create cache with TTL and size limit
let cache: Cache<String, Vec<u8>> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300)) // 5 minute TTL
    .build();

// Insert serialized analytics result
cache.insert(cache_key, serialized_result).await;

// Retrieve (returns Option)
if let Some(result) = cache.get(&cache_key).await {
    return deserialize(result);
}
```

### 2. redb (embedded ACID KV)

[redb](https://github.com/cberner/redb) is an embedded key-value database in pure Rust with ACID semantics.

**Analysis from design.md:**
- Copy-on-write B-tree structure (never corrupts on crash)
- Memory-mapped reads provide excellent read performance for hot data
- Configurable durability: `Durability::None` (non-durable), `Durability::Eventual` (1PC+C with checksum), `Durability::Immediate` (2PC)
- Supports arbitrary byte slices as values (suitable for binary blobs)
- Single-writer, multiple-reader MVCC isolation
- XXH3_128 checksums detect partial writes

**Key design considerations from redb's design.md:**
- "Non-durable commits" provide atomicity without fsync (good for cache)
- Savepoints enable point-in-time snapshots (not needed for cache)
- Memory-mapped I/O means hot data stays in page cache

**Weaknesses:**
- Synchronous API (needs `spawn_blocking` in async handlers)
- No built-in TTL (must implement expiration logic)
- Persistence overhead (unnecessary for pure cache use case)

**API pattern:**

```rust
use redb::{Database, TableDefinition, Durability};

const ANALYTICS_CACHE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("analytics_cache");

// Open database
let db = Database::create("analytics_cache.redb")?;

// Write with non-durable (cache-appropriate) durability
let txn = db.begin_write()?;
txn.set_durability(Durability::None);
{
    let mut table = txn.open_table(ANALYTICS_CACHE)?;
    table.insert(cache_key, serialized_result.as_slice())?;
}
txn.commit()?;

// Read (memory-mapped, fast for hot data)
let txn = db.begin_read()?;
let table = txn.open_table(ANALYTICS_CACHE)?;
let result = table.get(cache_key)?;
```

### 3. SQLite table (cache alongside event store)

Use a dedicated SQLite table in the existing database for cache storage.

**Strengths:**
- Already in the stack (no new dependency)
- sqlx provides async interface
- Can use WAL mode shared with event store

**Weaknesses:**
- Mixes cache (ephemeral) with source-of-truth (durable)
- SQLite single-writer constraint affects both cache and events
- Conceptually confusing (cache should be separate from primary data)

**Not recommended:** Violates separation of concerns and risks write contention.

### 4. sled (embedded database)

[sled](https://github.com/spacejam/sled) is a lock-free embedded database in Rust.

**Strengths:**
- Lock-free concurrent access
- Append-only log structure with crash safety
- Watch/subscribe functionality for change notifications

**Weaknesses:**
- Unstable 1.0-alpha status (edition = "2024")
- Known space amplification issues ("uses too much space sometimes")
- Author working on major rewrite (komora/marble storage engine)
- Less predictable than redb for write-heavy workloads

**Not recommended:** Stability concerns for a template project.
Benchmark data shows redb outperforms sled on individual writes (227ms vs 642ms) and large writes (8805ms vs 37736ms).

## Serialization format evaluation

Analytics query results must be serialized for cache storage.
The choice affects both storage efficiency and read performance.

### Comparison matrix

| Format | Serialize | Deserialize | Size | Zero-copy | DuckDB native |
|--------|-----------|-------------|------|-----------|---------------|
| bincode | Fast | Fast | Compact | No | No |
| rkyv | Fast | **Ultra-fast** (21ns) | Compact | **Yes** | No |
| Arrow IPC | Fast | Fast | Larger | Yes | **Yes** |
| JSON | Slow | Slow | Large | No | No |

### Analysis

**bincode:**
- General-purpose, well-understood
- Good performance for both serialize and deserialize
- ~300ns deserialize (vs rkyv's 21ns)
- Already used in ironstar for session storage (redb)

**rkyv:**
- Zero-copy deserialization is ideal for read-heavy caches
- 10-15x faster deserialization than bincode
- Requires `#[derive(Archive, Serialize, Deserialize)]` on types
- Schema evolution is more constrained than bincode

**Arrow IPC:**
- Native DuckDB format eliminates encode/decode for columnar data
- "Arrow's ability to eliminate encoding and decoding overheads typically yields faster and more efficient data interchange"
- Best for scenarios where query results are columnar and may be re-processed
- Larger than bincode/rkyv for small results

### Recommendation

Use **rkyv** for analytics cache serialization:
- Zero-copy deserialization aligns with cache read-heavy access pattern
- Benchmarks show 10-15x deserialization speedup over bincode
- Analytics results are typically read many times per write

For structured columnar results that need further processing (e.g., feeding into visualization), consider Arrow IPC for interoperability.

## Architecture decision

### Recommendation: moka with rkyv serialization

> **Semantic foundation**: The cache implements memoization over the query profunctor.
> TTL-based invalidation approximates naturality failure detection.
> See [denotational-semantics.md § Memoization](../core/denotational-semantics.md#duckdb-analytics-as-quotient-with-memoization).

For ironstar's single-node deployment target with rebuildable cache:

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Cache layer | moka | Async-native, TTL built-in, no persistence overhead |
| Serialization | rkyv | Zero-copy deserialization for read-heavy pattern |
| Invalidation | Per-aggregate-type + time-window | Precise invalidation, minimal over-invalidation |
| SSE integration | Separate analytics_bus | Isolates analytics updates from domain events |

### Why not redb for cache?

redb is excellent for session storage (where persistence is valuable) but introduces unnecessary complexity for analytics cache:
- Synchronous API requires spawn_blocking wrappers
- Persistence is unnecessary (cache is rebuildable from DuckDB)
- Manual TTL implementation adds complexity
- Memory-mapped I/O is overkill when data fits in memory

### Hybrid approach for future consideration

If cache persistence becomes valuable (e.g., faster startup with warm cache), consider:

```rust
use async_duckdb::Pool;

/// Layered cache: moka (hot) -> redb (warm) -> DuckDB (cold)
pub struct LayeredAnalyticsCache {
    hot: Cache<String, Vec<u8>>,          // moka (sub-ms access)
    warm: Option<Database>,                // redb (10ms access)
    cold: Arc<Pool>,                       // async-duckdb pool (seconds access)
}

impl LayeredAnalyticsCache {
    pub async fn get(&self, key: &str) -> Result<Vec<u8>, Error> {
        // L1: moka (in-memory)
        if let Some(data) = self.hot.get(key).await {
            return Ok(data);
        }

        // L2: redb (persistent, optional)
        if let Some(ref db) = self.warm {
            if let Some(data) = self.get_from_redb(db, key)? {
                self.hot.insert(key.to_string(), data.clone()).await;
                return Ok(data);
            }
        }

        // L3: async-duckdb pool (compute)
        let data = self.execute_and_serialize(key).await?;
        self.hot.insert(key.to_string(), data.clone()).await;
        if let Some(ref db) = self.warm {
            self.write_to_redb(db, key, &data)?;
        }

        Ok(data)
    }

    async fn execute_and_serialize(&self, query_key: &str) -> Result<Vec<u8>, Error> {
        // Execute analytics query via pool
        let results = self.cold.conn(|conn| {
            // Parse query_key and execute appropriate DuckDB query
            execute_analytics_query(conn, query_key)
        }).await?;

        // Serialize results
        Ok(bincode::serialize(&results)?)
    }
}
```

This layered approach adds the redb warm layer only when persistence justifies the added complexity.

For implementation patterns and Zenoh-based invalidation, see `analytics-cache-patterns.md`.

## Tradeoff summary

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| moka only | Simple, async-native, TTL built-in | No persistence, memory-bound | **Use for v1** |
| redb only | Persistent, memory-mapped reads | Sync API, manual TTL, overkill | Use for session store |
| moka + redb | Fast + persistent warm layer | Added complexity | Consider for v2 if needed |
| SQLite table | Already in stack | Mixes cache with source-of-truth | Not recommended |
| sled | Lock-free | Unstable 1.0-alpha, space issues | Not recommended |

## References

- [moka documentation](https://docs.rs/moka/latest/moka/)
- [moka GitHub](https://github.com/moka-rs/moka)
- [rkyv benchmark](https://david.kolo.ski/blog/rkyv-is-faster-than/)
- [rust serialization benchmark](https://github.com/djkoloski/rust_serialization_benchmark)
- [redb design.md](/Users/crs58/projects/rust-workspace/redb/docs/design.md)
- [redb 1.0 release](https://www.redb.org/post/2023/06/16/1-0-stable-release/)
- [Arrow IPC in DuckDB](https://duckdb.org/2025/05/23/arrow-ipc-support-in-duckdb)
- [sled GitHub](https://github.com/spacejam/sled)

## Related documentation

- Cache implementation patterns: `analytics-cache-patterns.md`
- Event sourcing core concepts: `../cqrs/event-sourcing-core.md`
- Projection patterns: `../cqrs/projection-patterns.md`
- Zenoh event bus integration: `zenoh-event-bus.md`
- Architecture decisions: `../core/architecture-decisions.md` (section 6: DuckDB)
- Session storage with redb: `../core/architecture-decisions.md` (section 5: redb)
