#!/usr/bin/env python3
"""Render the build-graph report from a committed raw derivation-show snapshot.

Reads ``logs/build-graph/raw/`` (the ``<cat>__<name>.json`` files produced by the
``build-graph-snapshot`` extraction) and emits graphviz ``.dot`` sources plus rendered
SVG/PNG/PDF artifacts under ``logs/build-graph/graph-viz/``.

Two incommensurable edge relations live in the raw derivations, and each report
view is built from exactly one of them:

  env.dependencies / env.buildDependencies
      The crate-to-crate dependency DAG. crate2nix wires these as space-separated
      ``/nix/store/<hash>-rust_<crate>-<ver>-lib`` store paths. Reverse-engineering
      them recovers the architecture: ~1382 edges over the 443 crate derivations.
      The ARCHITECTURE views (member-dag, crate-overview) are built from this relation.

  inputs.drvs
      The full transitive build closure of a derivation (every ``.drv`` it pulls in,
      not just direct crate deps). Reverse-reachability from a seed crate over this
      relation yields exactly the set of derivations that rebuild when the seed's
      version bumps. The CONE views are built from this relation.

The two are not comparable: an architecture edge count and a closure-reachable node
count measure different things, and the index says so explicitly.

Node identity is the full crate name (crate + version) so that a crate carried at two
versions appears as two distinct nodes, matching crate2nix's per-version store paths.
The build system is discovered from the derivation ``system`` field rather than assumed.

Renderer binaries (``dot``, ``sfdp``) are invoked directly; in the packaged app they
arrive on PATH via ``runtimeInputs = [ graphviz ]``. There is no ``nixpkgs#graphviz``
app to ``nix run`` — only the individual binaries exist.
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path


LIB_RE = re.compile(r"/nix/store/[a-z0-9]+-rust_(?P<full>[^ ]+?)-lib(?:\b|$)")

# crate2nix profile/test variants share the workspace member crate names; the role
# coloring is the architecture-level grouping documented in the project's CLAUDE.md.
ROLE_OF_MEMBER = {
    "ironstar-core": "foundation",
    "ironstar-shared-kernel": "foundation",
    "ironstar-todo": "domain",
    "ironstar-session": "domain",
    "ironstar-analytics": "domain",
    "ironstar-workspace": "domain",
    "ironstar-event-store": "infra",
    "ironstar-event-bus": "infra",
    "ironstar-analytics-infra": "infra",
    "ironstar-session-store": "infra",
    "ironstar": "binary",
}
ROLE_FILL = {
    "foundation": "#cfe8ff",
    "domain": "#d6f5d6",
    "infra": "#ffe8cc",
    "binary": "#f5d6e8",
}

# Cone palette, preserved from the seed convention so the rendered cones read the same.
CONE_SEED_FILL = "#d93025"
CONE_MEMBER_FILL = "#34a853"
CONE_BINARY_FILL = "#1a73e8"
CONE_INTERMEDIATE_FILL = "#f1f3f4"

CONE_SEEDS = ["adler2", "zenoh", "tokio", "libduckdb-sys", "sqlx-core"]

# DPI conventions carried over from the seed renders (RECIPE.md / index.md).
MEMBER_DAG_DPI = 140
CRATE_OVERVIEW_DPI = 200
CONE_DPI = {"tokio": 150, "_default": 130}

# Canonical raw root: the release binary's closure matches CI's .#ironstar-release and
# the dev binary is the deterministic fallback. Both carry the full crate set.
RAW_ROOT_PREFERENCE = [
    "packages__ironstar-release.json",
    "packages__ironstar.json",
]


@dataclass(frozen=True)
class Crate:
    """A single crate derivation node, identified by crate name and version."""

    name: str
    version: str

    @property
    def full(self) -> str:
        return f"{self.name}-{self.version}" if self.version else self.name

    @property
    def node_id(self) -> str:
        ident = self.full.replace(".", "_").replace("+", "_").replace("-", "_")
        return ident


@dataclass
class Graph:
    """Crate DAG (architecture relation) plus build closure (cone relation)."""

    system: str
    crates: dict[str, Crate]                       # full-name -> Crate
    members: set[str]                              # full-names that are workspace members
    binary: str | None                             # full-name of the ironstar binary
    dep_edges: set[tuple[str, str]]                # architecture: dependent -> dependency
    closure_fwd: dict[str, set[str]] = field(default_factory=dict)  # cone: drv -> {child drvs}

    @property
    def member_short(self) -> dict[str, str]:
        return {full: self.crates[full].name for full in self.members}


def crate_name_and_version(env: dict, drv_name: str) -> tuple[str, str] | None:
    """Authoritative crate identity from env fields, falling back to the drv name."""
    name = env.get("crateName")
    version = env.get("crateVersion") or env.get("version") or ""
    if name:
        return name, version
    if drv_name.startswith("rust_"):
        stem = drv_name[len("rust_"):]
        parts = stem.rsplit("-", 1)
        if len(parts) == 2 and parts[1] and parts[1][0].isdigit():
            return parts[0], parts[1]
        return stem, ""
    return None


def parse_lib_paths(field_value: str) -> list[str]:
    if not field_value:
        return []
    return [m.group("full") for m in LIB_RE.finditer(field_value)]


def load_graph(raw_dir: Path) -> Graph:
    raw_path = _pick_raw_root(raw_dir)
    data = json.loads(raw_path.read_text())
    derivations = data["derivations"]

    rust_drvs = {
        drvpath: node
        for drvpath, node in derivations.items()
        if node.get("name", "").startswith("rust_")
    }

    system = _discover_system(rust_drvs)

    crates: dict[str, Crate] = {}
    drvpath_to_full: dict[str, str] = {}
    for drvpath, node in rust_drvs.items():
        env = node.get("env", {}) or {}
        identity = crate_name_and_version(env, node.get("name", ""))
        if identity is None:
            continue
        name, version = identity
        crate = Crate(name=name, version=version)
        crates[crate.full] = crate
        drvpath_to_full[drvpath] = crate.full

    members = {full for full, c in crates.items() if c.name in ROLE_OF_MEMBER}
    binary = next((full for full, c in crates.items() if c.name == "ironstar"), None)

    dep_edges: set[tuple[str, str]] = set()
    for drvpath, node in rust_drvs.items():
        src_full = drvpath_to_full.get(drvpath)
        if src_full is None:
            continue
        env = node.get("env", {}) or {}
        for field_name in ("dependencies", "buildDependencies"):
            for dst_full in parse_lib_paths(env.get(field_name, "")):
                if dst_full != src_full and dst_full in crates:
                    dep_edges.add((src_full, dst_full))

    # Build closure over inputs.drvs, restricted to rust crate nodes; the cone relation.
    closure_fwd: dict[str, set[str]] = {full: set() for full in crates}
    for drvpath, node in rust_drvs.items():
        src_full = drvpath_to_full.get(drvpath)
        if src_full is None:
            continue
        children = (node.get("inputs", {}) or {}).get("drvs", {}) or {}
        for child_drvpath in children:
            child_full = drvpath_to_full.get(child_drvpath)
            if child_full is not None and child_full != src_full:
                closure_fwd[src_full].add(child_full)

    return Graph(
        system=system,
        crates=crates,
        members=members,
        binary=binary,
        dep_edges=dep_edges,
        closure_fwd=closure_fwd,
    )


def _pick_raw_root(raw_dir: Path) -> Path:
    for candidate in RAW_ROOT_PREFERENCE:
        path = raw_dir / candidate
        if path.exists():
            return path
    available = sorted(p.name for p in raw_dir.glob("*.json"))
    raise SystemExit(
        f"no canonical raw root found in {raw_dir}; "
        f"looked for {RAW_ROOT_PREFERENCE}, available: {available}"
    )


def _discover_system(rust_drvs: dict) -> str:
    systems = {node.get("system") for node in rust_drvs.values() if node.get("system")}
    if len(systems) == 1:
        return next(iter(systems))
    if systems:
        return sorted(systems)[0]
    return "unknown-system"


def quote(value: str) -> str:
    return '"' + value.replace('"', '\\"') + '"'


def write_member_dag(graph: Graph, out_dir: Path) -> Path:
    """Role-colored DAG of the workspace members, member-to-member edges only.

    Binary at top, infra/domain in the middle, foundations at the bottom; the edges
    are the architecture (dependency) relation restricted to the 11 members.
    """
    member_edges = sorted(
        (a, b) for a, b in graph.dep_edges if a in graph.members and b in graph.members
    )
    lines = [
        "digraph member_dag {",
        "  rankdir=BT;",
        '  bgcolor="white";',
        '  graph [fontname="Helvetica", ranksep=0.6, nodesep=0.35, splines=true, pad=0.3];',
        '  node [style="filled,rounded", shape=box, fontname="Helvetica", fontsize=14, penwidth=1.6];',
        '  edge [color="#555555", arrowsize=0.7];',
        f'  labelloc="t"; fontsize=15; fontname="Helvetica-Bold";'
        f' label="Workspace member architecture'
        f'  ({len(graph.members)} members, {len(member_edges)} member edges; {graph.system})";',
    ]
    for full in sorted(graph.members, key=lambda f: graph.crates[f].name):
        crate = graph.crates[full]
        role = ROLE_OF_MEMBER.get(crate.name, "domain")
        shape = "box3d" if role == "binary" else "box"
        label = f"{crate.name}\\n[{role}] v{crate.version}"
        lines.append(
            f"  {quote(crate.node_id)} "
            f'[fillcolor="{ROLE_FILL[role]}", shape={shape}, label={quote(label)}];'
        )
    for a, b in member_edges:
        lines.append(f"  {quote(graph.crates[a].node_id)} -> {quote(graph.crates[b].node_id)};")
    lines += _role_legend_cluster()
    lines.append("}")
    path = out_dir / "member-dag.dot"
    path.write_text("\n".join(lines) + "\n")
    return path


def _role_legend_cluster() -> list[str]:
    lines = [
        "  subgraph cluster_role_legend {",
        '    label="role"; fontsize=11; fontname="Helvetica"; style="rounded"; color="#cccccc"; margin=8;',
        '    node [fontsize=10, shape=box, style="filled,rounded"];',
    ]
    for role in ("binary", "domain", "infra", "foundation"):
        shape = "box3d" if role == "binary" else "box"
        lines.append(
            f"    lg_{role} "
            f'[label="{role}", fillcolor="{ROLE_FILL[role]}", shape={shape}];'
        )
    lines.append("    lg_binary -> lg_domain -> lg_infra -> lg_foundation [style=invis];")
    lines.append("  }")
    return lines


def write_crate_overview(graph: Graph, out_dir: Path) -> Path:
    """Full crate topology as a force-directed (sfdp) map, nodes sized by fan-in.

    Members are large role-colored boxes; heavy crates (high fan-in) are sized up and
    labeled; the long tail forms a gray fan-in-sized cloud. Only members and the
    top fan-in nodes carry labels so the map reads as topology, not a label hairball.
    """
    indeg: dict[str, int] = defaultdict(int)
    for _, b in graph.dep_edges:
        indeg[b] += 1
    max_indeg = max(indeg.values(), default=1)
    top_fanin = {
        full
        for full, _ in sorted(indeg.items(), key=lambda kv: -kv[1])[:20]
    }

    def size_for(full: str) -> float:
        return 0.15 + 1.1 * (indeg.get(full, 0) / max_indeg)

    lines = [
        "digraph crate_overview {",
        "  layout=sfdp;",
        '  overlap=prism; overlap_scaling=-4; sep="+8"; K=0.6;',
        '  bgcolor="white";',
        '  node [shape=ellipse, fontname="Helvetica", fontsize=8, '
        'width=0.1, height=0.1, margin="0.02,0.01"];',
        '  edge [color="#cccccc", arrowsize=0.4, penwidth=0.4];',
        f'  labelloc="t"; fontsize=16; fontname="Helvetica-Bold";'
        f' label="Crate topology'
        f'  ({len(graph.crates)} crate derivations, {len(graph.dep_edges)} dependency edges; {graph.system})";',
    ]
    for full in sorted(graph.crates):
        crate = graph.crates[full]
        size = size_for(full)
        if full in graph.members:
            role = ROLE_OF_MEMBER.get(crate.name, "domain")
            shape = "box3d" if role == "binary" else "box"
            lines.append(
                f"  {quote(crate.node_id)} "
                f'[label={quote(crate.name)}, style="filled,bold", fillcolor="{ROLE_FILL[role]}", '
                f'shape={shape}, fontsize=13, penwidth=2.2, color="#222222", '
                f"width={size:.3f}, height={size:.3f}];"
            )
        elif full in top_fanin:
            lines.append(
                f"  {quote(crate.node_id)} "
                f'[label={quote(crate.name)}, style="filled", fillcolor="#f6c6b8", '
                f'fontsize=11, penwidth=1.0, color="#b5651d", '
                f"width={size:.3f}, height={size:.3f}];"
            )
        else:
            lines.append(
                f"  {quote(crate.node_id)} "
                f'[label="", color="#aaaaaa", '
                f"width={size:.3f}, height={size:.3f}, fixedsize=true];"
            )
    for a, b in sorted(graph.dep_edges):
        if a in graph.members or b in graph.members:
            lines.append(
                f"  {quote(graph.crates[a].node_id)} -> {quote(graph.crates[b].node_id)} "
                f'[color="#7799bb", penwidth=0.7];'
            )
        else:
            lines.append(f"  {quote(graph.crates[a].node_id)} -> {quote(graph.crates[b].node_id)};")
    lines.append("}")
    path = out_dir / "crate-overview.dot"
    path.write_text("\n".join(lines) + "\n")
    return path


@dataclass
class ConeResult:
    seed_short: str
    seed_full: str
    nodes: set[str]
    deps_in: int
    members_in: int
    edges: int
    dot_path: Path


def _reverse_cone(graph: Graph, seed_full: str) -> set[str]:
    """Reverse-reachability over the build closure: everything that rebuilds on a bump."""
    rev: dict[str, set[str]] = defaultdict(set)
    for src, children in graph.closure_fwd.items():
        for child in children:
            rev[child].add(src)
    seen: set[str] = set()
    stack = [seed_full]
    while stack:
        current = stack.pop()
        for parent in rev[current]:
            if parent not in seen:
                seen.add(parent)
                stack.append(parent)
    return seen | {seed_full}


def _transitive_reduction(
    nodes: set[str], succ_of: dict[str, set[str]]
) -> list[tuple[str, str]]:
    """Minimal edge set whose transitive closure equals the induced subgraph's.

    ``succ_of[u]`` is u's direct successors (dependencies) already restricted to nodes.
    """
    reach: dict[str, set[str]] = {}

    def reachable(u: str) -> set[str]:
        if u in reach:
            return reach[u]
        reach[u] = set()  # guard against cycles in the closure relation
        acc: set[str] = set()
        for v in succ_of[u]:
            acc.add(v)
            acc |= reachable(v)
        reach[u] = acc
        return acc

    for u in nodes:
        reachable(u)

    reduced: list[tuple[str, str]] = []
    for u in nodes:
        for v in succ_of[u]:
            if not any(v in reach[w] for w in succ_of[u] if w != v):
                reduced.append((u, v))
    return reduced


def build_cone(graph: Graph, seed_short: str, out_dir: Path) -> ConeResult | None:
    candidates = sorted(
        full for full, c in graph.crates.items() if c.name == seed_short
    )
    if not candidates:
        print(
            f"warning: cone seed {seed_short!r} absent from graph; skipping",
            file=sys.stderr,
        )
        return None
    seed_full = candidates[0]
    nodes = _reverse_cone(graph, seed_full)

    succ_of = {u: (graph.closure_fwd.get(u, set()) & nodes) for u in nodes}
    reduced = _transitive_reduction(nodes, succ_of)

    deps_in = sorted(n for n in nodes if n not in graph.members)
    members_in = sorted(n for n in nodes if n in graph.members)
    total_deps = sum(1 for full in graph.crates if full not in graph.members)
    total_members = len(graph.members)
    seed_version = graph.crates[seed_full].version

    lines = [
        f'digraph "{seed_short}_cone" {{',
        "  rankdir=BT;",
        '  bgcolor="white";',
        '  graph [fontname="Helvetica", fontsize=11, ranksep=0.55, nodesep=0.28, splines=true, pad=0.3];',
        '  node [fontname="Helvetica", fontsize=10, shape=box, style="rounded,filled", penwidth=1.0, margin="0.10,0.05"];',
        '  edge [color="#9aa0a6", arrowsize=0.6, penwidth=0.8];',
    ]
    title = (
        f"Fan-in cone: {seed_short} v{seed_version}"
        f"  |  rebuild set = {len(deps_in)} of {total_deps} dep crates"
        f" + {len(members_in)} of {total_members} workspace members"
        f"  (total {len(nodes)} crate derivations; edges transitively reduced; {graph.system})"
    )
    lines.append(f'  labelloc="t"; fontsize=13; fontname="Helvetica-Bold"; label="{title}";')

    for full in sorted(nodes, key=lambda f: graph.crates[f].full):
        crate = graph.crates[full]
        if full == seed_full:
            fill, fontcolor, penwidth, fontsize, shape = CONE_SEED_FILL, "white", "2.2", "13", "box"
        elif crate.name == "ironstar":
            fill, fontcolor, penwidth, fontsize, shape = CONE_BINARY_FILL, "white", "1.8", "12", "box3d"
        elif full in graph.members:
            fill, fontcolor, penwidth, fontsize, shape = CONE_MEMBER_FILL, "white", "1.6", "11", "box"
        else:
            fill, fontcolor, penwidth, fontsize, shape = CONE_INTERMEDIATE_FILL, "#202124", "1.0", "10", "box"
        label = f"{crate.name}\\n{crate.version}"
        lines.append(
            f"  {quote(crate.node_id)} "
            f'[label="{label}", fillcolor="{fill}", fontcolor="{fontcolor}", '
            f"penwidth={penwidth}, fontsize={fontsize}, shape={shape}];"
        )

    # u depends on v; draw v -> u so the bump cascade arrows point up toward dependents.
    for u, v in reduced:
        lines.append(f"  {quote(graph.crates[v].node_id)} -> {quote(graph.crates[u].node_id)};")

    lines += _cone_legend_cluster()
    lines.append("}")

    dot_path = out_dir / f"{seed_short}-cone.dot"
    dot_path.write_text("\n".join(lines) + "\n")
    return ConeResult(
        seed_short=seed_short,
        seed_full=seed_full,
        nodes=nodes,
        deps_in=len(deps_in),
        members_in=len(members_in),
        edges=len(reduced),
        dot_path=dot_path,
    )


def _cone_legend_cluster() -> list[str]:
    return [
        "  subgraph cluster_legend {",
        '    label="legend  (arrows: dependency -> dependent; bump cascades upward)";'
        ' fontsize=10; fontname="Helvetica"; style="rounded"; color="#cccccc"; margin=8;',
        '    node [fontsize=9, margin="0.08,0.04"];',
        f'    lg_seed [label="bumped crate", fillcolor="{CONE_SEED_FILL}", fontcolor="white", penwidth=2.0];',
        f'    lg_ws [label="workspace member", fillcolor="{CONE_MEMBER_FILL}", fontcolor="white"];',
        f'    lg_bin [label="binary (ironstar)", fillcolor="{CONE_BINARY_FILL}", fontcolor="white", shape=box3d];',
        f'    lg_dep [label="intermediate dep", fillcolor="{CONE_INTERMEDIATE_FILL}", fontcolor="#202124"];',
        "    lg_seed -> lg_dep -> lg_ws -> lg_bin [style=invis];",
        "  }",
    ]


@dataclass
class RenderResult:
    dot_path: Path
    svg: Path | None = None
    png: Path | None = None
    pdf: Path | None = None


def _pdf_supported(engine: str) -> bool:
    try:
        subprocess.run(
            [engine, "-Tpdf"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
        )
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def render_dot(
    dot_path: Path, engine: str, dpi: int, emit_pdf: bool
) -> RenderResult:
    stem = dot_path.with_suffix("")
    result = RenderResult(dot_path=dot_path)

    svg_path = stem.with_suffix(".svg")
    _run_engine(engine, dot_path, svg_path, fmt="svg")
    result.svg = svg_path

    png_path = stem.with_suffix(".png")
    _run_engine(engine, dot_path, png_path, fmt="png", dpi=dpi)
    result.png = png_path

    if emit_pdf:
        pdf_path = stem.with_suffix(".pdf")
        _run_engine(engine, dot_path, pdf_path, fmt="pdf")
        result.pdf = pdf_path

    return result


def _run_engine(
    engine: str, dot_path: Path, out_path: Path, fmt: str, dpi: int | None = None
) -> None:
    cmd = [engine, f"-T{fmt}"]
    if dpi is not None:
        cmd.append(f"-Gdpi={dpi}")
    cmd += [str(dot_path), "-o", str(out_path)]
    subprocess.run(cmd, check=True)


def write_index(
    graph: Graph,
    out_dir: Path,
    arch_renders: list[RenderResult],
    cone_results: list[ConeResult],
    cone_renders: dict[str, RenderResult],
    skipped_seeds: list[str],
) -> Path:
    arch_edge_count = len(graph.dep_edges)
    member_edges = sum(
        1 for a, b in graph.dep_edges if a in graph.members and b in graph.members
    )

    def rel(path: Path | None) -> str:
        return path.name if path is not None else "(not rendered)"

    lines: list[str] = []
    lines.append("---")
    lines.append("title: Ironstar build-graph report index")
    lines.append("---")
    lines.append("")
    lines.append(
        "Rendered dependency-graph artifacts for the crate2nix ironstar workspace, "
        f"generated from the committed raw derivation-show snapshot ({graph.system})."
    )
    lines.append(
        "Everything here lives under `logs/build-graph/graph-viz/`, which is gitignored; "
        "regenerate it from `logs/build-graph/raw/` with the build-graph-report app."
    )
    lines.append("")
    lines.append("## Edge semantics (read this first)")
    lines.append("")
    lines.append(
        "Two incommensurable edge relations live in the raw derivations, and each view "
        "below is built from exactly one of them."
    )
    lines.append("")
    lines.append(
        "The architecture views (`member-dag`, `crate-overview`) are the crate-to-crate "
        "dependency DAG recovered from `env.dependencies` / `env.buildDependencies` "
        f"(the crate2nix `-lib` store-path wiring): {arch_edge_count} dependency edges "
        f"over {len(graph.crates)} crate derivations, of which {member_edges} are "
        "member-to-member."
    )
    lines.append("")
    lines.append(
        "The cone views are reverse-reachability sets over `inputs.drvs` (the full "
        "transitive build closure), so a cone's node count is the rebuild set when the "
        "seed crate's version bumps. These counts are NOT comparable to the architecture "
        "edge counts: a closure-reachable node count and a dependency-edge count measure "
        "different things, and any `edges.ndjson` architecture-edge total is "
        "incommensurable with a cone size."
    )
    lines.append("")
    lines.append("## Summary table")
    lines.append("")
    lines.append("| view | relation | nodes | edges |")
    lines.append("|------|----------|-------|-------|")
    lines.append(
        f"| member-dag | architecture (env deps) | {len(graph.members)} | {member_edges} |"
    )
    lines.append(
        f"| crate-overview | architecture (env deps) | {len(graph.crates)} | {arch_edge_count} |"
    )
    for cone in cone_results:
        lines.append(
            f"| {cone.seed_short}-cone | build closure (inputs.drvs) | "
            f"{len(cone.nodes)} | {cone.edges} |"
        )
    lines.append("")
    lines.append("## Architecture views")
    lines.append("")
    arch_by_stem = {r.dot_path.stem: r for r in arch_renders}
    member = arch_by_stem.get("member-dag")
    if member is not None:
        lines.append("### member-dag")
        lines.append("")
        lines.append(
            "The 11 workspace members as a role-colored DAG (binary at top, foundations "
            "at the bottom), edges from a member to the members it directly depends on. "
            "Edge relation: the crate dependency DAG restricted to members."
        )
        lines.append("")
        lines.append(
            f"Artifacts: [{rel(member.svg)}]({rel(member.svg)}), "
            f"[{rel(member.png)}]({rel(member.png)})"
            + (f", [{rel(member.pdf)}]({rel(member.pdf)})" if member.pdf else "")
            + "."
        )
        lines.append("")
    overview = arch_by_stem.get("crate-overview")
    if overview is not None:
        lines.append("### crate-overview")
        lines.append("")
        lines.append(
            "The full crate topology as an sfdp force-directed map, nodes sized by fan-in. "
            "Members are large role-colored boxes; high-fan-in crates are sized up and "
            "labeled; the long tail is the gray fan-in-sized cloud. Edge relation: the "
            "full crate dependency DAG."
        )
        lines.append("")
        lines.append(
            f"Artifacts: [{rel(overview.svg)}]({rel(overview.svg)}), "
            f"[{rel(overview.png)}]({rel(overview.png)})"
            + (f", [{rel(overview.pdf)}]({rel(overview.pdf)})" if overview.pdf else "")
            + "."
        )
        lines.append("")
    lines.append("## Cone views (cache-story rebuild sets)")
    lines.append("")
    lines.append(
        "Each cone is the reverse-reachability set from a seed crate over the build "
        "closure: exactly the derivations that rebuild when the seed's version bumps. "
        "Convention: seed accented red at the bottom, members green, binary blue box3d "
        "at the top, intermediates gray, arrows dependency -> dependent (the bump cascades "
        "upward), a legend cluster, and transitive reduction so the rendered edges are "
        "minimal. Titles annotate the rebuild set as N of the dep crates + M of the 11 members."
    )
    lines.append("")
    for cone in cone_results:
        render = cone_renders.get(cone.seed_short)
        lines.append(f"### {cone.seed_short}-cone")
        lines.append("")
        lines.append(
            f"Seed `{cone.seed_short}` v{graph.crates[cone.seed_full].version}: "
            f"rebuild set {cone.deps_in} dep crates + {cone.members_in} members "
            f"({len(cone.nodes)} derivations, {cone.edges} reduced edges)."
        )
        if render is not None:
            lines.append("")
            lines.append(
                f"Artifacts: [{rel(render.svg)}]({rel(render.svg)}), "
                f"[{rel(render.png)}]({rel(render.png)})"
                + (f", [{rel(render.pdf)}]({rel(render.pdf)})" if render.pdf else "")
                + "."
            )
        lines.append("")
    if skipped_seeds:
        lines.append(
            "Seeds absent from this graph (skipped, not an error): "
            + ", ".join(f"`{s}`" for s in skipped_seeds)
            + "."
        )
        lines.append("")
    lines.append("## Manual nixgraph escape hatch")
    lines.append("")
    lines.append(
        "sbomnix's `nixgraph` is the off-the-shelf alternative for ad-hoc provenance "
        "queries. It is locked to `dot -Kdot` hierarchical layout, so it degrades on "
        "dense scopes. A shallow inverse query is the only safe default:"
    )
    lines.append("")
    lines.append("```bash")
    lines.append("DRV=$(nix eval --raw .#packages.${system}.ironstar-release.drvPath)")
    lines.append(
        "nix shell nixpkgs#sbomnix nixpkgs#graphviz -c \\"
    )
    lines.append(
        "  nixgraph --buildtime --inverse 'rust_zenoh' --depth 4 "
        "--colorize 'rust_ironstar' --out nixgraph-zenoh-inverse.png \"$DRV\""
    )
    lines.append("```")
    lines.append("")
    lines.append(
        "Never run `nixgraph --buildtime` at depth >= 2 over the full closure, and never "
        "run `--until` over the full closure: the hierarchical `dot` render produces a "
        "non-terminating job or a multi-megabyte max-height ribbon with zero legible nodes."
    )
    lines.append("")

    path = out_dir / "index.md"
    path.write_text("\n".join(lines) + "\n")
    return path


def main(argv: list[str]) -> int:
    repo_root = _repo_root()
    raw_dir = Path(argv[1]) if len(argv) > 1 else repo_root / "logs/build-graph/raw"
    out_dir = (
        Path(argv[2]) if len(argv) > 2 else repo_root / "logs/build-graph/graph-viz"
    )
    out_dir.mkdir(parents=True, exist_ok=True)

    dot_engine = shutil.which("dot")
    sfdp_engine = shutil.which("sfdp")
    if dot_engine is None or sfdp_engine is None:
        raise SystemExit(
            "graphviz 'dot' and 'sfdp' must be on PATH "
            "(provided via runtimeInputs = [ graphviz ] in the packaged app)"
        )

    graph = load_graph(raw_dir)
    print(
        f"loaded {len(graph.crates)} crate derivations, {len(graph.members)} members, "
        f"{len(graph.dep_edges)} dependency edges ({graph.system})"
    )

    emit_pdf = _pdf_supported(dot_engine)
    if not emit_pdf:
        print("note: dot -Tpdf probe failed; emitting SVG+PNG only", file=sys.stderr)

    created: list[Path] = []

    member_dot = write_member_dag(graph, out_dir)
    overview_dot = write_crate_overview(graph, out_dir)

    arch_renders = [
        render_dot(member_dot, dot_engine, MEMBER_DAG_DPI, emit_pdf),
        render_dot(overview_dot, sfdp_engine, CRATE_OVERVIEW_DPI, emit_pdf),
    ]
    for render in arch_renders:
        created += [p for p in (render.dot_path, render.svg, render.png, render.pdf) if p]

    cone_results: list[ConeResult] = []
    cone_renders: dict[str, RenderResult] = {}
    skipped: list[str] = []
    for seed in CONE_SEEDS:
        cone = build_cone(graph, seed, out_dir)
        if cone is None:
            skipped.append(seed)
            continue
        cone_results.append(cone)
        dpi = CONE_DPI.get(seed, CONE_DPI["_default"])
        render = render_dot(cone.dot_path, dot_engine, dpi, emit_pdf)
        cone_renders[seed] = render
        created += [p for p in (render.dot_path, render.svg, render.png, render.pdf) if p]
        print(
            f"cone {seed}: nodes={len(cone.nodes)} deps={cone.deps_in} "
            f"members={cone.members_in} edges={cone.edges}"
        )

    index_path = write_index(
        graph, out_dir, arch_renders, cone_results, cone_renders, skipped
    )
    created.append(index_path)

    print(f"wrote {len(created)} artifacts under {out_dir}")
    for path in created:
        print(f"  {path}")
    return 0


def _repo_root() -> Path:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            check=True,
            capture_output=True,
            text=True,
        )
        return Path(result.stdout.strip())
    except (subprocess.CalledProcessError, FileNotFoundError):
        return Path.cwd()


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
