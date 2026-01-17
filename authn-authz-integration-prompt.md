# AuthN/AuthZ Integration Review & Roadmap Task

## Context

You are reviewing the ironstar project to integrate a carefully designed authentication and authorization stack into our existing event-sourced, type-driven architecture. This task requires multi-phase analysis across several interconnected artifacts.

**Read CLAUDE.md first** for project conventions, then proceed with the phased approach below.

## Phase 1: Background Research (Explore/Task Agents)

Before integration analysis, gather comprehensive context from each source:

### 1.1 Idris Type Specifications
```bash
# Explore the spec/ directory structure and key type definitions
fd -e idr . spec/ | head -50
# Look for existing Session, Identity, User, Principal, or Auth-related types
rg -l "Session|Identity|User|Principal|Auth|Credential" spec/
# Examine the core domain model types
bat spec/Domain/*.idr 2>/dev/null || cat spec/Domain/*.idr
```

### 1.2 Qlerify Event Model Analysis
```bash
# Locate qlerify output files
fd -e json . -p qlerify
# Use jaq to explore event structure
jaq -r 'keys' path/to/qlerify/output.json
# Extract all event names
jaq -r '.. | .events? // empty | .[].name' path/to/qlerify/*.json
# Use DuckDB to analyze event relationships
duckdb -c "
  SELECT DISTINCT 
    json_extract_string(event, '$.aggregate') as aggregate,
    json_extract_string(event, '$.name') as event_name,
    json_extract_string(event, '$.command') as triggering_command
  FROM read_json_auto('path/to/qlerify/*.json', union_by_name=true)
  WHERE json_extract_string(event, '$.aggregate') ILIKE '%user%' 
     OR json_extract_string(event, '$.aggregate') ILIKE '%session%'
     OR json_extract_string(event, '$.aggregate') ILIKE '%auth%'
"
```

### 1.3 Event Catalog Documentation
```bash
# Find event catalog files
fd -e md -e yaml . docs/event-catalog 2>/dev/null || fd -e md -e yaml . eventcatalog
# List documented domains/services
ls -la docs/event-catalog/domains/ 2>/dev/null
# Check for existing auth-related documentation
rg -l -i "auth|session|identity|credential" docs/
```

### 1.4 Architecture Documentation
```bash
# Review architecture docs structure
tree docs/notes/architecture/ 2>/dev/null || ls -laR docs/notes/architecture/
# Find auth-related architecture decisions
rg -l -i "auth|session|identity|oidc|webauthn|passkey|cedar|policy" docs/notes/architecture/
# Check for ADRs (Architecture Decision Records)
fd -e md . docs/notes/architecture/ -x head -30 {}
```

### 1.5 Beads Epics and Issues
```bash
# If using beads CLI or local tracking
fd -e md . .beads/ 2>/dev/null || fd -e yaml . .beads/
# Or query the beads API/files for auth-related items
rg -i "auth|session|identity|login|sso|passkey" .beads/ 2>/dev/null
# List all epics
cat .beads/epics/*.md 2>/dev/null | head -100
```

## Phase 2: Integration Analysis

After gathering context, analyze the following relationships:

### 2.1 Type System Alignment Matrix

Create a mapping table:
| Concept | Idris spec/ | Qlerify Events | Event Catalog | Architecture Docs | Beads |
|---------|-------------|----------------|---------------|-------------------|-------|
| User/Identity | ? | ? | ? | ? | ? |
| Session | ? | ? | ? | ? | ? |
| Credential | ? | ? | ? | ? | ? |
| AuthMethod | ? | ? | ? | ? | ? |
| Permission/Policy | ? | ? | ? | ? | ? |
| Tenant/Org | ? | ? | ? | ? | ? |

### 2.2 Consistency Checks

Verify:
- [ ] Event names in qlerify match Idris type constructors
- [ ] Event catalog documents all events from qlerify
- [ ] Architecture docs reference correct type names
- [ ] Beads epics/issues align with documented scope
- [ ] No orphaned or undocumented concepts

### 2.3 Gap Analysis

Identify what's missing for the AuthN/AuthZ stack below:

---
## AuthN/AuthZ Stack Specification

The following conversation documents the complete authentication and authorization stack we've designed for ironstar. Review this specification against existing artifacts:

---

## Complete AuthN + AuthZ Dependency Stack

