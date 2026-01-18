-- Event store schema adapted from fstore-sql for SQLite.
-- Reference: ~/projects/rust-workspace/fstore-sql/schema.sql

-- Events table: append-only event log with optimistic locking via previous_id chain.
-- Global monotonic ordering via id column for SSE Last-Event-ID semantics.
CREATE TABLE IF NOT EXISTS events (
    -- Global SSE sequence (monotonic ordering across all aggregates)
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Event UUID (Version for optimistic locking)
    event_id TEXT NOT NULL UNIQUE CHECK(length(event_id) = 36),
    -- Aggregate type from DeciderType trait (e.g., "Todo", "QuerySession")
    aggregate_type TEXT NOT NULL,
    -- Aggregate identifier from Identifier trait
    aggregate_id TEXT NOT NULL,
    -- Chain predecessor for optimistic locking (NULL for first event)
    previous_id TEXT UNIQUE REFERENCES events(event_id),
    -- Event variant name from EventType trait
    event_type TEXT NOT NULL,
    -- Schema version for upcaster routing
    schema_version INTEGER NOT NULL DEFAULT 1,
    -- JSON event data (payload)
    payload TEXT NOT NULL CHECK(json_valid(payload)),
    -- Command UUID that caused this event (causation tracking)
    command_id TEXT,
    -- JSON correlation context (correlation_id, caused_by, actor)
    metadata TEXT CHECK(metadata IS NULL OR json_valid(metadata)),
    -- Terminal state marker from IsFinal trait
    final INTEGER NOT NULL DEFAULT 0,
    -- Event creation timestamp (ISO 8601 UTC)
    created_at TEXT NOT NULL DEFAULT(datetime('now', 'utc'))
) STRICT;

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_type, aggregate_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_stream ON events(aggregate_id, id);
CREATE INDEX IF NOT EXISTS idx_events_previous ON events(previous_id) WHERE previous_id IS NOT NULL;

-- Trigger: Prevent UPDATE on events (immutability)
CREATE TRIGGER IF NOT EXISTS prevent_event_update
BEFORE UPDATE ON events
BEGIN
    SELECT RAISE(ABORT, 'Events are immutable: UPDATE not allowed');
END;

-- Trigger: Prevent DELETE on events (immutability)
CREATE TRIGGER IF NOT EXISTS prevent_event_delete
BEFORE DELETE ON events
BEGIN
    SELECT RAISE(ABORT, 'Events are immutable: DELETE not allowed');
END;

-- Trigger: NULL previous_id only allowed for first event per aggregate
CREATE TRIGGER IF NOT EXISTS check_first_event
BEFORE INSERT ON events
WHEN NEW.previous_id IS NULL
BEGIN
    SELECT RAISE(ABORT, 'previous_id can only be NULL for the first event in an aggregate')
    WHERE EXISTS(
        SELECT 1 FROM events
        WHERE aggregate_type = NEW.aggregate_type
        AND aggregate_id = NEW.aggregate_id
    );
END;

-- Trigger: previous_id must reference an event in the same aggregate
CREATE TRIGGER IF NOT EXISTS check_previous_id_same_aggregate
BEFORE INSERT ON events
WHEN NEW.previous_id IS NOT NULL
BEGIN
    SELECT RAISE(ABORT, 'previous_id must reference an event in the same aggregate')
    WHERE NOT EXISTS(
        SELECT 1 FROM events
        WHERE event_id = NEW.previous_id
        AND aggregate_type = NEW.aggregate_type
        AND aggregate_id = NEW.aggregate_id
    );
END;

-- Trigger: Cannot append to finalized stream
CREATE TRIGGER IF NOT EXISTS check_not_final
BEFORE INSERT ON events
BEGIN
    SELECT RAISE(ABORT, 'Cannot append events to a finalized aggregate stream')
    WHERE EXISTS(
        SELECT 1 FROM events
        WHERE aggregate_type = NEW.aggregate_type
        AND aggregate_id = NEW.aggregate_id
        AND final = 1
    );
END;
