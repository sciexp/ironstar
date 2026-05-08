#!/usr/bin/env bash
# shellcheck shell=bash
# release-packages.sh — Parametric semantic-release wrapper.
#
# Dispatches on argv $1 (pkgName) to run semantic-release for one of:
#   ironstar-docs            (tag prefix: @ironstar/docs-v*)
#   ironstar-eventcatalog    (tag prefix: @ironstar/eventcatalog-v*)
#
# The package.json `name` field is `@ironstar/<dirname>` and
# semantic-release-monorepo's default tagFormat is `${name}-v${version}`,
# so the live tag prefix is `@ironstar/<dirname>-v` (NOT `ironstar-<dirname>-v`,
# which is the argv vocabulary). The pkg_dirname → tag_prefix mapping lives
# in the per-package dispatch case below.
#
# Required (config, injected by release-packages.nix):
#   IRONSTAR_DOCS_PAYLOAD              ironstar-docs derivation outPath
#                                      (currently unused by the release path;
#                                      retained for parity with deploy-sites
#                                      and future post-release artifact use).
#   IRONSTAR_EVENTCATALOG_PAYLOAD      ironstar-eventcatalog derivation outPath
#                                      (likewise retained for parity).
#   IRONSTAR_DOCS_NODE_MODULES         ironstar-docs-deps node_modules tree
#                                      hosting node_modules/.bin/semantic-release.
#   IRONSTAR_EVENTCATALOG_NODE_MODULES ironstar-eventcatalog-deps node_modules
#                                      tree hosting .bin/semantic-release.
#
# Required (secret, production path only — not --dry-run):
#   GITHUB_TOKEN         @semantic-release/github auth for tag push and
#                        release publish. Filtered-out plugin list under
#                        --dry-run means no token is consulted in that mode.
#
# Optional (CI-mode signalling; required by env-ci on the effect path):
#   CI                   "true" tells semantic-release / env-ci that the
#                        run is non-interactive CI. Required in the
#                        buildbot-effects bwrap sandbox (not a recognised
#                        CI provider; semantic-release would otherwise
#                        abort `running on a CI environment is required`).
#
# Optional (repo-root resolution; env-first with errexit-tolerant fallback):
#   RELEASE_REPO_ROOT    absolute path to the working tree's repo root.
#                        Required in the bwrap sandbox (no .git bind-mount;
#                        `git rev-parse --show-toplevel` would fail).
#                        Fallback: git rev-parse --show-toplevel || pwd.
#
# Optional (git identity; env-first, NO .git/config writes — bwrap mounts
# /nix/store ro-bind, so `git config user.email …` would fail to lock
# .git/config). git honours these natively without any config write.
# Defaults applied if unset:
#   GIT_AUTHOR_NAME      / GIT_AUTHOR_EMAIL    (semantic-release@ironstar.local)
#   GIT_COMMITTER_NAME   / GIT_COMMITTER_EMAIL (semantic-release@ironstar.local)

set -euo pipefail

usage() {
  cat <<'EOF'
usage: release-packages <pkgName> [--dry-run] [-- extra semantic-release args]
       release-packages info <pkgName>
       release-packages --help

Run semantic-release against an ironstar monorepo package, or extract
release info as JSON.

Where:
  <pkgName>   one of: ironstar-docs, ironstar-eventcatalog

Subcommands:
  (default)   Run semantic-release for <pkgName>. With --dry-run, the
              @semantic-release/github plugin is filtered out so
              GITHUB_TOKEN is not required for previews.
  info        Emit release info JSON (version, tag, released) from the
              latest git tag matching the package's @ironstar/<dirname>-v*
              prefix.

Flags:
  --dry-run   Dry-run (skips @semantic-release/github; no GITHUB_TOKEN needed).
  --help, -h  Print this usage and exit 0.

Environment contract (see top-of-file header for full details):
  Required (secret, production path only):
    GITHUB_TOKEN
  Required (config, injected by release-packages.nix):
    IRONSTAR_DOCS_PAYLOAD,             IRONSTAR_DOCS_NODE_MODULES
    IRONSTAR_EVENTCATALOG_PAYLOAD,     IRONSTAR_EVENTCATALOG_NODE_MODULES
  Optional:
    CI, RELEASE_REPO_ROOT,
    GIT_AUTHOR_NAME, GIT_AUTHOR_EMAIL,
    GIT_COMMITTER_NAME, GIT_COMMITTER_EMAIL

Examples:
  nix run .#release-packages -- ironstar-docs --dry-run
  nix run .#release-packages -- ironstar-eventcatalog
  nix run .#release-packages -- info ironstar-docs
EOF
}

# --- argv preflight -------------------------------------------------------

