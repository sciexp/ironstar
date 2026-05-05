#!/usr/bin/env bash
# release-packages.sh - Parametric semantic-release wrapper.
#
# Usage:
#   release-packages <pkgName>
#   release-packages <pkgName> --dry-run
#   release-packages info <pkgName>
#   release-packages --help
#
# pkgName ∈ { ironstar-docs, ironstar-eventcatalog }.
#
# Routes to the appropriate eval-time-injected nix payload via env vars:
#   IRONSTAR_DOCS_PAYLOAD             ironstar-docs derivation outPath
#   IRONSTAR_EVENTCATALOG_PAYLOAD     ironstar-eventcatalog derivation outPath
#
# Required env (CI):
#   GITHUB_TOKEN, CI, RELEASE_REPO_ROOT,
#   GIT_AUTHOR_NAME, GIT_AUTHOR_EMAIL,
#   GIT_COMMITTER_NAME, GIT_COMMITTER_EMAIL.
#
# This is a placeholder skeleton. Full implementation follows the vanixiets
# modules/apps/release/release.sh pattern: configure git identity, link the
# nix-provided node_modules into the package directory, invoke
# semantic-release with the appropriate config, filter
# @semantic-release/github plugin when --dry-run, and emit JSON for
# `info <pkgName>`. Acceptance criterion at obh.11 is locally-verified:
# nix eval of .program returns a /nix/store/... path. Runtime exercise is
# gated on the magnetite + buildbot-effects loop landing post obh.26+obh.27
# + restart.

set -euo pipefail

usage() {
  cat <<'EOF'
release-packages — semantic-release wrapper for ironstar packages

Usage:
  release-packages <pkgName>
  release-packages <pkgName> --dry-run
  release-packages info <pkgName>
  release-packages --help

Where:
  pkgName      one of: ironstar-docs, ironstar-eventcatalog

Env (CI):
  GITHUB_TOKEN, CI, RELEASE_REPO_ROOT,
  GIT_AUTHOR_NAME, GIT_AUTHOR_EMAIL,
  GIT_COMMITTER_NAME, GIT_COMMITTER_EMAIL
EOF
}

if [[ $# -lt 1 ]]; then
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
      exit 2
    fi
    ;;
esac

pkg_name="$1"
shift

case "$pkg_name" in
  ironstar-docs)
    : "${IRONSTAR_DOCS_PAYLOAD:?IRONSTAR_DOCS_PAYLOAD not set; release-packages.nix must inject the nix-built payload}"
    payload="$IRONSTAR_DOCS_PAYLOAD"
    ;;
  ironstar-eventcatalog)
    : "${IRONSTAR_EVENTCATALOG_PAYLOAD:?IRONSTAR_EVENTCATALOG_PAYLOAD not set; release-packages.nix must inject the nix-built payload}"
    payload="$IRONSTAR_EVENTCATALOG_PAYLOAD"
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

dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
  esac
done

case "$mode" in
  info)
    jq -n \
      --arg pkg "$pkg_name" \
      --arg payload "$payload" \
      '{ pkgName: $pkg, payload: $payload, status: "placeholder" }'
    ;;
  release)
    if [[ "$dry_run" -eq 1 ]]; then
      echo "[release-packages] would dry-run semantic-release for $pkg_name (payload=$payload)"
    else
      echo "[release-packages] would invoke semantic-release for $pkg_name (payload=$payload)"
    fi
    ;;
esac

# Full semantic-release invocation deferred to integration-verified phase.
# See vanixiets modules/apps/release/release.sh for the canonical
# git-config + node_modules link + semantic-release pattern.
