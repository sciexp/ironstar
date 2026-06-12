-- Canned review queries over the hash-free build-graph projection
-- (the inputs.drvs build-closure, normalized to logical six-tuple nodes).
-- Run after load_duckdb.sql has populated nodes, edges, root_membership.

-- Q1: duplication by crate. Dev-profile crate compiles whose distinct store-hash
-- count exceeds one, grouped by crate name. drv_hash_count is summed across the
-- per-version logical nodes of a crate because the committed snapshot pools
-- duplication by crateName (collapsing multiple resolved versions of one crate),
-- and a store hash belongs to exactly one version, so the sum never double-counts.
-- The total row count reproduces the committed snapshot's
-- duplication.dev_profile_crates_with_multiplicity (202).
SELECT
    pname,
    sum(drv_hash_count) AS distinct_compile_drvs
FROM nodes
WHERE build_class IN ('crate-compile', 'crate-test-compile')
  AND profile = 'dev'
GROUP BY pname
HAVING sum(drv_hash_count) > 1
ORDER BY distinct_compile_drvs DESC, pname;

-- Q1 summary: the scalar that must match snapshot.json.
SELECT count(*) AS dev_profile_crates_with_multiplicity
FROM (
    SELECT pname
    FROM nodes
    WHERE build_class IN ('crate-compile', 'crate-test-compile')
      AND profile = 'dev'
    GROUP BY pname
    HAVING sum(drv_hash_count) > 1
);

-- Q2: heavy-crate ego neighborhoods. One-hop in-edges and out-edges for each of
-- the 13 heavy-crate seeds, across all roots, deduplicated to distinct logical
-- neighbors. direction='out' is a build dependency of the seed; direction='in' is
-- a consumer that builds against the seed.
WITH seeds(pname) AS (
    VALUES
        ('zenoh'), ('tokio'), ('sqlx'), ('sqlx-core'), ('sqlx-sqlite'),
        ('libduckdb-sys'), ('duckdb'), ('libsqlite3-sys'), ('arrow'),
        ('arrow-array'), ('moka'), ('arrow-buffer'), ('rkyv')
),
seed_nodes AS (
    SELECT DISTINCT n.node_id, n.pname AS seed_pname
    FROM nodes n
    JOIN seeds s ON n.pname = s.pname
    WHERE n.build_class IN ('crate-compile', 'crate-test-compile')
)
SELECT
    sn.seed_pname,
    'out' AS direction,
    np.pname AS neighbor_pname,
    np.version AS neighbor_version,
    np.profile AS neighbor_profile,
    count(DISTINCT e.root) AS roots_with_edge
FROM seed_nodes sn
JOIN edges e ON e.src_node_id = sn.node_id
JOIN nodes np ON np.node_id = e.dst_node_id
GROUP BY sn.seed_pname, np.pname, np.version, np.profile
UNION ALL
SELECT
    sn.seed_pname,
    'in' AS direction,
    np.pname AS neighbor_pname,
    np.version AS neighbor_version,
    np.profile AS neighbor_profile,
    count(DISTINCT e.root) AS roots_with_edge
FROM seed_nodes sn
JOIN edges e ON e.dst_node_id = sn.node_id
JOIN nodes np ON np.node_id = e.src_node_id
GROUP BY sn.seed_pname, np.pname, np.version, np.profile
ORDER BY seed_pname, direction, neighbor_pname, neighbor_version;

-- Q3: dev/release divergence. Crate compiles present in the dev profile but absent
-- from the release profile (and vice versa), keyed on (pname, version). A full anti-
-- join surfaces both directions; presence_gap names which profile is missing.
WITH dev AS (
    SELECT DISTINCT pname, version
    FROM nodes
    WHERE build_class = 'crate-compile' AND profile = 'dev'
),
rel AS (
    SELECT DISTINCT pname, version
    FROM nodes
    WHERE build_class = 'crate-compile' AND profile = 'release'
)
SELECT pname, version, 'release_missing' AS presence_gap
FROM dev
WHERE NOT EXISTS (
    SELECT 1 FROM rel WHERE rel.pname = dev.pname AND rel.version = dev.version
)
UNION ALL
SELECT pname, version, 'dev_missing' AS presence_gap
FROM rel
WHERE NOT EXISTS (
    SELECT 1 FROM dev WHERE dev.pname = rel.pname AND dev.version = rel.version
)
ORDER BY presence_gap, pname, version;

-- Q4: member condensation. Contract the graph to member_scope and count how many
-- distinct dependency nodes are shared across more than one member's test closure.
-- Member-scoped nodes (member_scope != 'shared') name the owning workspace member;
-- this measures the cross-member shared-dependency surface by joining member-scoped
-- source nodes to their shared dependency targets and counting members per target.
WITH member_edges AS (
    SELECT DISTINCT
        src.member_scope AS member,
        e.dst_node_id
    FROM edges e
    JOIN nodes src ON src.node_id = e.src_node_id
    JOIN nodes dst ON dst.node_id = e.dst_node_id
    WHERE src.member_scope <> 'shared'
      AND dst.member_scope = 'shared'
)
SELECT
    dst_node_id,
    count(DISTINCT member) AS sharing_members
FROM member_edges
GROUP BY dst_node_id
HAVING count(DISTINCT member) > 1
ORDER BY sharing_members DESC, dst_node_id;
