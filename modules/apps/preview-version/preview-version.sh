#!/usr/bin/env bash
# shellcheck shell=bash
# preview-version.sh — Parametric semantic-release version preview.
#
# Argv grammar:
#   preview-version [target-branch] [pkgName]
#     target-branch  release branch to simulate merging into (default: main)
#     pkgName        ironstar-docs | ironstar-eventcatalog (omit for root)
#
# Required (config, injected by preview-version.nix runtimeEnv):
#   IRONSTAR_DOCS_NODE_MODULES         ironstar-docs-deps node_modules tree
#   IRONSTAR_EVENTCATALOG_NODE_MODULES ironstar-eventcatalog-deps node_modules tree
#
# Optional (caller-provided):
#   CURRENT_BRANCH  bookmark/branch name to attach HEAD to when invoked from
#                   a jj-colocated detached-HEAD setup.
#
# No secret env vars required: semantic-release runs --dry-run with
# @semantic-release/github filtered out of the plugin list.

set -euo pipefail

usage() {
  cat <<'EOF'
usage: preview-version [target-branch] [pkgName]
       preview-version --help

Preview the semantic-release version that would be published after merging
the current branch into <target-branch>. Simulates the merge via
`git merge-tree --write-tree`, runs semantic-release in --dry-run / --no-ci
mode against a temporary worktree, and prints the next version (or a
no-bump / unsupported-branch notice).

Positional arguments:
  target-branch   Release branch to simulate merging into (default: main).
  pkgName         ironstar-docs | ironstar-eventcatalog. Omit for root package.

Flags:
  --help, -h      Print this usage and exit 0.

Environment:
  IRONSTAR_DOCS_NODE_MODULES         (required for ironstar-docs)
  IRONSTAR_EVENTCATALOG_NODE_MODULES (required for ironstar-eventcatalog)
  CURRENT_BRANCH  (optional) Bookmark/branch name to attach HEAD to when
                  invoked from a jj-colocated detached-HEAD setup.

Examples:
  nix run .#preview-version
  nix run .#preview-version -- main ironstar-docs
  nix run .#preview-version -- beta ironstar-eventcatalog
EOF
}

case "${1:-}" in
  -h | --help)
    usage
    exit 0
    ;;
esac

TARGET_BRANCH="${1:-main}"
PKG_NAME="${2:-}"

# --- per-package dispatch -------------------------------------------------

PACKAGE_PATH=""
NODE_MODULES_PATH=""
case "$PKG_NAME" in
  "")
    : # root package; no node_modules link
    ;;
  ironstar-docs)
    : "${IRONSTAR_DOCS_NODE_MODULES:?IRONSTAR_DOCS_NODE_MODULES not set; preview-version.nix must expose ironstar-docs-deps via runtimeEnv}"
    PACKAGE_PATH="packages/docs"
    NODE_MODULES_PATH="$IRONSTAR_DOCS_NODE_MODULES"
    ;;
  ironstar-eventcatalog)
    : "${IRONSTAR_EVENTCATALOG_NODE_MODULES:?IRONSTAR_EVENTCATALOG_NODE_MODULES not set; preview-version.nix must expose ironstar-eventcatalog-deps via runtimeEnv}"
    PACKAGE_PATH="packages/eventcatalog"
    NODE_MODULES_PATH="$IRONSTAR_EVENTCATALOG_NODE_MODULES"
    ;;
  *)
    echo "error: unknown pkgName '$PKG_NAME'; expected ironstar-docs or ironstar-eventcatalog (or empty for root)" >&2
    usage >&2
    exit 2
    ;;
esac

REPO_ROOT=$(git rev-parse --show-toplevel)
WORKTREE_DIR=$(mktemp -d "${TMPDIR:-/tmp}/semantic-release-preview.XXXXXX")

ORIGINAL_TARGET_HEAD=""
ORIGINAL_REMOTE_HEAD=""

# Track node_modules symlink(s) we created for cleanup.
WORKTREE_NODE_MODULES_LINK=""
LOCAL_NODE_MODULES_LINK=""

# Local bare clone used to redirect semantic-release verifyAuth's
# `git push --dry-run HEAD:<target-branch>` away from the GitHub remote.
PREVIEW_BARE_DIR=""
PREVIEW_BARE=""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Resolve CURRENT_BRANCH with jj-colocated detached-HEAD awareness.
ORIGINAL_HEAD_SHA=""
WE_ATTACHED_HEAD=0
if [ -n "${CURRENT_BRANCH:-}" ]; then
  DETECTED_BRANCH=$(git branch --show-current)
  if [ -z "$DETECTED_BRANCH" ]; then
    ORIGINAL_HEAD_SHA=$(git rev-parse --verify HEAD)
    echo -e "${BLUE}CURRENT_BRANCH=${CURRENT_BRANCH} override; attaching HEAD for duration of preview${NC}" >&2
    if ! git checkout --quiet "$CURRENT_BRANCH"; then
      echo -e "${RED}error: failed to checkout branch '${CURRENT_BRANCH}' (set via CURRENT_BRANCH env var)${NC}" >&2
      exit 1
    fi
    WE_ATTACHED_HEAD=1
  fi