if [[ $# -lt 1 ]]; then
  echo "error: missing pkgName" >&2
  usage >&2
  exit 2
fi

# Subcommand dispatch: `info <pkgName>` is a separate code path.
mode="release"
case "$1" in
  -h | --help)
    usage
    exit 0
    ;;
  info)
    mode="info"
    shift
    if [[ $# -lt 1 ]]; then
      echo "error: info requires <pkgName>" >&2
      usage >&2
      exit 2
    fi
    ;;
esac

pkg_name="$1"
shift

# --- per-package dispatch -------------------------------------------------
#
# Resolve the payload, node_modules tree, monorepo subpath, and
# package.json name (= tag prefix root) up front. The tag prefix matches
# semantic-release-monorepo's default tagFormat `${name}-v${version}`
# applied to the package.json `name` field, which is `@ironstar/<dirname>`
# for both packages.

case "$pkg_name" in
  ironstar-docs)
    : "${IRONSTAR_DOCS_PAYLOAD:?IRONSTAR_DOCS_PAYLOAD not set; release-packages.nix must inject the nix-built payload}"
    : "${IRONSTAR_DOCS_NODE_MODULES:?IRONSTAR_DOCS_NODE_MODULES not set; release-packages.nix must expose ironstar-docs-deps via runtimeEnv}"
    payload="$IRONSTAR_DOCS_PAYLOAD"
    node_modules_path="$IRONSTAR_DOCS_NODE_MODULES"
    pkg_dirname="docs"
    pkg_json_name="@ironstar/docs"
    ;;
  ironstar-eventcatalog)
    : "${IRONSTAR_EVENTCATALOG_PAYLOAD:?IRONSTAR_EVENTCATALOG_PAYLOAD not set; release-packages.nix must inject the nix-built payload}"
    : "${IRONSTAR_EVENTCATALOG_NODE_MODULES:?IRONSTAR_EVENTCATALOG_NODE_MODULES not set; release-packages.nix must expose ironstar-eventcatalog-deps via runtimeEnv}"
    payload="$IRONSTAR_EVENTCATALOG_PAYLOAD"
    node_modules_path="$IRONSTAR_EVENTCATALOG_NODE_MODULES"
    pkg_dirname="eventcatalog"
    pkg_json_name="@ironstar/eventcatalog"
    ;;
  *)
    echo "error: unknown pkgName '$pkg_name'; expected ironstar-docs or ironstar-eventcatalog" >&2
    usage >&2
    exit 2
    ;;
esac

[[ -d "$payload" ]] || {
  echo "error: payload path '$payload' is not a directory" >&2
  exit 1
}

package_path="packages/${pkg_dirname}"
tag_glob="${pkg_json_name}-v*"

# --- info subcommand ------------------------------------------------------
#
# Reads the latest git tag matching the package's tagFormat prefix and
# emits {version, tag, released} JSON. Mirrors vanixiets emit_release_info
# (release.sh:60-88) but with the ironstar package.json `name` substituted
# for the basename-derived prefix, since semantic-release-monorepo uses
# the full package.json name as the default tagFormat root.

emit_release_info() {
  local latest_tag=""
  local version=""

  # Resolve repo root for git tag query; same env-first / git-fallback
  # cascade as the release path below.
  local query_root="${RELEASE_REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

  latest_tag=$(git -C "$query_root" tag --list "$tag_glob" --sort=-v:refname 2>/dev/null | head -1 || true)

  if [ -n "$latest_tag" ]; then
    version=$(printf '%s\n' "$latest_tag" \
      | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z]+\.[0-9]+)?' \
      | head -1 || true)
    if [ -z "$version" ]; then
      version="unknown"
    fi
    jq -cn \
      --arg pkg "$pkg_name" \
      --arg v "$version" \
      --arg t "$latest_tag" \
      '{pkgName: $pkg, version: $v, tag: $t, released: true}'
  else
    jq -cn \
      --arg pkg "$pkg_name" \
      '{pkgName: $pkg, version: "unknown", tag: "", released: false}'
  fi
}

if [ "$mode" = "info" ]; then
  emit_release_info
  exit 0
fi

# --- release path: argv parsing -------------------------------------------

dry_run=0
extra_args=()

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run=1
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    --)
      shift
      extra_args+=("$@")
      break
      ;;
    *)
      extra_args+=("$1")
      shift
      ;;
  esac
done

# --- repo root + cd into package -----------------------------------------
#
# Repo-root resolution: env-first, then error-tolerant git fallback, then
# pwd. Required because the buildbot-effects bwrap sandbox does not bind-
# mount the working tree's .git, so `git rev-parse --show-toplevel` would
# fail with `fatal: not a git repository` (exit 128) and abort the script.
# The effect preamble sets RELEASE_REPO_ROOT="$clone_dir" so this branch
# resolves without invoking git. Local-shell callers leave RELEASE_REPO_ROOT
# unset, exercising the git fallback against the live worktree.

repo_root="${RELEASE_REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$repo_root"

