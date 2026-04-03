#!/usr/bin/env bash
# Check if playwright package versions match playwright-web-flake pin
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get playwright-web-flake version from flake.nix
FLAKE_VERSION=$(grep "playwright-web-flake.url" flake.nix | sed 's/.*\/\([0-9.]*\)".*/\1/')

MISMATCH=0

check_version() {
  local file="$1"
  local pkg="$2"
  local version
  version=$(jq -r ".devDependencies.\"$pkg\" // empty" "$file" | sed 's/[^0-9.]//g')
  if [ -z "$version" ]; then
    return
  fi
  local flake_maj_min npm_maj_min
  flake_maj_min=$(echo "$FLAKE_VERSION" | cut -d. -f1-2)
  npm_maj_min=$(echo "$version" | cut -d. -f1-2)
  if [ "$flake_maj_min" != "$npm_maj_min" ]; then
    echo -e "  ${RED}x${NC} $file $pkg: $version (expected $FLAKE_VERSION)"
    MISMATCH=1
  else
    echo -e "  ${GREEN}ok${NC} $file $pkg: $version"
  fi
}

echo "playwright-web-flake: $FLAKE_VERSION"
echo ""

check_version package.json "@playwright/test"
check_version packages/docs/package.json "@playwright/test"
check_version packages/docs/package.json "playwright"
check_version packages/eventcatalog/package.json "@playwright/test"
check_version packages/eventcatalog/package.json "playwright"

echo ""
if [ "$MISMATCH" -eq 0 ]; then
  echo -e "${GREEN}Versions synchronized${NC}"
  exit 0
else
  echo -e "${RED}Version mismatch detected${NC}"
  echo -e "${YELLOW}Run: just update-playwright${NC}"
  exit 1
fi
