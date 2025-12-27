# Event bus fallback options

**Default architecture**: Ironstar uses Zenoh in embedded mode as the primary event bus.

This document describes the optional tokio::broadcast fallback for template users with extreme resource constraints, and the coexistence patterns for adapting ironstar to existing codebases.
For the canonical Zenoh-first architecture, see `zenoh-event-bus.md`.
For infrastructure decision rationale, see `../decisions/infrastructure-decisions.md`.

## When to consider the tokio::broadcast fallback

**Default choice**: Zenoh embedded mode is the canonical event bus architecture for ironstar.

**Optional fallback**: Consider tokio::broadcast only if:
1. Deployment environment has extreme memory constraints (<10MB available for event bus)
2. Latency requirements are <50μs (Zenoh embedded mode adds ~100μs vs tokio::broadcast's <1μs)
3. Application will never scale beyond a single node with <256 concurrent SSE clients and <1000 events/sec

**Single-node scaling limits for tokio::broadcast:**

tokio::broadcast becomes a bottleneck at:
- ~256 concurrent SSE clients (subscribers)
- ~1000 events/second throughput

These limits are imposed by in-memory channel capacity and lock contention.

**When to use Zenoh instead (the default):**

Use Zenoh (the canonical architecture) when:
1. Standard deployment resources available (Zenoh embedded mode uses ~2MB memory)
2. Future scaling may require multi-node deployment (horizontal scaling)
3. Cross-service event distribution may be needed (microservices pattern)
4. Key expression filtering provides architectural value (aggregate-type routing)

## Zenoh embedded mode — the canonical architecture

**Algebraic justification:**

Zenoh's key-expression model is a *free monoid* over path segments:

```rust
// Key expressions form a monoid under path concatenation
// Pattern matching is a semilattice (wildcards, unions)
let key = format!("events/{}/{}/{}", aggregate_type, aggregate_id, sequence);

// Put is an effectful operation (IO monad)
session.put(&key, payload).await?;

// Subscribe returns a stream (comonadic, produces values)
let subscriber = session.subscribe("events/**").await?;
```

**Why Zenoh is the default choice:**

- Pure Rust implementation (no external server dependency)
- Embedded mode requires only 4 configuration lines
- Key expression filtering enables sophisticated routing patterns
- Distribution-ready without code changes (just configuration)
- Both streaming and storage in one abstraction
- Storage backends (RocksDB, S3) provide durability for future use
- Production-ready (Eclipse Foundation, April 2025 Gozuryū release)

## Coexistence strategy: DualEventBus for backward compatibility

The DualEventBus pattern supports both Zenoh and tokio::broadcast simultaneously, allowing template users to adapt ironstar to existing broadcast-based codebases gradually.

**Note**: New ironstar instantiations use Zenoh-only by default.
This pattern is only needed when adapting ironstar to existing systems with tokio::broadcast dependencies.

**Trait abstraction:**

```rust
use async_trait::async_trait;
use futures::stream::Stream;

// Trait abstraction allows swapping implementations
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, events: Vec<Event>) -> Result<Vec<StoredEvent>>;
    async fn load(&self, aggregate_id: &str) -> Result<Vec<StoredEvent>>;
    fn subscribe(&self) -> impl Stream<Item = StoredEvent>;
}

// Default: SQLite + Zenoh embedded implementation
pub struct ZenohEventStore {
    pool: SqlitePool,
    session: zenoh::Session
}

// Fallback: SQLite + tokio::broadcast implementation (for extreme resource constraints)
pub struct BroadcastEventStore {
    pool: SqlitePool,
    bus: EventBus
}

// Future: Zenoh storage backend (fully distributed, no SQLite dependency)
pub struct ZenohStorageEventStore {
    session: zenoh::Session,
    // No SQLite dependency - uses Zenoh storage backend
}
```

**DualEventBus implementation:**

```rust
use tokio::sync::broadcast;
use zenoh::prelude::r#async::*;

pub enum EventBusMode {
    Zenoh(zenoh::Session),                    // Default: Zenoh-only
    Broadcast(broadcast::Sender<StoredEvent>), // Fallback: broadcast-only
    Dual {                                     // Coexistence: both for backward compatibility
        zenoh: zenoh::Session,
        broadcast: broadcast::Sender<StoredEvent>,
    },
}

pub struct DualEventBus {
    mode: EventBusMode,
}

impl DualEventBus {
    pub async fn new_zenoh(config: zenoh::config::Config) -> Result<Self, zenoh::Error> {
        let session = zenoh::open(config).res().await?;
        Ok(Self {
            mode: EventBusMode::Zenoh(session),
        })
    }

    pub fn new_broadcast(tx: broadcast::Sender<StoredEvent>) -> Self {
        Self {
            mode: EventBusMode::Broadcast(tx),
        }
    }

    pub async fn new_dual(
        config: zenoh::config::Config,
        tx: broadcast::Sender<StoredEvent>,
    ) -> Result<Self, zenoh::Error> {
        let session = zenoh::open(config).res().await?;
        Ok(Self {
            mode: EventBusMode::Dual {
                zenoh: session,
                broadcast: tx,
            },
        })
    }

    pub async fn publish(&self, event: &StoredEvent) -> Result<(), Error> {
        match &self.mode {
            EventBusMode::Zenoh(session) => {
                let key = event_key(event);
                let payload = serde_json::to_vec(event)?;
                session.put(&key, payload).res().await?;
            }
            EventBusMode::Broadcast(tx) => {
                tx.send(event.clone())
                    .map_err(|_| Error::BroadcastFailed)?;
            }
            EventBusMode::Dual { zenoh, broadcast } => {
                // Publish to both buses (Zenoh primary, broadcast for compatibility)
                let key = event_key(event);
                let payload = serde_json::to_vec(event)?;
                let _ = zenoh.put(&key, payload).res().await;
                let _ = broadcast.send(event.clone());
            }
        }
        Ok(())
    }

    pub async fn subscribe(&self) -> Result<EventSubscription, Error> {
        match &self.mode {
            EventBusMode::Zenoh(session) => {
                let subscriber = session.declare_subscriber("events/**").res().await?;
                Ok(EventSubscription::Zenoh(subscriber))
            }
            EventBusMode::Broadcast(tx) => {
                Ok(EventSubscription::Broadcast(tx.subscribe()))
            }
            EventBusMode::Dual { zenoh, .. } => {
                // In dual mode, new subscribers use Zenoh (the primary)
                let subscriber = zenoh.declare_subscriber("events/**").res().await?;
                Ok(EventSubscription::Zenoh(subscriber))
            }
        }
    }
}

pub enum EventSubscription {
    Broadcast(broadcast::Receiver<StoredEvent>),
    Zenoh(zenoh::subscriber::Subscriber<'static, ()>),
}

fn event_key(event: &StoredEvent) -> String {
    format!(
        "events/{}/{}/{}",
        event.aggregate_type, event.aggregate_id, event.sequence
    )
}
```

## Key expression patterns

For complete Zenoh key expression patterns including wildcards (`*`, `**`, `$*`), subscription semantics, and session-scoped routing, see `zenoh-event-bus.md`.

Zenoh uses hierarchical event keys for sophisticated routing:

```
events/{aggregate_type}/{aggregate_id}/{sequence}
```

This structure enables aggregate-type filtering (`events/Todo/**`), instance-specific subscriptions (`events/Todo/uuid-123/**`), and cache invalidation patterns.
See `analytics-cache-patterns.md` Pattern 4 for Zenoh-based cache invalidation.

## Deployment modes

**Default: Zenoh embedded mode**

```rust
let mut config = zenoh::config::Config::default();
config.insert_json5("listen/endpoints", "[]").unwrap();
config.insert_json5("connect/endpoints", "[]").unwrap();
config.insert_json5("scouting/multicast/enabled", "false").unwrap();
config.insert_json5("scouting/gossip/enabled", "false").unwrap();

let bus = DualEventBus::new_zenoh(config).await?;
```

**Fallback: tokio::broadcast (extreme resource constraints)**

```rust
let (tx, _rx) = broadcast::channel(1024);
let bus = DualEventBus::new_broadcast(tx);
```

**Coexistence: Dual-mode for backward compatibility**

```rust
let (tx, _rx) = broadcast::channel(1024);
let mut config = zenoh::config::Config::default();
config.insert_json5("listen/endpoints", "[]").unwrap();
config.insert_json5("connect/endpoints", "[]").unwrap();
config.insert_json5("scouting/multicast/enabled", "false").unwrap();
config.insert_json5("scouting/gossip/enabled", "false").unwrap();

let bus = DualEventBus::new_dual(config, tx).await?;

// Events published to both Zenoh (primary) and broadcast (compatibility)
// New subscribers use Zenoh
// Legacy subscribers continue using broadcast until transitioned
```

**Distributed: Zenoh peer mode (multi-node deployment)**

```rust
let mut config = zenoh::config::Config::default();
config.set_mode(Some(zenoh::config::WhatAmI::Peer))?;
let bus = DualEventBus::new_zenoh(config).await?;
```

**Future: Zenoh storage backend (fully distributed)**

Replace SQLite event store with Zenoh's RocksDB/S3 storage backend for fully distributed event sourcing.

```rust
pub struct ZenohStorageEventStore {
    session: zenoh::Session,
}

impl ZenohStorageEventStore {
    pub async fn new(config: zenoh::config::Config) -> Result<Self, zenoh::Error> {
        let session = zenoh::open(config).res().await?;
        Ok(Self { session })
    }

    pub async fn append(&self, events: Vec<Event>) -> Result<Vec<StoredEvent>> {
        for event in events {
            let key = event_key(&event);
            let payload = serde_json::to_vec(&event)?;
            self.session.put(&key, payload).res().await?;
        }
        Ok(stored_events)
    }

    pub async fn load(&self, aggregate_id: &str) -> Result<Vec<StoredEvent>> {
        let selector = format!("events/*/{aggregate_id}/**");
        let mut replies = self.session.get(&selector).res().await?;

        let mut events = Vec::new();
        while let Ok(reply) = replies.recv_async().await {
            if let Ok(sample) = reply.sample {
                let event: StoredEvent = serde_json::from_slice(&sample.value.payload.contiguous())?;
                events.push(event);
            }
        }

        events.sort_by_key(|e| e.sequence);
        Ok(events)
    }
}
```

## Rollback procedure for dual-mode deployments

If template users running dual-mode encounter issues with Zenoh:

1. **Immediate rollback (dual mode active):**
   ```rust
   // Change mode back to Broadcast-only
   let bus = DualEventBus::new_broadcast(tx);
   ```

2. **Full rollback (Zenoh-only to broadcast):**
   ```rust
   // Redeploy with broadcast-only configuration
   let (tx, _rx) = broadcast::channel(1024);
   let bus = DualEventBus::new_broadcast(tx);
   ```

3. **Data consistency:**
   - SQLite event store remains source of truth
   - Zenoh is additive (pub/sub only, not storage)
   - No data loss risk during rollback

**Note**: Fresh ironstar instantiations use Zenoh-only by default and do not have a broadcast fallback to roll back to.

## Testing dual mode

```rust
#[tokio::test]
async fn test_dual_mode_publish() {
    let (tx, mut rx1) = broadcast::channel(16);
    let mut config = zenoh::config::Config::default();
    config.insert_json5("listen/endpoints", "[]").unwrap();
    config.insert_json5("connect/endpoints", "[]").unwrap();
    config.insert_json5("scouting/multicast/enabled", "false").unwrap();
    config.insert_json5("scouting/gossip/enabled", "false").unwrap();

    let bus = DualEventBus::new_dual(config, tx).await.unwrap();

    let event = StoredEvent { /* ... */ };
    bus.publish(&event).await.unwrap();

    // Verify broadcast received event
    let received_broadcast = rx1.recv().await.unwrap();
    assert_eq!(received_broadcast, event);

    // Verify Zenoh received event
    // (requires Zenoh subscriber setup)
}
```

## Performance characteristics

| Metric | Zenoh (embedded) | tokio::broadcast | Zenoh (distributed) |
|--------|------------------|------------------|---------------------|
| Latency (local) | ~100μs | <1μs | ~200μs |
| Throughput (single node) | ~100K/sec | ~1M/sec | ~50K/sec |
| Memory per subscriber | ~32KB | ~8KB | ~64KB |
| Max subscribers (practical) | ~10K | ~256 | ~100K |
| Network overhead | None | None | Yes (TCP/UDP) |
| Key expression filtering | Yes | No | Yes |
| Distribution-ready | Yes (config change only) | No | Yes |

**Recommendation:** Use Zenoh embedded mode (the default) unless extreme resource constraints require the broadcast fallback.

## Related documentation

- Infrastructure decisions: `../decisions/infrastructure-decisions.md`
- Analytics cache architecture: `analytics-cache-architecture.md` (Pattern 4: Zenoh invalidation)
- Session management: `session-management.md` (per-session Zenoh keys)
- SSE connection lifecycle: `../cqrs/sse-connection-lifecycle.md` (subscription patterns)
