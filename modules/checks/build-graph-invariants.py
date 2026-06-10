#!/usr/bin/env python3
"""Validate a committed build-graph snapshot against the accepted ceilings.

Pure, no-network, no-recursive-nix: reads the committed snapshot and baseline
JSON as plain files and checks the snapshot stays within the envelope. Reports
every violation, then exits non-zero with an actionable remediation message if any
ceiling is breached or any required root or member is missing.
"""

from __future__ import annotations

import json
import sys


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: build-graph-invariants.py <snapshot> <baseline>", file=sys.stderr)
        return 2
    snapshot = json.loads(open(sys.argv[1]).read())
    baseline = json.loads(open(sys.argv[2]).read())

    violations: list[str] = []

    snapshot_system = snapshot.get("system")
    baseline_system = baseline.get("system")
    if snapshot_system != baseline_system:
        violations.append(
            f"system mismatch: snapshot {snapshot_system!r} != baseline {baseline_system!r}"
        )

    missing_roots = snapshot.get("missing_roots", [])
    if missing_roots:
        violations.append(
            "snapshot did not capture every canonical root (missing: "
            + ", ".join(missing_roots)
            + ")"
        )

    snapshot_roots = set(snapshot.get("roots", []))
    for root in baseline.get("required_roots", []):
        if root not in snapshot_roots:
            violations.append(f"required root absent from snapshot: {root}")

    present_members = set(snapshot.get("workspace_members_present", []))
    for member in baseline.get("required_members", []):
        if member not in present_members:
            violations.append(f"required workspace member absent from snapshot: {member}")

    per_root = snapshot.get("per_root", {})
    for root, ceiling in baseline.get("per_root_logical_node_ceiling", {}).items():
        observed = per_root.get(root, {}).get("logical_nodes")
        if observed is None:
            violations.append(f"per-root node count missing for {root}")
        elif observed > ceiling:
            violations.append(
                f"{root} logical_nodes {observed} exceeds ceiling {ceiling}"
            )

    duplication = snapshot.get("duplication", {})
    for metric, ceiling in baseline.get("duplication_ceiling", {}).items():
        observed = duplication.get(metric)
        if observed is None:
            violations.append(f"duplication metric missing: {metric}")
        elif observed > ceiling:
            violations.append(
                f"duplication.{metric} {observed} exceeds ceiling {ceiling}"
            )

    heavy = snapshot.get("heavy_crate_distinct_compiles", {})
    for crate, ceiling in baseline.get("heavy_crate_distinct_compile_ceiling", {}).items():
        observed = heavy.get(crate, 0)
        if observed > ceiling:
            violations.append(
                f"heavy crate {crate} distinct compiles {observed} exceeds ceiling {ceiling}"
            )

    vendor_observed = snapshot.get("vendor_monolith_count", 0)
    vendor_ceiling = baseline.get("vendor_monolith_ceiling")
    if vendor_ceiling is not None and vendor_observed > vendor_ceiling:
        violations.append(
            f"vendor_monolith_count {vendor_observed} exceeds ceiling {vendor_ceiling}"
        )

    if violations:
        print("error: build-graph snapshot has left the accepted envelope.", file=sys.stderr)
        print("", file=sys.stderr)
        for violation in violations:
            print(f"    {violation}", file=sys.stderr)
        print("", file=sys.stderr)
        print(
            "If this is unintended drift, run `just regenerate-build-graph-snapshot` "
            "to refresh modules/checks/build-graph-snapshot.json and inspect the diff.",
            file=sys.stderr,
        )
        print(
            "If the new shape is a deliberate, reviewed change, raise the affected "
            "ceilings in modules/checks/build-graph-baseline.json.",
            file=sys.stderr,
        )
        return 1

    print(
        f"build-graph snapshot is within the accepted envelope "
        f"({len(baseline.get('required_members', []))} members, "
        f"{len(snapshot_roots)} roots)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
