#!/usr/bin/env bash
# deploy-sites.sh - Parametric Cloudflare Workers deploy.
#
# Usage:
#   deploy-sites <pkgName> preview <branch>
#   deploy-sites <pkgName> production
#
# pkgName ∈ { ironstar-docs, ironstar-eventcatalog }.
#
# Routes to the appropriate eval-time-injected nix payload via env vars:
#   IRONSTAR_DOCS_PAYLOAD             ironstar-docs derivation outPath
#   IRONSTAR_EVENTCATALOG_PAYLOAD     ironstar-eventcatalog derivation outPath
#
# Required env (CI):
#   CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID
#
# This is a placeholder skeleton. Full implementation follows the vanixiets
# modules/apps/docs/deploy.sh pattern: materialise a writable copy of the
# nix payload, then invoke wrangler against the appropriate config target.
# Acceptance criterion at obh.11 is locally-verified: nix eval of
# .program returns a /nix/store/... path. Runtime exercise is gated on
# the magnetite + buildbot-effects loop landing post obh.26+obh.27 + restart.

set -euo pipefail

usage() {
  cat <<'EOF'
deploy-sites — Cloudflare Workers deploy for ironstar sites

Usage:
  deploy-sites <pkgName> preview <branch>
  deploy-sites <pkgName> production

Where:
  pkgName      one of: ironstar-docs, ironstar-eventcatalog

Env (CI):
  CLOUDFLARE_API_TOKEN
  CLOUDFLARE_ACCOUNT_ID
EOF
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

pkg_name="$1"
shift

case "$pkg_name" in
  ironstar-docs)
    : "${IRONSTAR_DOCS_PAYLOAD:?IRONSTAR_DOCS_PAYLOAD not set; deploy-sites.nix must inject the nix-built payload}"
    payload="$IRONSTAR_DOCS_PAYLOAD"
    ;;
  ironstar-eventcatalog)
    : "${IRONSTAR_EVENTCATALOG_PAYLOAD:?IRONSTAR_EVENTCATALOG_PAYLOAD not set; deploy-sites.nix must inject the nix-built payload}"
    payload="$IRONSTAR_EVENTCATALOG_PAYLOAD"
    ;;
  -h | --help)
    usage
    exit 0
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

if [[ $# -lt 1 ]]; then
  echo "error: missing deploy mode (preview|production)" >&2
  usage >&2
  exit 2
fi

mode="$1"
shift

case "$mode" in
  preview)
    if [[ $# -lt 1 ]]; then
      echo "error: preview mode requires <branch>" >&2
      exit 2
    fi
    branch="$1"
    echo "[deploy-sites] would deploy $pkg_name (preview, branch=$branch) from $payload"
    ;;
  production)
    echo "[deploy-sites] would deploy $pkg_name (production) from $payload"
    ;;
  *)
    echo "error: unknown mode '$mode'; expected preview or production" >&2
    usage >&2
    exit 2
    ;;
esac

# Full wrangler invocation deferred to integration-verified phase.
# See vanixiets modules/apps/docs/deploy.sh for the canonical
# materialise-and-invoke pattern.
