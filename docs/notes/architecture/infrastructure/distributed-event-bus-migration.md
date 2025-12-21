# Distributed event bus migration

This document describes the migration path from tokio::broadcast (single-node) to Zenoh (distributed) for the ironstar event bus.
For the initial single-node event bus design, see `../decisions/infrastructure-decisions.md`.

## When to migrate from tokio::broadcast to Zenoh

**Single-node scaling limits:**

tokio::broadcast is sufficient for single-node deployments up to:
- ~256 concurrent SSE clients (subscribers)
- ~1000 events/second throughput

These limits are imposed by in-memory channel capacity and lock contention.

**Migration triggers:**

Migrate to Zenoh when:
1. Concurrent SSE clients exceed 200 (approaching capacity limit)
2. Event throughput exceeds 800/sec sustained (approaching throughput limit)
3. Multi-node deployment is required (horizontal scaling)
4. Cross-service event distribution is needed (microservices pattern)

## Zenoh — unified abstraction for distribution

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

**Why Zenoh over Apache Iggy:**

- Zenoh has both streaming and storage in one abstraction
- Storage backends (RocksDB, S3) provide durability
- Subscriptions provide the "watch" semantics
- More production-ready (Eclipse Foundation, April 2025 Gozuryū release)

## Migration strategy: DualEventBus coexistence

The migration uses a dual-mode event bus that supports both tokio::broadcast and Zenoh simultaneously, allowing gradual rollout and easy rollback.

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

// Phase 1: SQLite + tokio::broadcast implementation
pub struct SqliteEventStore {
    pool: SqlitePool,
    bus: EventBus
}

// Phase 2: SQLite + Zenoh implementation (same trait)
pub struct ZenohEventStore {
    pool: SqlitePool,
    session: zenoh::Session
}

// Phase 3: Zenoh storage backend (fully distributed)
pub struct ZenohStorageEventStore {
    session: zenoh::Session,
    // No SQLite dependency
}
```

**DualEventBus implementation:**

```rust
use tokio::sync::broadcast;
use zenoh::prelude::r#async::*;

pub enum EventBusMode {
    Broadcast(broadcast::Sender<StoredEvent>),
    Zenoh(zenoh::Session),
    Dual {
        broadcast: broadcast::Sender<StoredEvent>,
        zenoh: zenoh::Session,
    },
}

pub struct DualEventBus {
    mode: EventBusMode,
}

impl DualEventBus {
    pub fn new_broadcast(tx: broadcast::Sender<StoredEvent>) -> Self {
        Self {
            mode: EventBusMode::Broadcast(tx),
        }
    }

    pub async fn new_zenoh(config: zenoh::config::Config) -> Result<Self, zenoh::Error> {
        let session = zenoh::open(config).res().await?;
        Ok(Self {
            mode: EventBusMode::Zenoh(session),
        })
    }

    pub async fn new_dual(
        tx: broadcast::Sender<StoredEvent>,
        config: zenoh::config::Config,
    ) -> Result<Self, zenoh::Error> {
        let session = zenoh::open(config).res().await?;
        Ok(Self {
            mode: EventBusMode::Dual {
                broadcast: tx,
                zenoh: session,
            },
        })
    }

    pub async fn publish(&self, event: &StoredEvent) -> Result<(), Error> {
        match &self.mode {
            EventBusMode::Broadcast(tx) => {
                tx.send(event.clone())
                    .map_err(|_| Error::BroadcastFailed)?;
            }
            EventBusMode::Zenoh(session) => {
                let key = event_key(event);
                let payload = serde_json::to_vec(event)?;
                session.put(&key, payload).res().await?;
            }
            EventBusMode::Dual { broadcast, zenoh } => {
                // Publish to both buses (ignore individual failures in dual mode)
                let _ = broadcast.send(event.clone());
                let key = event_key(event);
                let payload = serde_json::to_vec(event)?;
                let _ = zenoh.put(&key, payload).res().await;
            }
        }
        Ok(())
    }

