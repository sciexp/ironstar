||| Shared Kernel: User identity types
|||
||| This module defines cross-context identity types that are part of the
||| Shared Kernel between Session and Workspace bounded contexts.
|||
||| Shared Kernel pattern: These types are owned by neither context but
||| imported by both. This enables bidirectional sharing without creating
||| unidirectional dependencies between bounded contexts.
|||
||| Types exported:
||| - OAuthProvider: Authentication provider enumeration
||| - UserId: User identity combining provider and external ID
|||
||| == UserId Evolution Strategy ==
|||
||| Current representation: Composite key (provider, externalId) for OAuth lookup.
||| This design supports the MVP OAuth-only authentication model.
|||
||| Future evolution path (no breaking changes required):
||| 1. Rust implementation uses UUID as canonical users.id
||| 2. Composite (provider, externalId) stored in user_identities table for lookup
||| 3. UserId in events references the canonical UUID
||| 4. OAuthProvider enum can extend to AuthMethod sum type:
|||    - Add Passkey, EnterpriseSso variants
|||    - Existing OAuth variants remain unchanged
|||
||| This spec defines the logical domain model. The Rust implementation
||| bridges between composite OAuth identity and canonical UUID via the
||| user_identities table schema documented in oauth-authentication.md.
|||
||| See also: docs/notes/architecture/decisions/auth-evolution-strategy.md
module SharedKernel.UserId

%default total

------------------------------------------------------------------------
-- OAuth Provider
------------------------------------------------------------------------

||| OAuth provider enumeration
||| GitHub is primary provider, Google planned as future extension
public export
data OAuthProvider = GitHub | Google

public export
Eq OAuthProvider where
  GitHub == GitHub = True
  Google == Google = True
  _ == _ = False

public export
Show OAuthProvider where
  show GitHub = "GitHub"
  show Google = "Google"

------------------------------------------------------------------------
-- User Identifier
------------------------------------------------------------------------

||| User identifier combining OAuth provider and external ID
|||
||| Shared Kernel type imported by:
||| - Session context: for session state and events
||| - Workspace context: for workspace ownership
public export
record UserId where
  constructor MkUserId
  provider : OAuthProvider
  externalId : String

public export
Eq UserId where
  (MkUserId p1 id1) == (MkUserId p2 id2) = p1 == p2 && id1 == id2

public export
Show UserId where
  show (MkUserId prov extId) = show prov ++ ":" ++ extId
