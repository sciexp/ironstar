# Authentication and authorization evolution strategy

## Status

Accepted

## Context

Ironstar's MVP uses GitHub OAuth with SQLite session storage.
The architecture must support future evolution to:

- Multiple OAuth/OIDC providers (Google, Enterprise SSO)
- WebAuthn/Passkey authentication
- Cedar-based ABAC authorization
- Multi-tenancy (if required)

This ADR documents the evolution path ensuring current decisions do not create irreversible constraints.
The design preserves flexibility by separating concerns at type boundaries rather than embedding assumptions about authentication methods or authorization models into core domain types.

## Decision

### UserId representation

The `UserId` type in the Shared Kernel uses a dual representation strategy:

**Composite form for lookup:**

```idris
record UserId where
  constructor MkUserId
  provider : OAuthProvider
  externalId : String
```

The composite form `(provider, externalId)` enables efficient lookup when a user authenticates via OAuth.
The `user_identities` table indexes on `(provider, provider_user_id)` for this purpose.

**UUID for canonical identity:**

The `users` table uses a UUID primary key that serves as the canonical, provider-independent user identifier:

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,  -- UUID
    ...
);

CREATE TABLE user_identities (
    user_id TEXT NOT NULL REFERENCES users(id),
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    UNIQUE(provider, provider_user_id)
);
```

This separation means:

- OAuth lookups use `(provider, provider_user_id)` to find existing identities
- All other domain operations reference `users.id` (UUID)
- Multiple providers can link to a single user record
- Provider changes do not cascade through the domain

**Evolution path:**

When WebAuthn support is added, the `user_identities` pattern extends naturally.
Passkey credentials link to the same `users.id` without modifying the core `UserId` abstraction that flows through the domain.

### OAuthProvider to AuthMethod evolution

The current spec defines a closed enum:

```idris
data OAuthProvider = GitHub | Google
```

This closed form is intentional for the MVP, enforcing exhaustive pattern matching in session handling code.
The schema, however, uses a string column:

```sql
provider TEXT NOT NULL,  -- 'github' | 'google' | 'passkey' | 'saml:okta'
```

**Evolution strategy:**

The Idris spec evolves from closed to open representation:

```idris
-- MVP (closed, exhaustive matching)
data OAuthProvider = GitHub | Google

-- Future (open, extensible)
data AuthMethod
  = OAuth OAuthProvider
  | Passkey PasskeyCredentialId
  | EnterpriseSso SsoProviderId
```

This evolution is source-compatible at the database layer because the schema already uses string-based provider identification.
Only the Rust types need updating, and existing events deserialize correctly via upcasters that map `"github"` to `OAuth GitHub`.

**Key insight:**

The schema is additive-ready.
New authentication methods require:

1. New variant in `AuthMethod` sum type
2. Upcaster for existing events
3. New handler in boundary layer

No schema migrations, no aggregate redesign.

### Session aggregate boundaries

The Session aggregate handles lifecycle management only:

- `CreateSession`: Establish authenticated session after any auth method succeeds
- `RefreshSession`: Extend TTL on activity
- `InvalidateSession`: Explicit logout
- `SessionExpired`: TTL expiration (boundary-generated event)

**What Session does not handle:**

- OAuth token exchange (boundary layer effect)
- WebAuthn ceremony orchestration (separate aggregate)
- Credential validation (boundary layer effect)

**WebAuthn/Passkey architecture:**

When WebAuthn support is added, a separate `PasskeyCredential` aggregate manages:

```idris
data PasskeyCredentialCommand
  = RegisterCredential UserId CredentialCreationOptions
  | AuthenticateCredential CredentialId AuthenticatorResponse
  | RevokeCredential CredentialId

data PasskeyCredentialEvent
  = CredentialRegistered CredentialId UserId PublicKey Timestamp
  | CredentialUsed CredentialId Timestamp SignatureCount
  | CredentialRevoked CredentialId Timestamp
```

**Rationale:**

Combining Session and PasskeyCredential would violate SRP.
WebAuthn ceremonies involve complex state machines (challenge generation, attestation validation, authenticator data parsing) that have no conceptual relationship to session TTL management.
Isolating ceremony complexity prevents it from leaking into session handling code.

**Cross-aggregate flow:**

```
WebAuthn Authentication:
  Browser → AuthenticateCredential command
         → PasskeyCredential aggregate validates
         → CredentialUsed event
         → Boundary layer creates Session
         → SessionCreated event
```

The boundary layer orchestrates the handoff: successful credential authentication triggers session creation as a separate command.

### RBAC to Cedar ABAC migration

The MVP uses simple RBAC:

```rust
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