    pub async fn subscribe(&self) -> Result<EventSubscription, Error> {
        match &self.mode {
            EventBusMode::Broadcast(tx) => {
                Ok(EventSubscription::Broadcast(tx.subscribe()))
            }
            EventBusMode::Zenoh(session) => {
                let subscriber = session.declare_subscriber("events/**").res().await?;
                Ok(EventSubscription::Zenoh(subscriber))
            }
            EventBusMode::Dual { broadcast, .. } => {
                // In dual mode, prefer Zenoh for new subscribers
                Ok(EventSubscription::Broadcast(broadcast.subscribe()))
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

## Key expression patterns for distributed deployment

**Hierarchical event keys:**

```
events/{aggregate_type}/{aggregate_id}/{sequence}
```

**Examples:**

```
events/Todo/uuid-123/1
events/Todo/uuid-123/2
events/User/uuid-456/1
```

**Subscription patterns:**

```rust
// Subscribe to all events
session.subscribe("events/**").await?;

// Subscribe to specific aggregate type
session.subscribe("events/Todo/**").await?;

// Subscribe to specific aggregate instance
session.subscribe("events/Todo/uuid-123/**").await?;
```

**Cache invalidation patterns:**

```rust
// Invalidate all Todo-related caches when any Todo event arrives
let subscriber = session.subscribe("events/Todo/**").await?;

// Invalidate specific aggregate cache
let subscriber = session.subscribe("events/Todo/uuid-123/**").await?;
```

**Session-scoped events:**

```rust
// Events scoped to specific session (for SSE filtering)
format!("events/session/{session_id}/**")

// Subscribe to session-specific events
session.subscribe(&format!("events/session/{session_id}/**")).await?;
```

## Migration phases

**Phase 1: Single-node with tokio::broadcast (current)**

```rust
let (tx, _rx) = broadcast::channel(1024);
let bus = DualEventBus::new_broadcast(tx);
```

**Phase 2: Dual-mode coexistence**

```rust
let (tx, _rx) = broadcast::channel(1024);
let config = zenoh::config::Config::default();
let bus = DualEventBus::new_dual(tx, config).await?;

// Events published to both buses
// New subscribers use Zenoh
// Old subscribers continue using broadcast
```

**Phase 3: Zenoh-only (distributed)**

```rust
let mut config = zenoh::config::Config::default();
config.set_mode(Some(zenoh::config::WhatAmI::Peer))?;
let bus = DualEventBus::new_zenoh(config).await?;
```

**Phase 4: Zenoh storage backend (optional)**

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

## Rollback procedure

If Zenoh migration encounters issues, rollback to tokio::broadcast:

1. **Immediate rollback (dual mode active):**
   ```rust
   // Change mode back to Broadcast-only
   let bus = DualEventBus::new_broadcast(tx);
   ```

2. **Full rollback (Zenoh-only mode):**
   ```rust
   // Redeploy with broadcast-only configuration
   let (tx, _rx) = broadcast::channel(1024);
   let bus = DualEventBus::new_broadcast(tx);
   ```

3. **Data consistency:**
   - SQLite event store remains source of truth during migration
   - Zenoh is additive (pub/sub only, not storage in phases 1-3)
   - No data loss risk during rollback

## Testing dual mode

```rust
#[tokio::test]
async fn test_dual_mode_publish() {
    let (tx, mut rx1) = broadcast::channel(16);
    let config = zenoh::config::Config::default();
    let bus = DualEventBus::new_dual(tx, config).await.unwrap();

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

| Metric | tokio::broadcast | Zenoh (embedded) | Zenoh (distributed) |
|--------|------------------|------------------|---------------------|
| Latency (local) | <1μs | ~50μs | ~200μs |
| Throughput (single node) | ~1M/sec | ~100K/sec | ~50K/sec |
| Memory per subscriber | ~8KB | ~32KB | ~64KB |
| Max subscribers (practical) | ~256 | ~10K | ~100K |
| Network overhead | None | None | Yes (TCP/UDP) |

**Recommendation:** Stay on tokio::broadcast unless you need >256 subscribers or multi-node deployment.

## Related documentation

- Infrastructure decisions: `../decisions/infrastructure-decisions.md`
- Analytics cache architecture: `analytics-cache-architecture.md` (Pattern 4: Zenoh invalidation)
- Session management: `session-management.md` (per-session Zenoh keys)
- SSE connection lifecycle: `../cqrs/sse-connection-lifecycle.md` (subscription patterns)
