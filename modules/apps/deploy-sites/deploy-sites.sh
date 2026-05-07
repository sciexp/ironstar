#!/usr/bin/env bash
# shellcheck shell=bash
# deploy-sites.sh — Parametric Cloudflare Workers deploy.
#
# Dispatches on argv $1 (pkgName) to deploy one of:
#   ironstar-docs            (preview alias: b-<branch>-ironstar-docs.sciexp.workers.dev)
#   ironstar-eventcatalog    (preview alias: b-<branch>-ironstar-events.sciexp.workers.dev)
#                            (worker name from packages/eventcatalog/wrangler.jsonc is
#                             ironstar-events, NOT ironstar-eventcatalog)
#
# Required (config, injected by deploy-sites.nix):
#   IRONSTAR_DOCS_PAYLOAD              ironstar-docs derivation outPath
#                                      ($out/{*.html, _astro/, _worker.js/, wrangler.json}).
#   IRONSTAR_EVENTCATALOG_PAYLOAD      ironstar-eventcatalog derivation outPath
#                                      ($out/{*.html, _astro/, ..., wrangler.json}).
#   IRONSTAR_DOCS_NODE_MODULES         ironstar-docs-deps node_modules tree.
#   IRONSTAR_EVENTCATALOG_NODE_MODULES ironstar-eventcatalog-deps node_modules tree.
#
# Required (secret, caller-provided):
#   CLOUDFLARE_API_TOKEN     wrangler auth token (CONSUMED).
#   CLOUDFLARE_ACCOUNT_ID    Cloudflare account id (CONSUMED; account-scoped ops).
#
# Optional (env-first with git-fallback): every GIT_* consumer is
# `${GIT_X:-$(git … 2>/dev/null || true)}` so the script runs both inside the
# buildbot-effects bwrap sandbox (no .git bind-mounted; env pre-populated by
# the effect preamble) and from a live worktree (env unset; git fallback
# resolves locally):
#   GIT_REV, GIT_REV_SHORT, GIT_REV_SHORT12, GIT_BRANCH, GIT_COMMIT_MSG,
#   GIT_WORKTREE_STATUS.
# Optional (env-first with bash-builtin / shelled-fallback): the bwrap
# sandbox lacks `hostname`/`whoami` on PATH, so DEPLOY_HOST falls back to
# `${HOSTNAME%%.*}` (bash builtin populated from gethostname(2)) and
# DEPLOY_DEPLOYER falls back to GITHUB_ACTOR → `whoami 2>/dev/null` →
# "unknown".
# Optional (caller debugging / overrides):
#   WRANGLER, DEPLOY_SITES_DEBUG, GITHUB_ACTIONS / GITHUB_ACTOR /
#   GITHUB_WORKFLOW (when GITHUB_ACTIONS is set, the production deploy
#   message uses GITHUB_WORKFLOW (default "CI") as deploy context instead
#   of DEPLOY_HOST).

set -euo pipefail