```toml
[dependencies]
# ═══════════════════════════════════════════════════════════════════════════════
# WEB FRAMEWORK
# ═══════════════════════════════════════════════════════════════════════════════
axum = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.10", features = ["cookie-private", "typed-header"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# ═══════════════════════════════════════════════════════════════════════════════
# AUTHENTICATION
# ═══════════════════════════════════════════════════════════════════════════════
webauthn-rs = { version = "0.5", features = ["danger-allow-state-serialisation"] }
webauthn-rs-proto = "0.5"
openidconnect = "4"

# ═══════════════════════════════════════════════════════════════════════════════
# SESSION MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════════════
tower-sessions = "0.14"
tower-sessions-sqlx-store = { version = "0.14", features = ["postgres"] }

# ═══════════════════════════════════════════════════════════════════════════════
# AUTHORIZATION
# ═══════════════════════════════════════════════════════════════════════════════
cedar-policy = "4"

# ═══════════════════════════════════════════════════════════════════════════════
# PERSISTENCE
# ═══════════════════════════════════════════════════════════════════════════════
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }

# ═══════════════════════════════════════════════════════════════════════════════
# DOMAIN MODELING & TYPE SAFETY
# ═══════════════════════════════════════════════════════════════════════════════
thiserror = "2"
derive_more = { version = "1", features = ["from", "into", "display", "error"] }

# ═══════════════════════════════════════════════════════════════════════════════
# SERIALIZATION
# ═══════════════════════════════════════════════════════════════════════════════
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# ═══════════════════════════════════════════════════════════════════════════════
# PRIMITIVES
# ═══════════════════════════════════════════════════════════════════════════════
uuid = { version = "1", features = ["v4", "v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
url = "2"

# ═══════════════════════════════════════════════════════════════════════════════
# ASYNC RUNTIME & OBSERVABILITY
# ═══════════════════════════════════════════════════════════════════════════════
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

## Dependency Role Matrix

| Crate | Concern | Role | FP/Domain Modeling Alignment |
|-------|---------|------|------------------------------|
| **axum** | Web | Request routing, extractors, middleware composition | Tower's `Service` trait is a pure function $f: \text{Request} \to \text{Future}[\text{Response}]$; composable via combinators |
| **axum-extra** | Web | Private cookies, typed headers | Typed extractors enforce parse-don't-validate; `TypedHeader<T>` lifts raw bytes into domain types |
| **tower** | Web | Middleware as composable service transformers | Middleware is function composition: $\text{Layer}: \text{Service} \to \text{Service}$; stacks compose algebraically |
| **tower-http** | Web | CORS, tracing middleware | Cross-cutting concerns as pure transformations over the service stack |
| **webauthn-rs** | AuthN | WebAuthn/Passkey ceremonies | Ceremony states are explicit ADTs (`PasskeyRegistration`, `PasskeyAuthentication`); state machine encoded in types |
| **webauthn-rs-proto** | AuthN | WebAuthn protocol types | Protocol messages as sum/product types; serialization derived, not hand-rolled |
| **openidconnect** | AuthN | OIDC client for GitHub, Google, Enterprise SSO | Discovery, token exchange, claims as typed structs; `StandardClaims` is a product type |
| **tower-sessions** | Session | Session middleware | Session as `Option<T>` semantics; presence/absence explicit in types |
| **tower-sessions-sqlx-store** | Session | Postgres session persistence | Storage as effect; session operations return `Result<T, E>` |
| **cedar-policy** | AuthZ | Policy evaluation engine | Authorization as pure function: $(\text{Principal}, \text{Action}, \text{Resource}, \text{Context}) \to \text{Decision}$ |
| **sqlx** | Persistence | Async Postgres with compile-time query checking | Queries validated at compile time; row mapping via `FromRow` derive |
| **thiserror** | Errors | Structured error types | Errors as sum types (enums); `#[from]` for automatic lifting |
| **derive_more** | Types | Newtype derivations | Newtypes for domain primitives (`UserId`, `Email`, `TenantId`); enforces type distinctions |
| **serde** | Serialization | Derive-based (de)serialization | Serialization as typeclass/trait; `Serialize`/`Deserialize` are coherent, derivable |
| **serde_json** | Serialization | JSON codec | JSON as interchange format; Cedar context, OIDC claims |
| **uuid** | Primitives | Unique identifiers | `Uuid` as opaque identifier type; v7 for time-ordered, v4 for random |
| **chrono** | Primitives | Temporal types | `DateTime<Utc>` is a proper time instant; no stringly-typed timestamps |
| **url** | Primitives | URL parsing and validation | `Url` type enforces well-formedness; redirect URIs, issuer URLs |
| **tokio** | Runtime | Async executor | Effect interpretation; async as suspended computation |
| **tracing** | Observability | Structured logging, spans | Logging as effect; spans compose hierarchically |