impl Authorizer for RbacAuthorizer {
    fn can(&self, user: &User, resource: &str, action: &str) -> bool {
        match action {
            "read" => user.has_role(Role::Viewer) || ...,
            "write" => user.has_role(Role::Editor) || ...,
            ...
        }
    }
}
```

**Cedar migration path:**

RBAC is a strict subset of ABAC.
Every RBAC rule maps to a Cedar policy:

```cedar
// RBAC: Admin can delete
permit(
  principal in Role::"Admin",
  action == Action::"delete",
  resource
);

// RBAC: Editor can write
permit(
  principal in Role::"Editor",
  action == Action::"write",
  resource
);

// ABAC extension: Owner can write their own resources
permit(
  principal,
  action == Action::"write",
  resource
) when { resource.owner == principal };
```

**Migration strategy:**

1. Keep `Role` enum and `has_role()` predicates
2. Add Cedar policy engine alongside RBAC
3. Migrate authorization checks incrementally
4. RBAC remains available during transition
5. Eventually deprecate simple RBAC if Cedar covers all cases

**User roles map to Cedar entities:**

```rust
// Current RBAC role
user.has_role(Role::Editor)

// Cedar entity attribute
cedar::Entity::with_uid("User", user.id)
    .with_parent("Role", "Editor")
```

The migration is additive.
Existing RBAC code continues working; Cedar policies layer on top for more sophisticated rules.

### SessionMetadata for audit trail

The `SessionCreated` event captures security-relevant metadata:

```idris
data SessionEvent
  = SessionCreated SessionId UserId OAuthProvider Timestamp ExpiresAt
  | ...
```

**Extended metadata (boundary-populated):**

```rust
pub struct SessionMetadata {
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub geo_location: Option<GeoLocation>,
    pub device_fingerprint: Option<String>,
}

// Stored in event payload
SessionCreated {
    session_id: SessionId,
    user_id: UserId,
    auth_method: AuthMethod,
    timestamp: Timestamp,
    expires_at: ExpiresAt,
    metadata: SessionMetadata,
}
```

**Why at boundary layer:**

The Idris spec remains pure: no IP addresses, no HTTP headers.
The Rust boundary layer extracts metadata from axum extractors:

```rust
async fn create_session_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ...
) -> Result<...> {
    let metadata = SessionMetadata {
        ip_address: Some(addr.ip()),
        user_agent: headers.get("user-agent").map(|h| h.to_str().ok()),
        ...
    };
    // Pass to session creation
}
```

**Use cases enabled:**

- Security audit: track session origins
- Device tracking: detect concurrent sessions from different devices
- Suspicious activity: flag sessions from unusual locations
- Compliance: GDPR access logs, SOC2 audit trails

This metadata is optional (wrapped in `Option`) to preserve flexibility and avoid requiring geo-location infrastructure in minimal deployments.

## Consequences

### Positive

**No architectural rewrites needed for future auth features.**
The separation of `users` (canonical identity) from `user_identities` (provider links) means adding WebAuthn requires only a new identity type, not a user schema migration.

**Schema is additive-ready.**
String-based provider columns, JSON metadata in events, and the `user_identities` pattern all support extension without breaking changes.

**Clear aggregate boundaries prevent ceremony complexity leaking into Session.**
WebAuthn's challenge-response flow, attestation validation, and credential management remain isolated.
Session handling stays simple: create, refresh, invalidate.

**RBAC to Cedar migration is incremental.**
Authorization can evolve without big-bang rewrites.
Simple deployments keep RBAC; complex deployments add Cedar policies.

### Negative

**Must maintain dual UserId representation understanding.**
Developers need to know when to use `(provider, externalId)` (OAuth lookup) versus UUID (domain operations).
Documentation and type signatures mitigate this, but it adds conceptual overhead.

**PasskeyCredential aggregate adds complexity when WebAuthn implemented.**
The additional aggregate means more events, more projections, more tests.
This is acceptable complexity for the feature's value, but it is not free.

**SessionMetadata extraction depends on deployment environment.**
IP addresses may be proxied (need `X-Forwarded-For` handling), geo-location requires external service or database.
Not all metadata fields will be populated in all deployments.

### Neutral

**Cedar policies can coexist with simple RBAC during transition.**
Neither approach invalidates the other.
Teams can migrate at their own pace, using RBAC for simple checks and Cedar for complex ones.

**Event schema evolution uses upcasters.**
Adding `metadata` field to `SessionCreated` requires an upcaster for historical events.
This is standard event sourcing practice, neither positive nor negative.

## References

- `spec/SharedKernel/UserId.idr` — Shared Kernel identity types
- `spec/Session/Session.idr` — Session aggregate specification
- `docs/notes/architecture/decisions/oauth-authentication.md` — OAuth flow and user storage schema
- `docs/notes/architecture/infrastructure/session-implementation.md` — Session service implementation patterns
- [Cedar language documentation](https://www.cedarpolicy.com/en/tutorial/overview) — ABAC policy language
- [WebAuthn specification](https://www.w3.org/TR/webauthn-2/) — Passkey authentication standard