else
  CURRENT_BRANCH=$(git branch --show-current)
  if [ -z "$CURRENT_BRANCH" ]; then
    echo -e "${RED}error: HEAD is detached and CURRENT_BRANCH env var is not set${NC}" >&2
    echo -e "${YELLOW}this is common in jj-colocated repositories where git HEAD is detached by default.${NC}" >&2
    echo -e "${YELLOW}to proceed, either:${NC}" >&2
    echo -e "${YELLOW}  - export CURRENT_BRANCH=<bookmark-name>  (recommended for jj callers)${NC}" >&2
    echo -e "${YELLOW}  - git checkout <bookmark-name>           (attach HEAD then re-run)${NC}" >&2
    exit 1
  fi
fi

# shellcheck disable=SC2329
cleanup() {
  local exit_code=$?

  if [ -n "$WORKTREE_NODE_MODULES_LINK" ] && [ -L "$WORKTREE_NODE_MODULES_LINK" ]; then
    rm -f "$WORKTREE_NODE_MODULES_LINK"
  fi
  if [ -n "$LOCAL_NODE_MODULES_LINK" ] && [ -L "$LOCAL_NODE_MODULES_LINK" ]; then
    rm -f "$LOCAL_NODE_MODULES_LINK"
  fi

  if [ -n "$ORIGINAL_TARGET_HEAD" ]; then
    echo -e "\n${BLUE}restoring ${TARGET_BRANCH} to original state...${NC}"
    git update-ref "refs/heads/$TARGET_BRANCH" "$ORIGINAL_TARGET_HEAD" 2>/dev/null || true
  fi

  if [ -n "$ORIGINAL_REMOTE_HEAD" ]; then
    git update-ref "refs/remotes/origin/$TARGET_BRANCH" "$ORIGINAL_REMOTE_HEAD" 2>/dev/null || true
  fi

  if [ -d "$WORKTREE_DIR" ]; then
    echo -e "${BLUE}cleaning up worktree...${NC}"
    git worktree remove --force "$WORKTREE_DIR" 2>/dev/null || true
    git worktree prune 2>/dev/null || true
  fi

  if [ -n "$PREVIEW_BARE_DIR" ] && [ -d "$PREVIEW_BARE_DIR" ]; then
    rm -rf "$PREVIEW_BARE_DIR"
  fi

  if [ "${WE_ATTACHED_HEAD:-0}" -eq 1 ] && [ -n "$ORIGINAL_HEAD_SHA" ] && [ -n "$REPO_ROOT" ]; then
    echo -e "${BLUE}restoring detached HEAD at ${ORIGINAL_HEAD_SHA}...${NC}" >&2
    ( cd "$REPO_ROOT" && git checkout --quiet --detach "$ORIGINAL_HEAD_SHA" ) || true
  fi

  exit "$exit_code"
}

trap cleanup EXIT INT TERM

# link_node_modules <target-dir>: symlink NODE_MODULES_PATH into the
# directory's node_modules slot, refusing to overwrite a real install.
link_node_modules() {
  local target_dir="$1"
  local slot="$target_dir/node_modules"
  if [[ -e "$slot" && ! -L "$slot" ]]; then
    echo -e "${RED}error: ${slot} exists and is not a symlink; refusing to overwrite a local bun install${NC}" >&2
    exit 1
  fi
  ln -snf "$NODE_MODULES_PATH" "$slot"
  echo "$slot"
}

if [ "$CURRENT_BRANCH" == "$TARGET_BRANCH" ]; then
  echo -e "${YELLOW}already on target branch ${TARGET_BRANCH}${NC}"
  echo -e "${YELLOW}running test-release instead of preview${NC}\n"
  if [ -n "$PACKAGE_PATH" ]; then
    cd "$REPO_ROOT/$PACKAGE_PATH"
    LOCAL_NODE_MODULES_LINK=$(link_node_modules "$PWD")
    exec node ./node_modules/.bin/semantic-release --dry-run --no-ci
  else
    cd "$REPO_ROOT"
    echo -e "${YELLOW}root package preview on target branch is not supported (no node_modules wired)${NC}" >&2
    exit 1
  fi
fi

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}semantic-release version preview${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "current branch:  ${GREEN}${CURRENT_BRANCH}${NC}"
echo -e "target branch:   ${GREEN}${TARGET_BRANCH}${NC}"
if [ -n "$PKG_NAME" ]; then
  echo -e "package:         ${GREEN}${PKG_NAME} (${PACKAGE_PATH})${NC}"
else
  echo -e "package:         ${GREEN}(root)${NC}"
fi
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}\n"

if ! git show-ref --verify --quiet "refs/heads/$TARGET_BRANCH"; then
  echo -e "${RED}error: target branch '${TARGET_BRANCH}' does not exist${NC}" >&2
  exit 1
fi

