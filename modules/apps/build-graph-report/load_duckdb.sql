-- Load the hash-free build-graph projection (the inputs.drvs build-closure,
-- normalized to logical six-tuple nodes) into a queryable DuckDB database.
--
-- The three NDJSON inputs are emitted by emit_edges.py. Node identity is the
-- hash-free tuple (system, pname, version, build_class, profile, member_scope);
-- edges are the inputs.drvs adjacency lifted to logical-node endpoints and unioned
-- across the canonical roots; root_membership records which roots reach each node.
--
-- :raw_dir is bound on the duckdb command line (-c "SET VARIABLE ...") or via the
-- shell heredoc that substitutes the absolute logs/build-graph path. Endpoints are
-- reconstructed from the six-element src/dst arrays with array_to_string(..., '|'),
-- matching emit_edges.node_id.

INSTALL json;
LOAD json;

DROP TABLE IF EXISTS nodes;
CREATE TABLE nodes AS
SELECT
    node_id,
    system,
    pname,
    version,
    build_class,
    profile,
    member_scope,
    drv_hash_count
FROM read_json_auto(getvariable('raw_dir') || '/nodes.ndjson');

DROP TABLE IF EXISTS edges;
CREATE TABLE edges AS
SELECT
    root,
    array_to_string(src, '|') AS src_node_id,
    array_to_string(dst, '|') AS dst_node_id
FROM read_json_auto(getvariable('raw_dir') || '/edges.ndjson');

DROP TABLE IF EXISTS root_membership;
CREATE TABLE root_membership AS
SELECT
    root,
    node_id
FROM read_json_auto(getvariable('raw_dir') || '/root_membership.ndjson');

SELECT
    (SELECT count(*) FROM nodes) AS node_count,
    (SELECT count(*) FROM edges) AS edge_count,
    (SELECT count(*) FROM root_membership) AS root_membership_count;
