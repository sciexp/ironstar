#!/usr/bin/env bash
# Check if @playwright/test version matches playwright-web-flake pin
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get playwright-web-flake version from flake.nix
FLAKE_VERSION=$(grep "playwright-web-flake.url" flake.nix | sed 's/.*\/\([0-9.]*\)".*/\1/')

# Get @playwright/test version from package.json (strip ^, ~, etc.)
NPM_VERSION=$(jq -r '.devDependencies."@playwright/test"' packages/docs/package.json | sed 's/[^0-9.]//g')

# Extract major.minor for comparison (ignore patch)
FLAKE_MAJ_MIN=$(echo "$FLAKE_VERSION" | cut -d. -f1-2)
NPM_MAJ_MIN=$(echo "$NPM_VERSION" | cut -d. -f1-2)

echo "Version Check:"
echo "  playwright-web-flake: $FLAKE_VERSION"
echo "  @playwright/test:     $NPM_VERSION"
echo ""

if [ "$FLAKE_MAJ_MIN" = "$NPM_MAJ_MIN" ]; then
    echo -e "${GREEN}✓ Versions synchronized${NC}"
    exit 0
else
    echo -e "${RED}✗ Version mismatch detected!${NC}"
    echo ""
    echo -e "${YELLOW}Action required:${NC}"
    if [[ "$FLAKE_VERSION" > "$NPM_VERSION" ]]; then
        echo "  1. Update @playwright/test to match flake:"
        echo "     just deps-update-playwright"
    else
        echo "  1. Update playwright-web-flake in flake.nix to $NPM_MAJ_MIN.x"
        echo "  2. Run: nix flake update playwright-web-flake"
        echo "  3. Test: just docs-test"
    fi
    exit 1
fi
