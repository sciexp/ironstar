---
title: ironstar-shared-kernel
---

# ironstar-shared-kernel

Foundation crate providing cross-context identity types for the ironstar workspace.
These types form the shared kernel in DDD terms, imported by Session, Workspace, and Analytics contexts without coupling them to each other.
See the [specification](../../spec/SharedKernel/README.md) for the logical domain model in Idris2.

## Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(transparent)]
pub struct UserId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    GitHub,
    Google,
}
```

`UserId` wraps a UUID v4 as the canonical user identity reference, serializing transparently as a UUID string.
Construction uses `UserId::new()` for fresh identities or `UserId::from_uuid(uuid)` when deserializing from storage.

`OAuthProvider` enumerates authentication providers, serializing as lowercase strings (`"github"`, `"google"`).

Both types derive `TS` for TypeScript type generation via ts-rs.

## Spec correspondence

| Spec (Idris2) | Implementation (Rust) | Notes |
|---|---|---|
| `data OAuthProvider = GitHub \| Google` | `enum OAuthProvider { GitHub, Google }` | Direct correspondence |
| `record UserId { provider : OAuthProvider, externalId : String }` | `struct UserId(Uuid)` | Divergence: implementation uses canonical UUID |

The spec defines `UserId` as a composite key `(provider, externalId)` for OAuth lookup.
The Rust implementation has advanced to the target state where `UserId` wraps a canonical UUID v4.
The composite key lookup is an infrastructure concern handled by the `user_identities` table, not the domain type.
See the [spec evolution strategy](../../spec/SharedKernel/README.md#evolution-strategy) for the rationale behind this divergence.

## Cross-links

- [Specification](../../spec/SharedKernel/README.md) for the logical domain model and evolution strategy.
- [ironstar-session](../ironstar-session/README.md) for session aggregate consuming `UserId`.
- [ironstar-workspace](../ironstar-workspace/README.md) for workspace ownership consuming `UserId`.