if [ ! -d "$package_path" ]; then
  printf 'error: package path %q does not exist relative to %s\n' \
    "$package_path" "$repo_root" >&2
  exit 1
fi

# Git identity: exported via GIT_AUTHOR_* / GIT_COMMITTER_* env vars rather
# than written to .git/config. Required because the buildbot-effects bwrap
# sandbox renders .git read-only (mounts /nix/store ro-bind only) and
# `git config user.email "…"` would fail with `error: could not lock config
# file .git/config`. git honours these env vars natively without any config
# write. Each export uses parameter-expansion default chaining so a pre-set
# value (effect preamble or caller env) is preserved unchanged.
export GIT_AUTHOR_NAME="${GIT_AUTHOR_NAME:-semantic-release}"
export GIT_AUTHOR_EMAIL="${GIT_AUTHOR_EMAIL:-semantic-release@ironstar.local}"
export GIT_COMMITTER_NAME="${GIT_COMMITTER_NAME:-semantic-release}"
export GIT_COMMITTER_EMAIL="${GIT_COMMITTER_EMAIL:-semantic-release@ironstar.local}"

cd "$package_path"

# Production-path contract guard: fail fast on missing GITHUB_TOKEN
# BEFORE any node_modules mutation so the error points at the contract
# rather than at an opaque state-mutation side effect. Gated on dry_run.
if [ "$dry_run" -ne 1 ]; then
  : "${GITHUB_TOKEN:?GITHUB_TOKEN is required for production semantic-release (see release-packages.sh header for caller mechanisms; not needed for --dry-run)}"
fi

# --- node_modules slot management ----------------------------------------
#
# Guard node_modules slot against clobbering a developer's real install.
# Production (non-dry-run): strict — refuse to overwrite a real node_modules
# directory. Only an empty slot or a pre-existing symlink is safe to clobber.
# Dry-run: proceed safely via two strategies that NEVER mutate the
# developer's real install in place:
#   (b) reuse the existing node_modules directly if it already contains a
#       usable semantic-release binary (common when the dev ran
#       `bun install` to completion), or
#   (a) move the existing node_modules aside to a tempdir, symlink
#       $node_modules_path in its place for the duration of the run, and
#       atomically restore the original on EXIT (including on error/SIGINT).

nm_exists_real=0
if [[ -e node_modules && ! -L node_modules ]]; then
  nm_exists_real=1
fi

if [ "$nm_exists_real" -eq 1 ] && [ "$dry_run" -ne 1 ]; then
  echo "error: $package_path/node_modules exists and is not a symlink; refusing to overwrite a local bun install" >&2
  exit 1
fi

if [ "$nm_exists_real" -eq 1 ] && [ -x node_modules/.bin/semantic-release ]; then
  # Dry-run strategy (b): reuse existing node_modules in place.
  echo "dry-run: reusing existing node_modules (.bin/semantic-release present)" >&2
elif [ "$nm_exists_real" -eq 1 ]; then
  # Dry-run strategy (a): move existing node_modules aside, symlink for the
  # duration of the run, restore atomically on exit.
  backup_dir="$(mktemp -d)"
  echo "dry-run: moving existing node_modules to ${backup_dir} (restored on exit)" >&2
  mv node_modules "${backup_dir}/node_modules"
  # shellcheck disable=SC2064
  trap "rm -f '${PWD}/node_modules'; mv '${backup_dir}/node_modules' '${PWD}/node_modules' 2>/dev/null || true; rmdir '${backup_dir}' 2>/dev/null || true" EXIT
  ln -snf "$node_modules_path" node_modules
else
  # Slot is empty or already a symlink — safe to (re)link.
  trap 'rm -f "$PWD/node_modules"' EXIT
  ln -snf "$node_modules_path" node_modules
fi

# --- semantic-release invocation -----------------------------------------

if [ "$dry_run" -eq 1 ]; then
  # Filter @semantic-release/github so GITHUB_TOKEN is not required for
  # preview; safe under --dry-run (prepare/publish steps are no-ops).
  # Mirrors the package.json "release" plugin block minus the github
  # plugin: commit-analyzer + release-notes-generator + changelog +
  # semantic-release-major-tag.
  plugins="@semantic-release/commit-analyzer,@semantic-release/release-notes-generator,@semantic-release/changelog,semantic-release-major-tag"
  echo "running semantic-release (dry-run, no GitHub plugin) in ${package_path}..."
  node ./node_modules/.bin/semantic-release \
    --dry-run \
    --no-ci \
    --plugins "$plugins" \
    "${extra_args[@]}"
else
  # GITHUB_TOKEN is enforced via the early :? guard above (placed before
  # node_modules setup so failure modes are contract-first).
  echo "running production semantic-release in ${package_path}..."
  node ./node_modules/.bin/semantic-release "${extra_args[@]}"
fi
