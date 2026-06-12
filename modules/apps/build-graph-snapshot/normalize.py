#!/usr/bin/env python3
"""Normalize `nix derivation show -r` graphs into a hash-free build-graph snapshot.

Reads the raw derivation-show JSON for each canonical root from a directory of
``<cat>__<name>.json`` files and emits a single deterministic snapshot to stdout.

Node identity is the hash-free tuple (system, logical_pname, version, build_class,
profile, member_scope): the store hash is dropped entirely so that semantically
identical rebuilds across commits collapse to one logical key, while the actionable
crate2nix profile/test/member distinctions are preserved. Duplication is the count
of distinct store hashes that collapse onto one logical key.

The snapshot is a committed, non-gating record of the build-graph shape, reviewed
through its git diff. It carries no store hashes and no timestamps and sorts every
collection, so two runs over the same roots produce byte-identical output.

The emitted ``provenance`` object fingerprints the substrate the snapshot was
derived from. ``root_drv_paths_sha256`` is the precise substrate fingerprint: it
hashes the sorted root ``.drv`` store paths, which are themselves a function of
Cargo.nix, flake.lock, and the nix expressions, so it changes whenever any of those
inputs change. ``cargo_nix_sha256`` is a human-meaningful pointer to one of those
inputs and does not on its own identify the substrate.
"""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

DRV_RE = re.compile(r"^/nix/store/([a-z0-9]{32})-(.+)\.drv$")
VENDOR_RE = re.compile(r"(vendor|cargo-deps|cargo-package)", re.IGNORECASE)

WORKSPACE_MEMBERS = [
    "ironstar",
    "ironstar-core",
    "ironstar-shared-kernel",
    "ironstar-todo",
    "ironstar-session",
    "ironstar-analytics",
    "ironstar-workspace",
    "ironstar-event-store",
    "ironstar-event-bus",
    "ironstar-analytics-infra",
    "ironstar-session-store",
]

HEAVY_CRATES = [
    "zenoh",
    "tokio",
    "sqlx",
    "sqlx-core",
    "sqlx-sqlite",
    "libduckdb-sys",
    "duckdb",
    "libsqlite3-sys",
    "arrow",
    "arrow-array",
    "moka",
    "arrow-buffer",
    "rkyv",
]

CANONICAL_ROOTS = [
    ("packages", "ironstar"),
    ("packages", "ironstar-release"),
    ("checks", "workspace-clippy"),
    ("checks", "workspace-test"),
    ("checks", "cargo-nix-lock-sync"),
    ("checks", "ironstar-test"),
    ("checks", "ironstar-core-test"),
    ("checks", "ironstar-shared-kernel-test"),
    ("checks", "ironstar-todo-test"),
    ("checks", "ironstar-session-test"),
    ("checks", "ironstar-analytics-test"),
    ("checks", "ironstar-workspace-test"),
    ("checks", "ironstar-event-store-test"),
    ("checks", "ironstar-event-bus-test"),
    ("checks", "ironstar-analytics-infra-test"),
    ("checks", "ironstar-session-store-test"),
]


def drv_hash(drvpath: str) -> str:
    match = DRV_RE.match(drvpath)
    return match.group(1) if match else drvpath


def build_class(name: str, env: dict) -> str:
    if env.get("crateName"):
        return "crate-test-compile" if env.get("buildTests") == "1" else "crate-compile"
    if name.startswith("run-tests-"):
        return "test-runner"
    if VENDOR_RE.search(name):
        return "vendor-blob"
    return "other"


def profile(env: dict) -> str:
    if env.get("buildTests") == "1":
        return "test"
    if env.get("release") == "1":
        return "release"
    return "dev" if env.get("crateName") else "na"


def member_scope(env: dict, members: set[str]) -> str:
    crate = env.get("crateName")
    if env.get("buildTests") == "1" and crate in members:
        return crate
    return "shared"


def logical_key(name: str, env: dict, system: str, members: set[str]):
    klass = build_class(name, env)
    if klass in ("crate-compile", "crate-test-compile"):
        pname = env.get("crateName", "")
        version = env.get("crateVersion", "")
    else:
        pname = name
        version = ""
    return (system, pname, version, klass, profile(env), member_scope(env, members))