ORIGINAL_TARGET_HEAD=$(git rev-parse "$TARGET_BRANCH")
ORIGINAL_REMOTE_HEAD=$(git rev-parse "origin/$TARGET_BRANCH" 2>/dev/null || echo "")

echo -e "${BLUE}simulating merge of ${CURRENT_BRANCH} → ${TARGET_BRANCH}...${NC}"

# `if ! VAR=$(...)` form deliberate: errexit + command-substitution
# combinations are fragile; explicit if-guard makes failure handling
# independent of errexit semantics.
if ! MERGE_OUTPUT=$(git merge-tree --write-tree "$TARGET_BRANCH" "$CURRENT_BRANCH" 2>&1); then
  echo -e "${RED}error: merge conflicts detected${NC}" >&2
  echo -e "${YELLOW}please resolve conflicts in your branch before previewing${NC}" >&2
  echo -e "\n${YELLOW}conflict details:${NC}" >&2
  echo "$MERGE_OUTPUT" >&2
  exit 1
fi

MERGE_TREE=$(echo "$MERGE_OUTPUT" | head -1)

if [ -z "$MERGE_TREE" ]; then
  echo -e "${RED}error: failed to create merge tree${NC}" >&2
  exit 1
fi

echo -e "${BLUE}creating temporary merge commit...${NC}"
TEMP_COMMIT=$(git commit-tree -p "$TARGET_BRANCH" -p "$CURRENT_BRANCH" \
  -m "Temporary merge for semantic-release preview" "$MERGE_TREE")

if [ -z "$TEMP_COMMIT" ]; then
  echo -e "${RED}error: failed to create temporary merge commit${NC}" >&2
  exit 1
fi

echo -e "${BLUE}temporarily updating ${TARGET_BRANCH} ref for analysis...${NC}"
git update-ref "refs/heads/$TARGET_BRANCH" "$TEMP_COMMIT"
git update-ref "refs/remotes/origin/$TARGET_BRANCH" "$TEMP_COMMIT"

# Local bare clone redirects semantic-release verifyAuth's dry-run push
# away from GitHub origin to a quiescent file:// remote.
echo -e "${BLUE}creating local bare clone for semantic-release repository-url override...${NC}"
PREVIEW_BARE_DIR=$(mktemp -d "${TMPDIR:-/tmp}/preview-bare.XXXXXX")
PREVIEW_BARE="$PREVIEW_BARE_DIR/preview.git"
git clone --quiet --bare "$REPO_ROOT" "$PREVIEW_BARE"

echo -e "${BLUE}creating temporary worktree at ${TARGET_BRANCH}...${NC}"
git worktree add --quiet "$WORKTREE_DIR" "$TARGET_BRANCH"

cd "$WORKTREE_DIR"

if [ -n "$PACKAGE_PATH" ]; then
  if [ ! -d "$PACKAGE_PATH" ]; then
    echo -e "${RED}error: package path '${PACKAGE_PATH}' does not exist${NC}" >&2
    exit 1
  fi
  WORKTREE_NODE_MODULES_LINK=$(link_node_modules "$WORKTREE_DIR/$PACKAGE_PATH")
  cd "$PACKAGE_PATH"
fi

echo -e "\n${BLUE}running semantic-release analysis...${NC}\n"

# Filter @semantic-release/github so GITHUB_TOKEN is not required for preview.
PLUGINS="@semantic-release/commit-analyzer,@semantic-release/release-notes-generator"

echo "RELEASE-PREVIEW-BARE: $PREVIEW_BARE"

OUTPUT=$(GITHUB_REF="refs/heads/$TARGET_BRANCH" node ./node_modules/.bin/semantic-release --dry-run --no-ci --repository-url "file://$PREVIEW_BARE" --branches "$TARGET_BRANCH" --plugins "$PLUGINS" 2>&1 || true)

echo "$OUTPUT" | grep -v "^$" | grep -vE "(No more plugins|does not provide step)" | \
  grep -E "(semantic-release|Running|analyzing|Found.*commits|release version|Release note|Features|Bug Fixes|Breaking Changes|Published|\*\s)" || true

echo -e "\n${BLUE}═══════════════════════════════════════════════════════════════${NC}"

if echo "$OUTPUT" | grep -q "There are no relevant changes"; then
  echo -e "${YELLOW}no version bump required${NC}"
  echo -e "no semantic commits found since last release"
elif echo "$OUTPUT" | grep -q "is not configured to publish from"; then
  echo -e "${YELLOW}cannot determine version${NC}"
  echo -e "branch ${TARGET_BRANCH} is not in release configuration"
elif VERSION=$(echo "$OUTPUT" | grep -oP 'next release version is \K[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?' | head -1); then
  echo -e "${GREEN}next version: ${VERSION}${NC}"
  if TYPE=$(echo "$OUTPUT" | grep -oP 'Release type: \K[a-z]+' | head -1); then
    echo -e "release type: ${TYPE}"
  fi
else
  echo -e "${YELLOW}could not parse version from output${NC}"
  echo -e "check the semantic-release output above for details"
fi

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}\n"

# "No version bump required" is a valid outcome, not an error: exit 0.
exit 0
