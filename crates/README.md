# Workspace dependency DAG

The ironstar Cargo workspace contains 11 crates organized into four role-based layers.
Edges in the diagram represent workspace-internal path dependencies only.

```mermaid
flowchart TD
    subgraph Foundation["Foundation"]
        core["ironstar-core\n(domain traits, error types)"]
        shared["ironstar-shared-kernel\n(cross-domain value objects)"]
    end

    subgraph Domain["Domain"]
        todo["ironstar-todo\n(Todo aggregate)"]
        session["ironstar-session\n(Session aggregate)"]
        analytics["ironstar-analytics\n(Catalog, QuerySession)"]
        workspace["ironstar-workspace\n(5 aggregates)"]
    end

    subgraph Infrastructure["Infrastructure"]
        evstore["ironstar-event-store\n(SqliteEventRepository)"]
        evbus["ironstar-event-bus\n(ZenohEventBus)"]
        ainfra["ironstar-analytics-infra\n(DuckDB, moka cache)"]
        sstore["ironstar-session-store\n(SqliteSessionStore)"]
    end

    subgraph Binary["Binary (composition root)"]
        app["ironstar\n(axum routes, handlers, SSE)"]
    end

    %% Domain crate dependencies
    todo --> core
    session --> core
    session --> shared
    analytics --> core
    workspace --> core
    workspace --> shared
    workspace --> analytics

    %% Infrastructure crate dependencies
    evstore --> core
    evbus --> core
    ainfra --> core
    ainfra --> evbus
    sstore --> core

    %% Binary crate dependencies
    app --> core
    app --> shared
    app --> todo
    app --> session
    app --> analytics
    app --> workspace
    app --> evstore
    app --> evbus
    app --> ainfra
    app --> sstore

    %% Layer styling
    style Foundation fill:#e8f4f8,stroke:#2980b9
    style Domain fill:#fef9e7,stroke:#f39c12
    style Infrastructure fill:#f5eef8,stroke:#8e44ad
    style Binary fill:#eafaf1,stroke:#27ae60
```

## Notes

The two foundation crates are independent of each other: `ironstar-core` provides domain traits and error types while `ironstar-shared-kernel` provides cross-domain value objects like `UserId` and `OAuthProvider`.

Among the domain crates, `ironstar-workspace` is unique in depending on another domain crate (`ironstar-analytics`), reflecting the workspace aggregate's need to reference analytics types.

The infrastructure layer maintains clean separation from the domain layer: no infrastructure crate depends on any domain crate.
Within infrastructure, `ironstar-analytics-infra` is the only crate with a cross-infrastructure dependency, relying on `ironstar-event-bus` for Zenoh-based cache invalidation.

The binary crate `ironstar` is the sole composition root, depending on all ten library crates to wire domain logic and infrastructure together behind axum routes and SSE endpoints.
