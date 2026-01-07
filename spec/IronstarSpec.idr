||| Ironstar specification integration module
|||
||| This is the top-level module that re-exports all bounded context
||| specifications and core abstractions for the ironstar template.
|||
||| The module serves two purposes:
||| 1. Single import point for external consumers needing full spec access
||| 2. Type-checking verification that all modules compose correctly
|||
||| Bounded contexts:
||| - Analytics (Core domain): DuckLake catalogs, query sessions, chart specs
||| - Session (Supporting): OAuth authentication, session lifecycle
||| - Workspace (Supporting): Dashboards, saved queries, user preferences
||| - Todo (Generic Example): Canonical CQRS/ES demonstration
|||
||| Core abstractions:
||| - Decider: Pure command handling (decide, evolve, initialState)
||| - View: Event projection to read models
||| - Saga: Event-to-command choreography
||| - Effect: Repository and notification interfaces
||| - Event: Common event infrastructure (Timestamp, SessionId, etc.)
|||
||| Usage from external code:
||| ```idris
||| import IronstarSpec
||| ```
|||
||| All public exports from sub-modules are re-exported here.
module IronstarSpec

-- Core abstractions (functional event sourcing patterns)
import public Core.Decider
import public Core.View
import public Core.Saga
import public Core.Effect
import public Core.Event

-- Analytics bounded context (Core domain)
import public Analytics.Analytics

-- Session bounded context (Supporting domain)
import public Session.Session

-- Workspace bounded context (Supporting domain)
import public Workspace.Workspace

-- Todo bounded context (Generic Example domain)
import public Todo.Todo

%default total
