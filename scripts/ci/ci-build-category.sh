#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# CI Category Builder
# ============================================================================
# Build specific categories of flake outputs for GitHub Actions matrix jobs.
# Designed to minimize disk space usage per job by building subsets of outputs.
#
# Usage:
#   ci-build-category.sh <system> <category>
#
# Arguments:
#   system    - Target system (x86_64-linux, aarch64-linux, aarch64-darwin)
#   category  - Output category to build:
#               - packages: all packages for system
#               - checks: checks only
#               - flake-check: all checks via nix flake check (parallel)
#               - devshells: devshells only
#
# Examples:
#   ci-build-category.sh x86_64-linux packages
#   ci-build-category.sh x86_64-linux checks
#   ci-build-category.sh x86_64-linux flake-check
#   ci-build-category.sh x86_64-linux devshells
# ============================================================================

# ============================================================================
# Argument Parsing
# ============================================================================

if [ $# -lt 2 ]; then
    echo "usage: $0 <system> <category>"
    echo ""
    echo "system: x86_64-linux, aarch64-linux, aarch64-darwin"
    echo "category: packages, checks, flake-check, devshells"
    exit 1
fi

SYSTEM="$1"
CATEGORY="$2"

# ============================================================================
# Validation
# ============================================================================

# Validate system
case "$SYSTEM" in
    x86_64-linux|aarch64-linux|aarch64-darwin)
        ;;
    *)
        echo "error: unsupported system '$SYSTEM'"
        echo "supported: x86_64-linux, aarch64-linux, aarch64-darwin"
        exit 1
        ;;
esac

# Validate category
case "$CATEGORY" in
    packages|checks|flake-check|devshells)
        ;;
    *)
        echo "error: unknown category '$CATEGORY'"
        echo "valid: packages, checks, flake-check, devshells"
        exit 1
        ;;
esac

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
    local title="$1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$title"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

print_step() {
    local step="$1"
    echo ""
    echo "step: $step"
}

report_disk_usage() {
    echo ""
    echo "disk usage:"
    df -h / | tail -1
}

# ============================================================================
# Category Build Functions
# ============================================================================

build_packages() {
    local system="$1"

    print_header "building packages for $system"

    print_step "discovering packages"
    local packages
    packages=$(nix eval ".#packages.$system" --apply 'builtins.attrNames' --json 2>/dev/null | jq -r '.[]' || echo "")

    if [ -z "$packages" ]; then
        echo "no packages found for $system"
        return 0
    fi

    local count
    count=$(echo "$packages" | wc -l | tr -d ' ')
    echo "found $count packages"

    print_step "building packages"
    local failed=0
    while read -r pkg; do
        if [ -n "$pkg" ]; then
            echo ""
            echo "building packages.$system.$pkg"
            if ! nix build ".#packages.$system.$pkg" -L --no-link; then
                echo "failed to build packages.$system.$pkg"
                failed=$((failed + 1))
            fi
        fi
    done <<< "$packages"

    if [ $failed -gt 0 ]; then
        echo ""
        echo "failed to build $failed packages"
        return 1
    fi

    echo ""
    echo "successfully built $count packages"
}

build_checks() {
    local system="$1"

    print_header "building checks for $system"

    print_step "discovering checks"
    local checks
    checks=$(nix eval ".#checks.$system" --apply 'builtins.attrNames' --json 2>/dev/null | jq -r '.[]' || echo "")

    if [ -z "$checks" ]; then
        echo "no checks found for $system"
        return 0
    fi

    local count
    count=$(echo "$checks" | wc -l | tr -d ' ')
    echo "found $count checks"

    print_step "building checks"
    local failed=0
    while read -r check; do
        if [ -n "$check" ]; then
            echo ""
            echo "building checks.$system.$check"
            if ! nix build ".#checks.$system.$check" -L --no-link; then
                echo "failed to build checks.$system.$check"
                failed=$((failed + 1))
            fi
        fi
    done <<< "$checks"

    if [ $failed -gt 0 ]; then
        echo ""
        echo "failed to build $failed checks"
        return 1
    fi

    echo ""
    echo "successfully built $count checks"
}

build_flake_check() {
    local system="$1"

    print_header "running nix flake check for $system"

    # nix flake check builds all checks in parallel with shared cargoArtifacts
    # --system ensures we check the correct platform
    nix flake check --system "$system" -L

    echo ""
    echo "flake check completed successfully"
}

build_devshells() {
    local system="$1"

    print_header "building devshells for $system"

    print_step "discovering devshells"
    local devshells
    devshells=$(nix eval ".#devShells.$system" --apply 'builtins.attrNames' --json 2>/dev/null | jq -r '.[]' || echo "")

    if [ -z "$devshells" ]; then
        echo "no devshells found for $system"
        return 0
    fi

    local count
    count=$(echo "$devshells" | wc -l | tr -d ' ')
    echo "found $count devshells"

    print_step "building devshells"
    local failed=0
    while read -r shell; do
        if [ -n "$shell" ]; then
            echo ""
            echo "building devShells.$system.$shell"
            if ! nix build ".#devShells.$system.$shell" -L --no-link; then
                echo "failed to build devShells.$system.$shell"
                failed=$((failed + 1))
            fi
        fi
    done <<< "$devshells"

    if [ $failed -gt 0 ]; then
        echo ""
        echo "failed to build $failed devshells"
        return 1
    fi

    echo ""
    echo "successfully built $count devshells"
}

# ============================================================================
# Main Execution
# ============================================================================

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              CI Category Builder                              ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "system: $SYSTEM"
echo "category: $CATEGORY"
echo ""

# Record start time and disk usage
START_TIME=$(date +%s)
echo "start time: $(date)"
report_disk_usage

# Execute appropriate build function
case "$CATEGORY" in
    packages)
        build_packages "$SYSTEM"
        ;;
    checks)
        build_checks "$SYSTEM"
        ;;
    flake-check)
        build_flake_check "$SYSTEM"
        ;;
    devshells)
        build_devshells "$SYSTEM"
        ;;
esac

BUILD_STATUS=$?

# Report completion
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
print_header "build summary"
echo ""
echo "category: $CATEGORY"
echo "duration: ${DURATION}s"
report_disk_usage
echo ""

if [ $BUILD_STATUS -eq 0 ]; then
    echo "status: success"
    exit 0
else
    echo "status: failed"
    exit 1
fi