usage() {
  cat <<'EOF'
usage: deploy-sites <pkgName> preview <branch>
       deploy-sites <pkgName> production
       deploy-sites --help

Where:
  <pkgName>   one of: ironstar-docs, ironstar-eventcatalog

Deploy the nix-built ironstar site payload to Cloudflare Workers.

Subcommands:
  preview <branch>   Upload a Cloudflare Workers preview version tagged with
                     the current HEAD short SHA, aliased at b-<sanitized-branch>.
                     <branch> defaults to `git branch --show-current`; explicit
                     value required when HEAD is detached.
  production         Promote the existing preview version matching the current
                     HEAD short SHA to 100% production traffic, or fall back to
                     a direct deploy of the nix-built payload when no matching
                     preview exists.

Flags:
  --help, -h         Print this usage and exit 0.

Environment contract (see top-of-file header for full details):
  Required (secret, caller-provided):
    CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID
  Required (config, injected by deploy-sites.nix):
    IRONSTAR_DOCS_PAYLOAD,             IRONSTAR_DOCS_NODE_MODULES
    IRONSTAR_EVENTCATALOG_PAYLOAD,     IRONSTAR_EVENTCATALOG_NODE_MODULES
  Optional (env-first with shelled-fallback):
    GIT_REV, GIT_REV_SHORT, GIT_REV_SHORT12, GIT_BRANCH, GIT_COMMIT_MSG,
    GIT_WORKTREE_STATUS, DEPLOY_HOST, DEPLOY_DEPLOYER
  Optional (caller debugging / overrides):
    WRANGLER, DEPLOY_SITES_DEBUG
    GITHUB_ACTIONS / GITHUB_ACTOR / GITHUB_WORKFLOW

Examples:
  nix run .#deploy-sites -- ironstar-docs preview my-feature-branch
  nix run .#deploy-sites -- ironstar-eventcatalog production
EOF
}

# --- argv preflight -------------------------------------------------------

case "${1:-}" in
  -h | --help)
    usage
    exit 0
    ;;
esac

if [[ $# -lt 1 ]]; then
  echo "error: missing pkgName" >&2
  usage >&2
  exit 2
fi

pkg_name="$1"
shift

# --- per-package dispatch -------------------------------------------------
#
# Resolve the payload, node_modules tree, and per-package preview-URL hostname
# fragment up front. The preview-URL fragment matches the worker `name` field
# in each package's wrangler.jsonc:
#   - ironstar-docs           → ironstar-docs.sciexp.workers.dev
#   - ironstar-eventcatalog   → ironstar-events.sciexp.workers.dev
# The Cloudflare account-subdomain (sciexp) is hardcoded to match
# packages/{docs,eventcatalog}/wrangler.jsonc and the existing justfile
# (lines 515 and 854); ironstar shares the sciexp Cloudflare account with
# the vanixiets reference.

case "$pkg_name" in
  ironstar-docs)
    : "${IRONSTAR_DOCS_PAYLOAD:?IRONSTAR_DOCS_PAYLOAD not set; deploy-sites.nix must inject the nix-built payload}"
    : "${IRONSTAR_DOCS_NODE_MODULES:?IRONSTAR_DOCS_NODE_MODULES not set; deploy-sites.nix must expose ironstar-docs-deps via runtimeEnv}"
    payload="$IRONSTAR_DOCS_PAYLOAD"
    node_modules_path="$IRONSTAR_DOCS_NODE_MODULES"
    preview_host_segment="ironstar-docs"
    production_url="https://ironstar.scientistexperience.net"
    ;;
  ironstar-eventcatalog)
    : "${IRONSTAR_EVENTCATALOG_PAYLOAD:?IRONSTAR_EVENTCATALOG_PAYLOAD not set; deploy-sites.nix must inject the nix-built payload}"
    : "${IRONSTAR_EVENTCATALOG_NODE_MODULES:?IRONSTAR_EVENTCATALOG_NODE_MODULES not set; deploy-sites.nix must expose ironstar-eventcatalog-deps via runtimeEnv}"
    payload="$IRONSTAR_EVENTCATALOG_PAYLOAD"
    node_modules_path="$IRONSTAR_EVENTCATALOG_NODE_MODULES"
    preview_host_segment="ironstar-events"
    production_url="https://events.ironstar.scientistexperience.net"
    ;;
  *)
    echo "error: unknown pkgName '$pkg_name'; expected ironstar-docs or ironstar-eventcatalog" >&2
    usage >&2
    exit 2
    ;;
esac

[[ -d "$payload" ]] || { echo "error: payload path '$payload' is not a directory" >&2; exit 1; }