## Domain Modeling Patterns by Concern

### AuthN Domain Types

```rust
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// NEWTYPES: Distinct types prevent mixing IDs
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct TenantId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct CredentialId(Uuid);

// ─────────────────────────────────────────────────────────────────────────────
// VALIDATED STRINGS: Parse, don't validate
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn parse(s: impl Into<String>) -> Result<Self, DomainError> {
        let s = s.into();
        // Minimal validation; OIDC provider is source of truth
        if s.contains('@') && s.len() > 3 {
            Ok(Self(s))
        } else {
            Err(DomainError::InvalidEmail(s))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SUM TYPES: Authentication methods are mutually exclusive
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    Passkey { credential_id: CredentialId },
    GitHub { provider_user_id: String },
    Google { provider_user_id: String },
    EnterpriseSso { issuer: url::Url, provider_user_id: String },
}

impl AuthMethod {
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Passkey { .. } => "passkey",
            Self::GitHub { .. } => "github",
            Self::Google { .. } => "google",
            Self::EnterpriseSso { .. } => "enterprise_sso",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PRODUCT TYPES: User aggregate
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub display_name: Option<String>,
    pub tenant_id: TenantId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // SCIM-ready fields
    pub scim_external_id: Option<String>,
    pub scim_provisioned: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// STATE MACHINES: WebAuthn ceremonies as explicit states
// ─────────────────────────────────────────────────────────────────────────────

/// Registration ceremony states
pub enum RegistrationState {
    NotStarted,
    ChallengeIssued { 
        state: webauthn_rs::prelude::PasskeyRegistration,
        user_id: UserId,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    Completed { credential: PasskeyCredential },
    Failed { reason: RegistrationError },
}

/// Authentication ceremony states  
pub enum AuthenticationState {
    NotStarted,
    ChallengeIssued {
        state: webauthn_rs::prelude::PasskeyAuthentication,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    Completed { user: User, credential_used: CredentialId },
    Failed { reason: AuthenticationError },
}
```

### AuthZ Domain Types

```rust
use cedar_policy::{Entity, EntityUid, RestrictedExpression};
use std::collections::HashSet;

// ─────────────────────────────────────────────────────────────────────────────
// ACTIONS: Closed set of operations
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Read,
    Write,
    Delete,
    ManageMembers,
    ManageSettings,
}

impl Action {
    pub fn as_cedar_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::ManageMembers => "manage_members",
            Self::ManageSettings => "manage_settings",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DECISION: Authorization result with diagnostics
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AuthzDecision {
    Allow { 
        determining_policies: Vec<String>,
    },
    Deny { 
        determining_policies: Vec<String>,
        reasons: Vec<String>,
    },
}

impl AuthzDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RESOURCE TRAIT: Domain entities convertible to Cedar entities
// ─────────────────────────────────────────────────────────────────────────────

pub trait AuthzResource {
    fn entity_type() -> &'static str;
    fn entity_id(&self) -> String;
    fn entity_attrs(&self) -> Vec<(String, RestrictedExpression)>;
    fn entity_parents(&self) -> HashSet<EntityUid>;
    
    fn to_cedar_entity(&self) -> Entity {
        let uid = EntityUid::from_type_name_and_id(
            Self::entity_type().parse().unwrap(),
            self.entity_id().parse().unwrap(),
        );
        Entity::new(
            uid,
            self.entity_attrs().into_iter().collect(),
            self.entity_parents(),
        ).unwrap()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// REQUEST CONTEXT: Ambient attributes for policy evaluation
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AuthzContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub request_id: String,
}

impl Default for AuthzContext {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            ip_address: None,
            user_agent: None,
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
```