def main() -> int:
    if len(sys.argv) != 5:
        print(
            "usage: normalize.py <raw-dir> <system> "
            "<cargo-nix-sha256> <root-drv-paths-sha256>",
            file=sys.stderr,
        )
        return 2
    raw_dir = Path(sys.argv[1])
    system = sys.argv[2]
    cargo_nix_sha256 = sys.argv[3]
    root_drv_paths_sha256 = sys.argv[4]
    members = set(WORKSPACE_MEMBERS)

    key_hashes: dict[tuple, set[str]] = defaultdict(set)
    per_root_nodes: dict[str, int] = {}
    per_root_edges: dict[str, int] = {}
    member_present: dict[str, bool] = {member: False for member in WORKSPACE_MEMBERS}
    heavy_hashes: dict[str, set[str]] = defaultdict(set)
    vendor_hashes: set[str] = set()
    dev_compile_hashes: dict[str, set[str]] = defaultdict(set)
    release_compile_hashes: dict[str, set[str]] = defaultdict(set)
    test_variant_hashes: dict[str, set[str]] = defaultdict(set)

    missing = []
    for cat, name in CANONICAL_ROOTS:
        path = raw_dir / f"{cat}__{name}.json"
        if not path.exists():
            missing.append(f"{cat}.{name}")
            continue
        data = json.loads(path.read_text())
        drvs = data["derivations"]
        root_keys: set[tuple] = set()
        root_edges: set[tuple] = set()
        for drvpath, node in drvs.items():
            env = node.get("env", {}) or {}
            node_name = node.get("name", "")
            klass = build_class(node_name, env)
            key = logical_key(node_name, env, system, members)
            digest = drv_hash(drvpath)
            key_hashes[key].add(digest)
            root_keys.add(key)
            crate = env.get("crateName")
            if klass == "vendor-blob":
                vendor_hashes.add(digest)
            if crate in members and (
                klass == "crate-compile" or env.get("buildTests") == "1"
            ):
                member_present[crate] = True
            if crate in HEAVY_CRATES and klass in (
                "crate-compile",
                "crate-test-compile",
            ):
                heavy_hashes[crate].add(digest)
            if crate:
                if env.get("buildTests") == "1":
                    test_variant_hashes[crate].add(digest)
                elif env.get("release") == "1":
                    release_compile_hashes[crate].add(digest)
                else:
                    dev_compile_hashes[crate].add(digest)
            for child in (node.get("inputs", {}).get("drvs", {}) or {}).keys():
                child_node = drvs.get(child)
                if child_node is None:
                    continue
                child_env = child_node.get("env", {}) or {}
                child_key = logical_key(
                    child_node.get("name", ""), child_env, system, members
                )
                root_edges.add((key, child_key))
        per_root_nodes[f"{cat}.{name}"] = len(root_keys)
        per_root_edges[f"{cat}.{name}"] = len(root_edges)

    actionable_dup = {
        crate: len(hashes)
        for crate, hashes in dev_compile_hashes.items()
        if len(hashes) > 1
    }
    snapshot = {
        "schema_version": 1,
        "system": system,
        "note": (
            "hash-free build-graph snapshot keyed on "
            "(system, logical_pname, version, build_class, profile, member_scope); "
            "duplication = distinct store hashes per logical key; "
            "regenerate with `just regenerate-build-graph-snapshot`"
        ),
        "provenance": {
            "cargo_nix_sha256": cargo_nix_sha256,
            "root_drv_paths_sha256": root_drv_paths_sha256,
        },
        "roots": sorted(f"{cat}.{name}" for cat, name in CANONICAL_ROOTS),
        "missing_roots": sorted(missing),
        "per_root": {
            root: {"logical_nodes": per_root_nodes[root], "logical_edges": per_root_edges[root]}
            for root in sorted(per_root_nodes)
        },
        "workspace_members_present": [
            member for member in WORKSPACE_MEMBERS if member_present[member]
        ],
        "workspace_members_total": len(WORKSPACE_MEMBERS),
        "duplication": {
            "dev_profile_crates_with_multiplicity": len(actionable_dup),
            "dev_profile_excess_compile_drvs": sum(v - 1 for v in actionable_dup.values()),
            "release_profile_distinct_compiles": sum(
                len(h) for h in release_compile_hashes.values()
            ),
            "test_variant_member_compiles": sum(
                len(h) for h in test_variant_hashes.values()
            ),
        },
        "heavy_crate_distinct_compiles": {
            crate: len(heavy_hashes[crate])
            for crate in HEAVY_CRATES
            if heavy_hashes[crate]
        },
        "vendor_monolith_count": len(vendor_hashes),
    }
    print(json.dumps(snapshot, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
