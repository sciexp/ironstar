#!/usr/bin/env python3
"""Project the `nix derivation show -r` raw graphs into hash-free node and edge tables.

This is the same ``inputs.drvs`` build-closure projection that
``build-graph-snapshot/normalize.py`` summarizes, emitted here in a relational
shape (one node per line, one edge per line) so DuckDB can answer ego-neighborhood
and divergence queries the scalar snapshot cannot.

Node identity is normalize.py's hash-free six-tuple
``(system, logical_pname, version, build_class, profile, member_scope)``; the store
hash is dropped so semantically identical rebuilds collapse to one logical node.
The duplication signal is preserved per node as ``drv_hash_count``: the number of
distinct store-hash digests that collapse onto that logical key, replicating
normalize.py's ``key_hashes`` accumulation.

Edges are the ``inputs.drvs`` adjacency of the raw derivation graph, lifted from
store-path endpoints to logical-node endpoints and unioned across all canonical
roots with the originating root retained as a column. normalize.py computes the
same ``root_edges`` set per root (its lines ~188-196) but discards it after
counting; this module persists it.

Node and edge lines are sorted for stable, reviewable diffs.

We import normalize.py as a module (via ``sys.path``) rather than copying its
``logical_key`` / ``build_class`` / ``profile`` / ``member_scope`` / ``drv_hash``
functions, so the compendium's node identity can never drift from the committed
snapshot's node identity.
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

SNAPSHOT_DIR = Path(__file__).resolve().parent.parent / "build-graph-snapshot"
sys.path.insert(0, str(SNAPSHOT_DIR))

import normalize as nz  # noqa: E402


def node_id(key: tuple) -> str:
    """Stable string join of the six-tuple logical key."""
    return "|".join(str(part) for part in key)


def main() -> int:
    if len(sys.argv) != 4:
        print(
            "usage: emit_edges.py <raw-dir> <system> <out-dir>",
            file=sys.stderr,
        )
        return 2
    raw_dir = Path(sys.argv[1])
    system = sys.argv[2]
    out_dir = Path(sys.argv[3])
    out_dir.mkdir(parents=True, exist_ok=True)
    members = set(nz.WORKSPACE_MEMBERS)

    key_hashes: dict[tuple, set[str]] = defaultdict(set)
    key_fields: dict[tuple, tuple] = {}
    edges: set[tuple[str, str, str]] = set()
    root_membership: set[tuple[str, str]] = set()

    missing = []
    for cat, name in nz.CANONICAL_ROOTS:
        path = raw_dir / f"{cat}__{name}.json"
        if not path.exists():
            missing.append(f"{cat}.{name}")
            continue
        root = f"{cat}.{name}"
        data = json.loads(path.read_text())
        drvs = data["derivations"]
        for drvpath, node in drvs.items():
            env = node.get("env", {}) or {}
            node_name = node.get("name", "")
            key = nz.logical_key(node_name, env, system, members)
            key_hashes[key].add(nz.drv_hash(drvpath))
            key_fields[key] = key
            src_id = node_id(key)
            root_membership.add((root, src_id))
            for child in (node.get("inputs", {}).get("drvs", {}) or {}).keys():
                child_node = drvs.get(child)
                if child_node is None:
                    continue
                child_env = child_node.get("env", {}) or {}
                child_key = nz.logical_key(
                    child_node.get("name", ""), child_env, system, members
                )
                edges.add((root, src_id, node_id(child_key)))

    if missing:
        print(f"warning: missing roots {sorted(missing)}", file=sys.stderr)

    nodes_path = out_dir / "nodes.ndjson"
    with nodes_path.open("w") as fh:
        for key in sorted(key_fields, key=node_id):
            system_, pname, version, build_class, profile, member_scope = key
            record = {
                "node_id": node_id(key),
                "system": system_,
                "pname": pname,
                "version": version,
                "build_class": build_class,
                "profile": profile,
                "member_scope": member_scope,
                "drv_hash_count": len(key_hashes[key]),
            }
            fh.write(json.dumps(record, sort_keys=True) + "\n")

    edges_path = out_dir / "edges.ndjson"
    with edges_path.open("w") as fh:
        for root, src_id, dst_id in sorted(edges):
            src_key = src_id.split("|")
            dst_key = dst_id.split("|")
            record = {
                "root": root,
                "src": src_key,
                "dst": dst_key,
            }
            fh.write(json.dumps(record, sort_keys=True) + "\n")

    membership_path = out_dir / "root_membership.ndjson"
    with membership_path.open("w") as fh:
        for root, nid in sorted(root_membership):
            fh.write(json.dumps({"root": root, "node_id": nid}, sort_keys=True) + "\n")

    print(
        f"wrote {len(key_fields)} nodes, {len(edges)} edges, "
        f"{len(root_membership)} root-membership rows to {out_dir}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