### Error Types (Sum Types)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthnError {
    // OIDC errors
    #[error("OIDC discovery failed: {0}")]
    OidcDiscovery(#[from] openidconnect::DiscoveryError<openidconnect::reqwest::Error<reqwest::Error>>),
    
    #[error("OIDC token exchange failed: {0}")]
    OidcTokenExchange(String),
    
    #[error("Invalid OIDC state")]
    InvalidOidcState,
    
    // WebAuthn errors
    #[error("WebAuthn error: {0}")]
    WebAuthn(#[from] webauthn_rs::prelude::WebauthnError),
    
    #[error("Passkey registration expired")]
    RegistrationExpired,
    
    #[error("Passkey authentication expired")]
    AuthenticationExpired,
    
    // Session errors
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Session expired")]
    SessionExpired,
    
    // User errors
    #[error("User not found")]
    UserNotFound,
    
    #[error("User not provisioned via SCIM")]
    UserNotProvisioned,
    
    // Infrastructure
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum AuthzError {
    #[error("Not authenticated")]
    NotAuthenticated,
    
    #[error("Access denied")]
    Forbidden,
    
    #[error("Policy validation failed: {0}")]
    PolicyValidation(String),
    
    #[error("Entity construction failed: {0}")]
    EntityConstruction(String),
    
    #[error("Cedar evaluation error: {0}")]
    Evaluation(String),
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    
    #[error("Invalid tenant ID")]
    InvalidTenantId,
}
```

### Pure Functions for Authorization

```rust
/// Authorization as a pure function
/// 
/// $\text{authorize}: \text{Principal} \times \text{Action} \times \text{Resource} \times \text{Context} \to \text{Decision}$
pub fn authorize(
    authorizer: &cedar_policy::Authorizer,
    policies: &cedar_policy::PolicySet,
    schema: &cedar_policy::Schema,
    principal: &User,
    action: Action,
    resource: &impl AuthzResource,
    context: AuthzContext,
) -> Result<AuthzDecision, AuthzError> {
    let principal_entity = principal.to_cedar_entity();
    let resource_entity = resource.to_cedar_entity();
    
    let entities = cedar_policy::Entities::from_entities(
        [principal_entity, resource_entity],
        Some(schema),
    ).map_err(|e| AuthzError::EntityConstruction(e.to_string()))?;
    
    let request = cedar_policy::Request::new(
        principal.cedar_uid(),
        action.cedar_uid(),
        resource.cedar_uid(),
        cedar_policy::Context::from_json_value(
            serde_json::to_value(&context).unwrap(),
            None,
        ).unwrap(),
        Some(schema),
    ).map_err(|e| AuthzError::Evaluation(e.to_string()))?;
    
    let response = authorizer.is_authorized(&request, policies, &entities);
    
    Ok(match response.decision() {
        cedar_policy::Decision::Allow => AuthzDecision::Allow {
            determining_policies: response.diagnostics()
                .reason()
                .map(|p| p.to_string())
                .collect(),
        },
        cedar_policy::Decision::Deny => AuthzDecision::Deny {
            determining_policies: response.diagnostics()
                .reason()
                .map(|p| p.to_string())
                .collect(),
            reasons: response.diagnostics()
                .errors()
                .map(|e| e.to_string())
                .collect(),
        },
    })
}
```

## Summary Table (Compact)

| Layer | Crate | One-Line Role |
|-------|-------|---------------|
| **Web** | `axum` | Composable HTTP service via Tower |
| | `axum-extra` | Typed cookies, typed headers |
| | `tower` | Middleware as service transformers |
| | `tower-http` | CORS, tracing layers |
| **AuthN** | `webauthn-rs` | Passkey registration/authentication |
| | `webauthn-rs-proto` | WebAuthn protocol types |
| | `openidconnect` | OIDC client (GitHub, Google, Enterprise) |
| **Session** | `tower-sessions` | Session middleware |
| | `tower-sessions-sqlx-store` | Postgres session store |
| **AuthZ** | `cedar-policy` | ABAC/RBAC policy evaluation |
| **Persistence** | `sqlx` | Compile-time checked SQL |
| **Types** | `thiserror` | Error sum types |
| | `derive_more` | Newtype derivations |
| | `serde` | Serialization typeclass |
| | `uuid` | Identity primitives |
| | `chrono` | Temporal primitives |
| | `url` | URL type |
| **Runtime** | `tokio` | Async effect interpreter |
| | `tracing` | Structured observability |

This stack gives you type-safe, composable authentication and authorization where:
- Identities are newtypes, not raw `Uuid`s
- States are explicit ADTs, not booleans
- Errors are sum types with `#[from]` lifting
- Authorization is a pure function from $(P, A, R, C) \to D$
- Effects (DB, session, network) are pushed to the edges

---

## Phase 3: Synthesis & Recommendations

Based on your analysis of the existing artifacts against the AuthN/AuthZ stack above, produce:

### 3.1 Completeness Report

```markdown
## Completeness Assessment

### Already Covered
- [ ] List concepts/events/types that already exist and align

### Partially Covered (Needs Extension)
- [ ] List items that exist but need updates to support new auth model

### Not Covered (New Additions Required)
- [ ] List entirely new concepts needed
```

### 3.2 Consistency Findings

```markdown
## Consistency Issues

### Naming Conflicts
- Existing: `Foo` vs Proposed: `Bar` — Resolution: ...

### Semantic Mismatches  
- Existing Session model assumes X, but new model requires Y

### Event Flow Gaps
- Missing events for: ...
- Missing commands for: ...
```

### 3.3 Idris Spec Updates Required

Propose specific additions/modifications to `spec/`:
- New types for `AuthMethod`, `PasskeyCredential`, `FederatedIdentity`
- Integration with existing `Session` type
- Cedar policy types if representing policies in Idris

### 3.4 Event Catalog Updates Required

List new event catalog entries needed:
- Domain: `Identity` or `Auth`
- Events: `UserRegistered`, `PasskeyRegistered`, `FederatedIdentityLinked`, `SessionEstablished`, etc.
- Commands: `RegisterPasskey`, `AuthenticateWithPasskey`, `InitiateOidcFlow`, etc.

### 3.5 Architecture Doc Updates Required

Specify updates to `docs/notes/architecture/`:
- New ADR for AuthN stack selection (WebAuthn + OIDC, no passwords)
- New ADR for AuthZ stack selection (Cedar)
- Integration doc showing auth flow through event-sourced system
- Update existing Session documentation

### 3.6 Beads Epic/Issue Creation

Draft new Beads items:

```yaml
# Epic: Authentication System Implementation
epic:
  title: "Implement WebAuthn + OIDC Authentication"
  description: |
    Implement passwordless authentication using WebAuthn (passkeys) 
    and federated identity via OIDC (GitHub, Google, Enterprise SSO)
  issues:
    - title: "Add Idris types for AuthMethod, PasskeyCredential, FederatedIdentity"
      labels: [types, spec]
    - title: "Implement webauthn-rs integration for passkey ceremonies"  
      labels: [authn, implementation]
    - title: "Implement openidconnect integration for social/enterprise SSO"
      labels: [authn, implementation]
    - title: "Update Session aggregate to track authentication method"
      labels: [domain, events]
    # ... additional issues

# Epic: Authorization System Implementation  
epic:
  title: "Implement Cedar-based ABAC Authorization"
  description: |
    Implement attribute-based access control using Cedar policy engine
  issues:
    - title: "Define Cedar schema for ironstar domain"
      labels: [authz, policy]
    - title: "Implement AuthzResource trait for domain entities"
      labels: [authz, implementation]
    # ... additional issues
```

### 3.7 Dependency Integration Plan


## Integration Roadmap

### Phase A: Foundation (Types & Events)
1. Update Idris spec with new auth types
2. Add events to qlerify model
3. Regenerate/update event catalog

### Phase B: AuthN Implementation  
1. Add dependencies to Cargo.toml
2. Implement domain types (newtypes, ADTs)
3. Implement WebAuthn ceremony handlers
4. Implement OIDC flows
5. Integrate with existing Session management

### Phase C: AuthZ Implementation
1. Define Cedar schema
2. Implement entity conversions
3. Create base policies
4. Add authorization middleware

### Phase D: Documentation & Testing
1. Update architecture docs
2. Add integration tests
3. Update API documentation
```

## Output Format

Provide your analysis as a structured document with:

1. **Executive Summary** — Key findings in 3-5 bullets
2. **Detailed Analysis** — Per-artifact findings from Phase 1-2
3. **Recommendations** — Specific, actionable items from Phase 3
4. **Proposed File Changes** — Concrete diffs or new file contents where appropriate
5. **Suggested Task Sequence** — Ordered list for implementation

## Constraints

- Preserve existing event-sourcing patterns
- Maintain consistency with Idris spec naming conventions
- Ensure all new events follow existing aggregate patterns
- Do not propose password-based authentication
- Cedar policies must align with Idris type structure
- All new types should use newtypes, not raw primitives