if [[ $# -lt 1 ]]; then
  echo "error: missing subcommand" >&2
  echo "usage: deploy-sites $pkg_name preview <branch> | deploy-sites $pkg_name production" >&2
  echo "(run with --help for full usage and env-var contract)" >&2
  exit 2
fi

mode="$1"
shift

# --- secret env preflight -------------------------------------------------

: "${CLOUDFLARE_API_TOKEN:?CLOUDFLARE_API_TOKEN is required}"
: "${CLOUDFLARE_ACCOUNT_ID:?CLOUDFLARE_ACCOUNT_ID is required}"

# --- hermetic wrangler binary --------------------------------------------
#
# Resolve wrangler from the per-package deps derivation node_modules tree.
# WRANGLER override allows test harnesses to substitute a no-op stub.
export WRANGLER="${WRANGLER:-$node_modules_path/.bin/wrangler}"

# Invoke wrangler via real node, not the .bin/wrangler shebang: bun's .bin
# wrappers point at bun-with-fake-node/bin/node (bun in node-compat mode),
# but bun's fetch() on linux-x64 silently hangs on keep-alive connection
# reuse to api.cloudflare.com — wrangler `versions upload` / `versions
# deploy` exit 0 with no Worker Version ID produced and no error. Prefixing
# `node` forces real-node (undici) runtime.

# --- materialise writable copy -------------------------------------------
#
# wrangler reads the config file at WRANGLER_CONFIG and may write state to
# .wrangler/ during deploy, so the nix-built payload must be copied out of
# the read-only store into a writable tmpdir before invocation.
tmpdir=$(mktemp -d -t deploy-sites.XXXXXX)
if [[ -n "${DEPLOY_SITES_DEBUG:-}" ]]; then
  echo "[deploy-sites] DEBUG: preserving tmpdir at $tmpdir" >&2
  trap 'echo "[deploy-sites] DEBUG: tmpdir preserved at '\''$tmpdir'\''" >&2' EXIT
else
  trap 'rm -rf "$tmpdir"' EXIT
fi
cp -R "$payload"/. "$tmpdir/"
chmod -R u+w "$tmpdir"

# The payload derivations bake a normalized wrangler.json at the payload
# root (see pkgs/by-name/ironstar-docs/package.nix and
# pkgs/by-name/ironstar-eventcatalog/package.nix install phases). All paths
# inside that wrangler.json are root-relative so they resolve correctly
# against the materialised tmpdir copy.
wrangler_config="$tmpdir/wrangler.json"
[[ -f "$wrangler_config" ]] || {
  echo "error: payload '$payload' does not contain wrangler.json" >&2
  echo "       expected at \$out/wrangler.json from pkgs/by-name/$pkg_name/package.nix install phase" >&2
  exit 1
}

# --- commit metadata ------------------------------------------------------
#
# Env-first with errexit-tolerant git fallback so a missing .git (bwrap
# sandbox) surfaces as empty strings rather than aborting; the env-first
# path supplies authoritative values in that case.
commit_sha="${GIT_REV:-$(git rev-parse HEAD 2>/dev/null || true)}"
commit_tag="${GIT_REV_SHORT12:-$(git rev-parse --short=12 HEAD 2>/dev/null || true)}"
commit_short="${GIT_REV_SHORT:-$(git rev-parse --short HEAD 2>/dev/null || true)}"
current_branch="${GIT_BRANCH:-$(git branch --show-current 2>/dev/null || true)}"

# Bash builtin `$HOSTNAME` is populated from gethostname(2) at shell startup,
# so `${HOSTNAME%%.*}` mimics `hostname -s` without shelling out — required
# because the bwrap sandbox lacks `hostname` on PATH.
deploy_host="${DEPLOY_HOST:-${HOSTNAME%%.*}}"
deployer="${DEPLOY_DEPLOYER:-${GITHUB_ACTOR:-$(whoami 2>/dev/null || echo unknown)}}"

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  deploy_context="${GITHUB_WORKFLOW:-CI}"
  deploy_msg="Deployed by ${deployer} from ${current_branch} via ${deploy_context}"
else
  deploy_msg="Deployed by ${deployer} from ${current_branch} on ${deploy_host}"
fi

case "$mode" in
  preview)
    branch="${1:-${current_branch:-}}"
    if [[ -z "$branch" ]]; then
      echo "error: preview requires a <branch> argument" >&2
      echo "usage: deploy-sites $pkg_name preview <branch>" >&2
      exit 2
    fi

    safe_branch=$(echo "$branch" \
      | tr '/' '-' \
      | tr -c 'a-zA-Z0-9-' '-' \
      | sed 's/--*/-/g; s/^-//; s/-$//' \
      | cut -c1-40)

    commit_msg="${GIT_COMMIT_MSG:-$(git log -1 --pretty=format:'%s' 2>/dev/null || true)}"
    if [[ -n "${GIT_WORKTREE_STATUS:-}" ]]; then
      git_status="$GIT_WORKTREE_STATUS"
    elif git diff-index --quiet HEAD -- 2>/dev/null; then
      git_status="clean"
    else
      # Non-zero from `git diff-index` covers both "dirty worktree" and
      # "not a git repository" — collapse both to "dirty" so downstream
      # version_message is always well-formed.
      git_status="dirty"
    fi
    version_message="[${branch}] ${commit_msg} (${commit_tag}, ${git_status})"

    echo "Deploying preview for $pkg_name on branch: ${branch}"
    echo "Sanitized alias: b-${safe_branch}"
    echo "Commit: ${commit_short} (${git_status})"
    echo "Full SHA: ${commit_sha}"
    echo "Tag: ${commit_tag}"
    echo "Message: ${commit_msg}"
    echo ""

    export VERSION_TAG="$commit_tag"
    export VERSION_MESSAGE="$version_message"
    export SAFE_BRANCH="$safe_branch"
    export WRANGLER_CONFIG="$wrangler_config"

    # Capture wrangler's machine-readable NDJSON event log via
    # WRANGLER_OUTPUT_FILE_PATH. wrangler 4.x does NOT accept --json on
    # `versions upload`; the NDJSON stream is the authoritative
    # machine-readable output channel. For `versions upload` we look for
    # the `version-upload` event, which carries `version_id`.
    #
    # Three post-conditions enforce the no-silent-success invariant:
    #   (a) the NDJSON event log contains a `type == "version-upload"` entry
    #       with a non-empty `version_id` (primary authoritative source)
    #   (b) `wrangler versions list --json` contains an entry whose
    #       annotations["workers/tag"] matches $commit_tag (server-side
    #       persistence cross-check)
    #   (c) only then is the user-visible success block echoed, including
    #       the authoritative Worker Version ID parsed from (a).
    wrangler_upload_ndjson="$tmpdir/wrangler-versions-upload.ndjson"
    wrangler_upload_stdout="$tmpdir/wrangler-versions-upload.stdout"
    wrangler_upload_stderr="$tmpdir/wrangler-versions-upload.stderr"
    : > "$wrangler_upload_ndjson"
    : > "$wrangler_upload_stdout"
    : > "$wrangler_upload_stderr"
    export WRANGLER_OUTPUT_FILE_PATH="$wrangler_upload_ndjson"

    printf '>> wrangler upload command: node %s --config %s versions upload --preview-alias %s --tag %s --message %q\n' \
      "$WRANGLER" "$WRANGLER_CONFIG" "b-${SAFE_BRANCH}" "$VERSION_TAG" "$VERSION_MESSAGE" >&2

    set +e
    node "$WRANGLER" --config "$WRANGLER_CONFIG" versions upload \
        --preview-alias "b-${SAFE_BRANCH}" \
        --tag "$VERSION_TAG" \
        --message "$VERSION_MESSAGE" \
      > >(tee "$wrangler_upload_stdout") \
      2> >(tee "$wrangler_upload_stderr" >&2)
    wrangler_upload_rc=$?
    set -e

    unset WRANGLER_OUTPUT_FILE_PATH

    # Post-condition (a): extract a non-empty Worker Version ID. Primary:
    # NDJSON `version-upload` event. Fallback: stdout line
    # `Worker Version ID: <uuid>`.
    version_id=""
    if [[ -s "$wrangler_upload_ndjson" ]]; then
      version_id=$(
        jq -rs '
          map(select(type == "object" and (.type // "") == "version-upload"))
          | .[0].version_id // empty
        ' "$wrangler_upload_ndjson" 2>/dev/null || true
      )
    fi
    if [[ -z "$version_id" ]]; then
      version_id=$(
        grep -oE 'Worker Version ID: [a-f0-9-]+' "$wrangler_upload_stdout" 2>/dev/null \
          | awk '{print $NF}' \
          | head -1 || true
      )
    fi
    if [[ -z "$version_id" ]]; then
      # Relax errexit for the diagnostic dump block. grep/sed/cat/head
      # failures here (missing stdout match, empty NDJSON, nonexistent log
      # file) must not abort before every dump section fires — the script's
      # fail contract is satisfied by the explicit `exit 1` at the end of
      # this block, not by intermediate pipeline exit codes.
      set +e
      echo "" >&2
      echo "error: wrangler exited 0 but produced no Worker Version ID" >&2
      echo "  post-condition (a) failed: neither WRANGLER_OUTPUT_FILE_PATH NDJSON" >&2
      echo "                              event log nor wrangler stdout contained" >&2
      echo "                              a recognizable Worker Version ID" >&2
      echo "  wrangler exit code:    $wrangler_upload_rc" >&2
      echo "  raw wrangler event log: $wrangler_upload_ndjson" >&2
      echo "  raw wrangler stdout:   $wrangler_upload_stdout" >&2
      echo "  raw wrangler stderr:   $wrangler_upload_stderr" >&2
      echo "  hints:" >&2
      echo "    - confirm CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID are exported by" >&2
      echo "      the caller (effect preamble, direnv, sops wrapper, or GHA env)" >&2
      echo "    - if linux-x64 regression, confirm wrangler invoked under real node and" >&2
      echo "      not bun-fake-node (see deploy-sites.sh node invocation rationale)" >&2
      echo "    - inspect the wrangler internal log dumped below / raw NDJSON and stdout paths above for any output" >&2
      echo "" >&2
      wrangler_log_path=""
      for candidate_dir in "$HOME/.wrangler/logs" "$HOME/.config/.wrangler/logs"; do
        if [[ -d "$candidate_dir" ]]; then
          newest=$(find "$candidate_dir" -maxdepth 1 -type f -name 'wrangler-*.log' 2>/dev/null | sort | tail -1 || true)
          if [[ -n "$newest" ]]; then
            wrangler_log_path="$newest"
            break
          fi
        fi
      done
      if [[ -n "$wrangler_log_path" && -f "$wrangler_log_path" ]]; then
        echo "--- begin wrangler internal log ($wrangler_log_path) ---" >&2
        cat "$wrangler_log_path" >&2 || true
        echo "--- end wrangler internal log ---" >&2
      else
        echo "wrangler internal log: no file found under \$HOME/.wrangler/logs or \$HOME/.config/.wrangler/logs" >&2
      fi
      echo "--- begin raw wrangler NDJSON ($wrangler_upload_ndjson) ---" >&2
      cat "$wrangler_upload_ndjson" >&2 || true
      echo "--- end raw wrangler NDJSON ---" >&2
      echo "--- begin raw wrangler stdout ($wrangler_upload_stdout) ---" >&2
      cat "$wrangler_upload_stdout" >&2 || true
      echo "--- end raw wrangler stdout ---" >&2
      echo "--- begin raw wrangler stderr ($wrangler_upload_stderr) ---" >&2
      cat "$wrangler_upload_stderr" >&2 || true
      echo "--- end raw wrangler stderr ---" >&2
      set -e
      exit 1
    fi

    # Post-condition (b): cross-check via versions list that the upload
    # landed server-side with the expected commit tag annotation. The `| cat`
    # pipe ensures wrangler's stdout is delivered through a pipe-shaped fd
    # before being redirected to disk (observed empirically: `wrangler ...
    # --json > file` intermittently produces zero bytes whereas `wrangler
    # ... --json | cat > file` reliably produces the full JSON output).
    wrangler_list_json="$tmpdir/wrangler-versions-list.json"

    node "$WRANGLER" --config "$WRANGLER_CONFIG" versions list --json \
      | cat > "$wrangler_list_json"

    matched_count=$(jq --arg tag "$commit_tag" \
      '[.[] | select(.annotations["workers/tag"] == $tag)] | length' \
      "$wrangler_list_json" 2>/dev/null || echo 0)
    if [[ "$matched_count" -lt 1 ]]; then
      echo "" >&2
      echo "error: uploaded version with tag ${commit_tag} not found in versions list" >&2
      echo "  post-condition (b) failed: wrangler versions list returned no entries" >&2
      echo "                              with annotations[\"workers/tag\"] == ${commit_tag}" >&2
      echo "  raw versions list output: $wrangler_list_json" >&2
      echo "  hint: wrangler reported a version_id locally but the Cloudflare API did" >&2
      echo "        not persist it; inspect the raw versions list for surrounding entries" >&2
      exit 1
    fi

    echo ""
    echo "Version uploaded successfully"
    echo "  Worker Version ID: ${version_id}"
    echo "  Tag: ${commit_tag}"
    echo "  Full SHA: ${commit_sha}"
    echo "  Message: ${version_message}"
    echo "  Preview URL: https://b-${safe_branch}-${preview_host_segment}.sciexp.workers.dev"
    ;;

  production)
    echo "Deploying $pkg_name to production from branch: ${current_branch}"
    echo "Current commit: ${commit_short}"
    echo "Full SHA: ${commit_sha}"
    echo "Looking for existing version with tag: ${commit_tag}"
    echo "Deployment message: ${deploy_msg}"
    echo ""

    export WRANGLER_CONFIG="$wrangler_config"

    # Query for an existing version uploaded from this commit (via preview).
    wrangler_list_json="$tmpdir/wrangler-versions-list.json"

    node "$WRANGLER" --config "$WRANGLER_CONFIG" versions list --json \
      | cat > "$wrangler_list_json"

    existing_version=$(jq -r --arg tag "$commit_tag" \
      '.[] | select(.annotations["workers/tag"] == $tag) | .id' \
      "$wrangler_list_json" 2>/dev/null | head -1 || true)

    if [[ -n "$existing_version" ]]; then
      echo "found existing version: ${existing_version}"
      echo "  this version was already built and tested in preview"
      echo "  promoting to 100% production traffic..."
      echo ""

      export DEPLOYMENT_MESSAGE="$deploy_msg"
      export EXISTING_VERSION="$existing_version"

      # Post-condition verification mirrors the preview path: capture
      # wrangler's NDJSON event log via WRANGLER_OUTPUT_FILE_PATH, assert a
      # non-empty deployment_id on the `version-deploy` event, then
      # cross-check via `wrangler deployments list --json` before declaring
      # success.
      deploy_ndjson="$tmpdir/wrangler-versions-deploy.ndjson"
      deploy_stdout="$tmpdir/wrangler-versions-deploy.stdout"
      : > "$deploy_ndjson"
      : > "$deploy_stdout"
      export WRANGLER_OUTPUT_FILE_PATH="$deploy_ndjson"

      node "$WRANGLER" --config "$WRANGLER_CONFIG" versions deploy \
          "${EXISTING_VERSION}@100%" \
          --yes \
          --message "$DEPLOYMENT_MESSAGE" \
        | tee "$deploy_stdout"

      unset WRANGLER_OUTPUT_FILE_PATH

      deployment_id=""
      if [[ -s "$deploy_ndjson" ]]; then
        deployment_id=$(
          jq -rs '
            map(select(type == "object" and (.type // "") == "version-deploy"))
            | .[0].deployment_id // empty
          ' "$deploy_ndjson" 2>/dev/null || true
        )
      fi
      if [[ -z "$deployment_id" ]]; then
        deployment_id=$(
          grep -oiE '(Deployment ID|deployment_id)[[:space:]]*:[[:space:]]*[a-f0-9-]+' \
            "$deploy_stdout" 2>/dev/null \
            | awk '{print $NF}' \
            | head -1 || true
        )
      fi
      if [[ -z "$deployment_id" ]]; then
        echo "" >&2
        echo "error: wrangler exited 0 but produced no Deployment ID" >&2
        echo "  post-condition (a) failed: neither WRANGLER_OUTPUT_FILE_PATH NDJSON" >&2
        echo "                              event log nor wrangler stdout contained" >&2
        echo "                              a recognizable Deployment ID" >&2
        echo "  raw wrangler event log: $deploy_ndjson" >&2
        echo "  raw wrangler stdout:   $deploy_stdout" >&2
        echo "  hints:" >&2
        echo "    - confirm CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID are exported by" >&2
        echo "      the caller" >&2
        echo "    - if linux-x64 regression, confirm wrangler invoked under real node and" >&2
        echo "      not bun-fake-node" >&2
        exit 1
      fi

      deployments_list_json="$tmpdir/wrangler-deployments-list.json"

      node "$WRANGLER" --config "$WRANGLER_CONFIG" deployments list --json \
        | cat > "$deployments_list_json"

      found_count=$(jq --arg did "$deployment_id" --arg vid "$existing_version" \
        '[.[] | select(
           .id == $did
           or .deployment_id == $did
           or ((.versions // []) | map(.version_id // .id // "") | index($vid) != null)
         )] | length' \
        "$deployments_list_json" 2>/dev/null || echo 0)
      if [[ "$found_count" -lt 1 ]]; then
        echo "" >&2
        echo "error: deployment ${deployment_id} (version ${existing_version}) not found in deployments list" >&2
        echo "  post-condition (b) failed: wrangler deployments list returned no" >&2
        echo "                              entries matching the just-deployed id/version" >&2
        echo "  raw deployments list output: $deployments_list_json" >&2
        echo "  hint: wrangler reported a deployment locally but the Cloudflare API did" >&2
        echo "        not persist it; inspect the raw deployments list for surrounding entries" >&2
        exit 1
      fi

      echo ""
      echo "successfully promoted version ${existing_version} to production"
      echo "  Deployment ID: ${deployment_id}"
      echo "  tag: ${commit_tag}"
      echo "  full SHA: ${commit_sha}"
      echo "  deployed by: ${deploy_msg}"
      echo "  production URL: ${production_url}"
    else
      echo "warning: no existing version found with tag: ${commit_tag}"
      echo "  this should only happen if:"
      echo "    - this is the first deployment"
      echo "    - commit was made directly on main (not recommended)"
      echo "    - version was cleaned up (retention policy)"
      echo ""
      echo "  falling back to direct deploy of the nix-built payload..."
      echo ""

      export DEPLOYMENT_MESSAGE="$deploy_msg"

      # Fallback direct-deploy: same post-condition pattern, but the NDJSON
      # event is `type == "deploy"` carrying `version_id` (no deployment_id
      # field on this event type).
      deploy_ndjson="$tmpdir/wrangler-deploy.ndjson"
      deploy_stdout="$tmpdir/wrangler-deploy.stdout"
      : > "$deploy_ndjson"
      : > "$deploy_stdout"
      export WRANGLER_OUTPUT_FILE_PATH="$deploy_ndjson"

      node "$WRANGLER" --config "$WRANGLER_CONFIG" deploy \
          --message "$DEPLOYMENT_MESSAGE" \
        | tee "$deploy_stdout"

      unset WRANGLER_OUTPUT_FILE_PATH

      deploy_version_id=""
      if [[ -s "$deploy_ndjson" ]]; then
        deploy_version_id=$(
          jq -rs '
            map(select(type == "object" and (.type // "") == "deploy"))
            | .[0].version_id // empty
          ' "$deploy_ndjson" 2>/dev/null || true
        )
      fi
      if [[ -z "$deploy_version_id" ]]; then
        deploy_version_id=$(
          grep -oiE '(Current Version ID|Worker Version ID|version_id)[[:space:]]*:[[:space:]]*[a-f0-9-]+' \
            "$deploy_stdout" 2>/dev/null \
            | awk '{print $NF}' \
            | head -1 || true
        )
      fi
      if [[ -z "$deploy_version_id" ]]; then
        echo "" >&2
        echo "error: wrangler exited 0 but produced no Deployment Version ID (fallback direct deploy)" >&2
        echo "  post-condition (a) failed: neither WRANGLER_OUTPUT_FILE_PATH NDJSON" >&2
        echo "                              event log nor wrangler stdout contained" >&2
        echo "                              a recognizable version_id" >&2
        echo "  raw wrangler event log: $deploy_ndjson" >&2
        echo "  raw wrangler stdout:   $deploy_stdout" >&2
        echo "  hints:" >&2
        echo "    - confirm CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID are exported by" >&2
        echo "      the caller" >&2
        echo "    - if linux-x64 regression, confirm wrangler invoked under real node and" >&2
        echo "      not bun-fake-node" >&2
        exit 1
      fi
      # Reuse deployment_id slot below (it now holds the just-deployed
      # version_id since `wrangler deploy` emits no server-assigned
      # deployment id directly).
      deployment_id="$deploy_version_id"

      deployments_list_json="$tmpdir/wrangler-deployments-list.json"

      node "$WRANGLER" --config "$WRANGLER_CONFIG" deployments list --json \
        | cat > "$deployments_list_json"

      found_count=$(jq --arg vid "$deploy_version_id" \
        '[.[] | select(
           .id == $vid
           or .deployment_id == $vid
           or ((.versions // []) | map(.version_id // .id // "") | index($vid) != null)
         )] | length' \
        "$deployments_list_json" 2>/dev/null || echo 0)
      if [[ "$found_count" -lt 1 ]]; then
        echo "" >&2
        echo "error: deployment for version ${deploy_version_id} not found in deployments list (fallback direct deploy)" >&2
        echo "  post-condition (b) failed: wrangler deployments list returned no" >&2
        echo "                              entries matching the just-deployed version_id" >&2
        echo "  raw deployments list output: $deployments_list_json" >&2
        echo "  hint: wrangler reported a deployment locally but the Cloudflare API did" >&2
        echo "        not persist it; inspect the raw deployments list for surrounding entries" >&2
        exit 1
      fi

      echo ""
      echo "deployed nix-built payload directly to production"
      echo "  Deployment Version ID: ${deployment_id}"
      echo "  tag: ${commit_tag}"
      echo "  full SHA: ${commit_sha}"
      echo "  deployed by: ${deploy_msg}"
      echo "  production URL: ${production_url}"
      echo "  warning: this version was not tested in preview first"
    fi
    ;;

  *)
    echo "error: unknown subcommand '$mode'" >&2
    echo "usage: deploy-sites $pkg_name preview <branch> | deploy-sites $pkg_name production" >&2
    echo "(run with --help for full usage and env-var contract)" >&2
    exit 2
    ;;
esac
