# List all recipes
default:
    @just --list

# Contents
## D2 Diagrams
## Workspace
## CI/CD
## Cloudflare
## Docs
## Nix
## Rust
## Spec (Idris2)
## Release
## Secrets
## Slides
## Testing
## Template

## D2 Diagrams

# Render all D2 diagrams to SVG
[group('d2')]
d2-render:
    @mkdir -p docs/notes/event-modeling/rendered
    fd . docs/notes/event-modeling/d2 -e d2 -x d2 {} docs/notes/event-modeling/rendered/{/.}.svg

# Watch D2 files and re-render on changes
[group('d2')]
d2-watch:
    fd . docs/notes/event-modeling/d2 -e d2 | entr -c just d2-render

## Workspace

# Install all workspace dependencies
[group('workspace')]
install:
  bun install

# Update dependencies to latest compatible versions (respects ^, ~)
[group('workspace')]
update:
  bun update

# Upgrade dependencies to latest versions (ignoring semver constraints)
[group('workspace')]
upgrade:
  bun upgrade

# Show outdated dependencies
[group('workspace')]
outdated:
  bun outdated

# Check if @playwright/test matches playwright-web-flake version
[group('workspace')]
check-playwright:
  @./scripts/check-playwright-sync.sh

# Update @playwright/test to match playwright-web-flake, update lockfile, and test
[group('workspace')]
update-playwright: check-playwright
  #!/usr/bin/env bash
  set -euo pipefail
  FLAKE_VERSION=$(grep "playwright-web-flake.url" flake.nix | sed 's/.*\/\([0-9.]*\)".*/\1/')
  echo "Updating @playwright/test to ^$FLAKE_VERSION..."
  cd packages/docs
  # Use jq to update package.json
  jq ".devDependencies.\"@playwright/test\" = \"^$FLAKE_VERSION\"" package.json > package.json.tmp
  mv package.json.tmp package.json
  cd ../..
  echo "Running bun install to update lockfile..."
  bun install
  echo "Running tests to verify..."
  just docs-test

# Clean all workspace build artifacts
[group('workspace')]
clean:
  rm -rf packages/*/dist packages/*/node_modules

# Run command in specific package
[group('workspace')]
pkg package +command:
  cd packages/{{ package }} && {{ command }}

# Run command in docs package
[group('workspace')]
docs +command:
  cd packages/docs && {{ command }}

## CI/CD

# Format code with Biome
[group('CI/CD')]
format:
  cd packages/docs && bun run format

# Lint code with Biome
[group('CI/CD')]
lint:
  cd packages/docs && bun run lint

# Check and fix code with Biome
[group('CI/CD')]
biome-check:
  cd packages/docs && bun run check:fix

# Build specific nix category (for CI matrix distribution)
[group('CI/CD')]
ci-build-category system category:
  @./scripts/ci/ci-build-category.sh "{{system}}" "{{category}}"

# Validate flake structure and required recipes
[group('CI/CD')]
validate-flake:
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Validating flake structure..."
  REQUIRED_RECIPES="check lint fmt test"
  JUST_RECIPES=$(just --summary)
  for recipe in $REQUIRED_RECIPES; do
    if echo "$JUST_RECIPES" | grep -qw "$recipe"; then
      echo "OK: '$recipe' recipe found"
    else
      echo "ERROR: '$recipe' recipe not found"
      exit 1
    fi
  done
  echo "Running nix flake check..."
  if command -v nom &> /dev/null; then
    nix flake check --impure --log-format internal-json -v |& nom --json
  else
    nix flake check --impure
  fi

# Run pre-commit hooks
[group('CI/CD')]
pre-commit:
  pre-commit run --all-files

# Update github vars for repo from environment variables
[group('CI/CD')]
ghvars repo="":
  #!/usr/bin/env bash
  REPO="${repo:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
  echo "vars before updates:"
  echo
  PAGER=cat gh variable list --repo="$REPO"
  echo
  sops exec-env vars/shared.yaml '\
  gh variable set CACHIX_CACHE_NAME --repo='"$REPO"' --body="$CACHIX_CACHE_NAME"'
  echo
  echo "vars after updates (wait 3 seconds for github to update):"
  sleep 3
  echo
  PAGER=cat gh variable list --repo="$REPO"

# Update github secrets for repo from sops-encrypted secrets
# Note: SOPS_AGE_KEY is uploaded separately via sops-upload-github-key
[group('CI/CD')]
ghsecrets repo="":
  #!/usr/bin/env bash
  REPO="${repo:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
  echo "secrets before updates:"
  echo
  PAGER=cat gh secret list --repo="$REPO"
  echo
  sops exec-env vars/shared.yaml '\
  gh secret set CACHIX_AUTH_TOKEN --repo='"$REPO"' --body="$CACHIX_AUTH_TOKEN" && \
  gh secret set CLOUDFLARE_ACCOUNT_ID --repo='"$REPO"' --body="$CLOUDFLARE_ACCOUNT_ID" && \
  gh secret set CLOUDFLARE_API_TOKEN --repo='"$REPO"' --body="$CLOUDFLARE_API_TOKEN" && \
  gh secret set FAST_FORWARD_PAT --repo='"$REPO"' --body="$FAST_FORWARD_PAT" && \
  gh secret set FLAKE_UPDATER_APP_ID --repo='"$REPO"' --body="$FLAKE_UPDATER_APP_ID" && \
  gh secret set FLAKE_UPDATER_PRIVATE_KEY --repo='"$REPO"' --body="$FLAKE_UPDATER_PRIVATE_KEY"'
  echo
  echo "secrets after updates (wait 3 seconds for github to update):"
  sleep 3
  echo
  PAGER=cat gh secret list --repo="$REPO"
  echo
  echo "ℹ️  SOPS_AGE_KEY not uploaded (requires special handling)"
  echo "   Upload with: just sops-upload-github-key"

# List available workflows and associated jobs using act
[group('CI/CD')]
list-workflows:
  @act -l

# Test build job locally with act
[group('CI/CD')]
test-build branch=`git branch --show-current`:
  @echo "Testing build job locally (branch: {{branch}})..."
  @sops exec-env vars/shared.yaml 'act pull_request \
    -W .github/workflows/ci.yaml \
    -j build \
    -s CI_AGE_KEY -s CACHIX_AUTH_TOKEN \
    -s GITHUB_TOKEN="$(gh auth token)" \
    --var CACHIX_CACHE_NAME \
    --input debug_enabled=false'

# Test deploy job locally with act
[group('CI/CD')]
test-deploy branch=`git branch --show-current`:
  @echo "Testing deploy job locally (branch: {{branch}})..."
  @echo "Note: Cloudflare deployment may not work in local environment"
  @sops exec-env vars/shared.yaml 'act push \
    -W .github/workflows/ci.yaml \
    -j deploy \
    -s CI_AGE_KEY -s CACHIX_AUTH_TOKEN \
    -s SOPS_AGE_KEY="$CI_AGE_KEY" \
    -s CLOUDFLARE_API_TOKEN -s CLOUDFLARE_ACCOUNT_ID \
    -s GITHUB_TOKEN="$(gh auth token)" \
    --var CACHIX_CACHE_NAME \
    --input debug_enabled=false'

# Trigger CI workflow remotely on GitHub
[group('CI/CD')]
gh-ci-run branch=`git branch --show-current` debug="false" deploy="false":
  #!/usr/bin/env bash
  echo "Triggering CI workflow on GitHub (branch: {{branch}}, debug: {{debug}}, deploy: {{deploy}})..."
  gh workflow run ci.yaml \
    --repo ${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)} \
    --ref "{{branch}}" \
    --field debug_enabled="{{debug}}" \
    --field deploy_enabled="{{deploy}}"
  echo "Check workflow status with: just gh-workflow-status"

# Test specific CI job remotely on GitHub
[group('CI/CD')]
gh-ci-job job branch=`git branch --show-current` debug="false":
  #!/usr/bin/env bash
  echo "Triggering CI job '{{job}}' on GitHub (branch: {{branch}}, debug: {{debug}})..."
  gh workflow run ci.yaml \
    --repo ${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)} \
    --ref "{{branch}}" \
    --field job="{{job}}" \
    --field debug_enabled="{{debug}}"
  echo "Check workflow status with: just gh-workflow-status"

# Test deploy workflow remotely on GitHub
[group('CI/CD')]
gh-deploy branch=`git branch --show-current` env="preview" debug="false":
  #!/usr/bin/env bash
  echo "Triggering deploy-docs workflow on GitHub (branch: {{branch}}, env: {{env}}, debug: {{debug}})..."
  gh workflow run deploy-docs.yaml \
    --repo ${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)} \
    --ref "{{branch}}" \
    --field branch="{{branch}}" \
    --field environment="{{env}}" \
    --field debug_enabled="{{debug}}"
  echo "Check workflow status with: just gh-workflow-status deploy-docs.yaml"

# View recent workflow runs status
[group('CI/CD')]
gh-workflow-status workflow="ci.yaml" branch=`git branch --show-current` limit="5":
  #!/usr/bin/env bash
  echo "Recent CI workflow runs:"
  gh run list \
    --workflow={{workflow}} \
    --branch={{branch}} \
    --limit={{limit}} \
    --repo ${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}

# Watch a specific workflow run
[group('CI/CD')]
gh-watch run_id="":
  #!/usr/bin/env bash
  if [ -z "{{run_id}}" ]; then
    echo "Getting latest workflow run..."
    RUN_ID=$(gh run list --workflow=ci.yaml --limit=1 --json databaseId -q '.[0].databaseId' \
      --repo ${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)})
    echo "Watching run: $RUN_ID"
    gh run watch $RUN_ID --repo ${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}
  else
    gh run watch {{run_id}} --repo ${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}
  fi

# View logs for a specific workflow run
[group('CI/CD')]
gh-logs run_id="" job="":
  #!/usr/bin/env bash
  REPO="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
  if [ -z "{{run_id}}" ]; then
    echo "Getting latest workflow run..."
    RUN_ID=$(gh run list --workflow=ci.yaml --limit=1 --json databaseId -q '.[0].databaseId' --repo $REPO)
  else
    RUN_ID="{{run_id}}"
  fi

  if [ -z "{{job}}" ]; then
    echo "Available jobs in run $RUN_ID:"
    gh run view $RUN_ID --repo $REPO --json jobs -q '.jobs[].name'
    echo ""
    echo "Viewing full run logs..."
    gh run view $RUN_ID --log --repo $REPO
  else
    echo "Viewing logs for job '{{job}}' in run $RUN_ID..."
    gh run view $RUN_ID --log --repo $REPO | grep -A 100 "{{job}}"
  fi

# Re-run a failed workflow
[group('CI/CD')]
gh-rerun run_id="" failed_only="true":
  #!/usr/bin/env bash
  REPO="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
  if [ -z "{{run_id}}" ]; then
    echo "Getting latest workflow run..."
    RUN_ID=$(gh run list --workflow=ci.yaml --limit=1 --json databaseId -q '.[0].databaseId' --repo $REPO)
  else
    RUN_ID="{{run_id}}"
  fi

  if [ "{{failed_only}}" = "true" ]; then
    echo "Re-running failed jobs in run $RUN_ID..."
    gh run rerun --failed $RUN_ID --repo $REPO
  else
    echo "Re-running all jobs in run $RUN_ID..."
    gh run rerun $RUN_ID --repo $REPO
  fi

# Cancel a running workflow
[group('CI/CD')]
gh-cancel run_id="":
  #!/usr/bin/env bash
  REPO="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
  if [ -z "{{run_id}}" ]; then
    echo "Getting latest workflow run..."
    RUN_ID=$(gh run list --workflow=ci.yaml --limit=1 --json databaseId -q '.[0].databaseId' --repo $REPO)
    echo "Canceling run: $RUN_ID"
    gh run cancel $RUN_ID --repo $REPO
  else
    gh run cancel {{run_id}} --repo $REPO
  fi

# List all packages in packages/ directory
[group('CI/CD')]
list-packages:
  @ls -1 packages/

# List packages in JSON format for CI matrix
[group('CI/CD')]
list-packages-json:
  #!/usr/bin/env bash
  cd packages
  packages=()
  for dir in */; do
    pkg_name="${dir%/}"
    if [ -f "$dir/package.json" ]; then
      packages+=("{\"name\":\"$pkg_name\",\"path\":\"packages/$pkg_name\"}")
    fi
  done
  echo "[$(IFS=,; echo "${packages[*]}")]"

# Validate package structure
[group('CI/CD')]
validate-package package:
  @echo "Validating package: {{ package }}"
  @test -d "packages/{{ package }}" || (echo "Package directory not found" && exit 1)
  @test -f "packages/{{ package }}/package.json" || (echo "package.json not found" && exit 1)
  @echo "✓ Package {{ package }} is valid"

## Cloudflare

# Preview the site locally with Cloudflare Workers
[group('cloudflare')]
cf-preview:
  bun run --filter '@ironstar/docs' preview

# Build and deploy the site to Cloudflare Workers
[group('cloudflare')]
cf-build-deploy: install
  bun run --filter '@ironstar/docs' deploy

# Deploy preview version with aliased preview URL for branch
[group('docs')]
docs-deploy-preview branch=`git branch --show-current`:
  #!/usr/bin/env bash
  set -euo pipefail
  cd packages/docs

  # Sanitize branch name for Cloudflare alias (must be valid subdomain component)
  # - Replace / and other non-alphanumeric chars with -
  # - Collapse consecutive hyphens, remove leading/trailing hyphens
  # - Truncate to 40 chars (safe for subdomain label limit of 63)
  SAFE_BRANCH=$(echo "{{branch}}" | tr '/' '-' | tr -c 'a-zA-Z0-9-' '-' | sed 's/--*/-/g; s/^-//; s/-$//' | cut -c1-40)

  # Capture git metadata (use 12-char SHA for tag - fits in 25 char limit, extremely collision-resistant)
  COMMIT_SHA=$(git rev-parse HEAD)
  COMMIT_TAG=$(git rev-parse --short=12 HEAD)
  COMMIT_SHORT=$(git rev-parse --short HEAD)
  COMMIT_MSG=$(git log -1 --pretty=format:'%s')
  GIT_STATUS=$(git diff-index --quiet HEAD -- && echo "clean" || echo "dirty")

  # Tag is 12-char SHA (deterministic, <= 25 chars, used to find this version on main)
  TAG="${COMMIT_TAG}"
  # Message includes full context for verification
  MESSAGE="[{{branch}}] ${COMMIT_MSG} (${COMMIT_TAG}, ${GIT_STATUS})"

  echo "Deploying preview for branch: {{branch}}"
  echo "Sanitized alias: b-${SAFE_BRANCH}"
  echo "Commit: ${COMMIT_SHORT} (${GIT_STATUS})"
  echo "Full SHA: ${COMMIT_SHA}"
  echo "Tag: ${COMMIT_TAG}"
  echo "Message: ${COMMIT_MSG}"
  echo ""

  # Export variables for use in sops exec-env
  export VERSION_TAG="${TAG}"
  export VERSION_MESSAGE="${MESSAGE}"
  export SAFE_BRANCH="${SAFE_BRANCH}"

  sops exec-env ../../vars/shared.yaml '
    echo "Building..."
    bun run build
    echo "Uploading version with preview alias and metadata..."
    bunx wrangler versions upload \
      --preview-alias "b-${SAFE_BRANCH}" \
      --tag "$VERSION_TAG" \
      --message "$VERSION_MESSAGE"
  '

  echo ""
  echo "✓ Version uploaded successfully"
  echo "  Tag: ${COMMIT_TAG}"
  echo "  Full SHA: ${COMMIT_SHA}"
  echo "  Message: ${MESSAGE}"
  echo "  Preview URL: https://b-${SAFE_BRANCH}-ironstar-docs.sciexp.workers.dev"

# Deploy to production (promote existing version or fallback to build+deploy)
[group('docs')]
docs-deploy-production:
  #!/usr/bin/env bash
  set -euo pipefail
  cd packages/docs

  # Get current commit tag (should match a previously uploaded version if fast-forward merged)
  CURRENT_SHA=$(git rev-parse HEAD)
  CURRENT_TAG=$(git rev-parse --short=12 HEAD)
  CURRENT_SHORT=$(git rev-parse --short HEAD)
  CURRENT_BRANCH=$(git branch --show-current)

  # Build deployment message (works in both CI and local)
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    # Running in GitHub Actions
    DEPLOYER="${GITHUB_ACTOR:-github-actions}"
    DEPLOY_CONTEXT="${GITHUB_WORKFLOW:-CI}"
    DEPLOY_MSG="Deployed by ${DEPLOYER} from ${CURRENT_BRANCH} via ${DEPLOY_CONTEXT}"
  else
    # Running locally
    DEPLOYER=$(whoami)
    DEPLOY_HOST=$(hostname -s)
    DEPLOY_MSG="Deployed by ${DEPLOYER} from ${CURRENT_BRANCH} on ${DEPLOY_HOST}"
  fi

  echo "Deploying to production from branch: ${CURRENT_BRANCH}"
  echo "Current commit: ${CURRENT_SHORT}"
  echo "Full SHA: ${CURRENT_SHA}"
  echo "Looking for existing version with tag: ${CURRENT_TAG}"
  echo "Deployment message: ${DEPLOY_MSG}"
  echo ""

  # Query for existing version with matching tag (take most recent if multiple)
  EXISTING_VERSION=$(sops exec-env ../../vars/shared.yaml \
    "bunx wrangler versions list --json" | \
    jq -r --arg tag "$CURRENT_TAG" \
    '.[] | select(.annotations["workers/tag"] == $tag) | .id' | head -1)

  if [ -n "$EXISTING_VERSION" ]; then
    echo "✓ Found existing version: ${EXISTING_VERSION}"
    echo "  This version was already built and tested in preview"
    echo "  Promoting to 100% production traffic..."
    echo ""

    # Export for use in sops exec-env
    export DEPLOYMENT_MESSAGE="${DEPLOY_MSG}"

    if sops exec-env ../../vars/shared.yaml "
      bunx wrangler versions deploy ${EXISTING_VERSION}@100% --yes --message \"\$DEPLOYMENT_MESSAGE\"
    "; then
      echo ""
      echo "✓ Successfully promoted version ${EXISTING_VERSION} to production"
      echo "  Tag: ${CURRENT_TAG}"
      echo "  Full SHA: ${CURRENT_SHA}"
      echo "  Deployed by: ${DEPLOY_MSG}"
      echo "  Production URL: https://ironstar.scientistexperience.net"
    else
      echo ""
      echo "✗ Failed to promote version ${EXISTING_VERSION}"
      echo "  Deployment was cancelled or failed"
      exit 1
    fi
  else
    echo "⚠ No existing version found with tag: ${CURRENT_TAG}"
    echo "  This should only happen if:"
    echo "    - This is the first deployment"
    echo "    - Commit was made directly on main (not recommended)"
    echo "    - Version was cleaned up (retention policy)"
    echo ""
    echo "  Falling back to direct build and deploy..."
    echo ""

    # Export for use in sops exec-env
    export DEPLOYMENT_MESSAGE="${DEPLOY_MSG}"

    sops exec-env ../../vars/shared.yaml '
      echo "Building..."
      bun run build
      echo "Deploying to production..."
      bunx wrangler deploy
    '

    echo ""
    echo "✓ Built and deployed to production"
    echo "  Full SHA: ${CURRENT_SHA}"
    echo "  Deployed by: ${DEPLOY_MSG}"
    echo "  Production URL: https://ironstar.scientistexperience.net"
  fi

# List recent versions
[group('docs')]
docs-versions limit="10":
  cd packages/docs && sops exec-env ../../vars/shared.yaml "bunx wrangler versions list --limit {{limit}}"

# View specific version details
[group('cloudflare')]
cf-version-view version_id:
  cd packages/docs && sops exec-env ../../vars/shared.yaml "bunx wrangler versions view {{version_id}}"

# Deploy specific version(s) with traffic split (gradual deployment)
[group('cloudflare')]
cf-versions-deploy:
  cd packages/docs && sops exec-env ../../vars/shared.yaml "bunx wrangler versions deploy"

# Tail live logs from Cloudflare Workers
[group('docs')]
docs-tail:
  cd packages/docs && sops exec-env ../../vars/shared.yaml "bunx wrangler tail"

# List deployments
[group('docs')]
docs-deployments:
  cd packages/docs && sops exec-env ../../vars/shared.yaml "bunx wrangler deployments list"

# Generate Cloudflare Worker types
# Note: --include-runtime=false works around wrangler 4.42.0 EPIPE bug
# Runtime types are only for IDE autocomplete; production builds don't need them
[group('cloudflare')]
cf-types:
  cd packages/docs && bun run cf-typegen --include-runtime=false

## Docs

# Start development server
[group('docs')]
docs-dev:
  cd packages/docs && bun run dev

# Build the documentation site
[group('docs')]
docs-build:
  cd packages/docs && bun run build

# Preview the built site
[group('docs')]
docs-preview:
  cd packages/docs && bun run preview

# Format documentation code
[group('docs')]
docs-format:
  cd packages/docs && bun run format

# Lint documentation code
[group('docs')]
docs-lint:
  cd packages/docs && bun run lint

# Check and fix documentation code
[group('docs')]
docs-check:
  cd packages/docs && bun run check:fix

# Validate documentation links
[group('docs')]
docs-linkcheck:
  cd packages/docs && bun run linkcheck

# Run all documentation tests
[group('docs')]
docs-test:
  cd packages/docs && bun run test

# Run documentation unit tests
[group('docs')]
docs-test-unit:
  cd packages/docs && bun run test:unit

# Run documentation E2E tests
[group('docs')]
docs-test-e2e:
  cd packages/docs && bun run test:e2e

# Generate documentation test coverage report
[group('docs')]
docs-test-coverage:
  cd packages/docs && bun run test:coverage

# Optimize favicon.svg with SVGO
[group('docs')]
docs-optimize-favicon:
  bunx svgo packages/docs/public/favicon.svg --multipass

## Nix

# Validate the Nix flake configuration (all checks including treefmt, nix-unit)
[group('nix')]
check:
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Running nix flake check..."
  echo ""
  echo "Note: nix-unit warnings are expected and harmless:"
  echo "  - 'unknown setting allowed-users/trusted-users' (daemon settings don't apply in pure eval)"
  echo "  - '--gc-roots-dir not specified' (nix-unit doesn't persist GC roots)"
  echo "  - 'input has an override for non-existent input self' (nix-unit internal mechanism)"
  echo "  - 'not writing modified lock file' (expected for read-only check)"
  echo ""
  if command -v nom &> /dev/null; then
    nix flake check --impure --log-format internal-json -v |& nom --json
  else
    nix flake check --impure
  fi

# Format all files with treefmt (via nix fmt)
[group('nix')]
fmt:
  nix fmt

# Check formatting without modifying
[group('nix')]
fmt-check:
  nix develop -c treefmt --check .

# Update all flake inputs to their latest versions
[group('nix')]
flake-update:
  nix flake update --impure

# Enter the Nix development shell
[group('nix')]
dev:
  nix develop

# Build the documentation package with Nix
[group('nix')]
nix-build:
  nix build .#docs

# Build package with nom and push to Cachix (with all dependencies)
[group('nix')]
cache package=".#ironstar":
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Building {{package}}..."
  STORE_PATH=$(nom build "{{package}}" --no-link --print-out-paths 2>&1 | tail -1)
  if [ -z "$STORE_PATH" ] || [ ! -e "$STORE_PATH" ]; then
    echo "Failed to build or get store path"
    exit 1
  fi
  echo "Built: $STORE_PATH"
  echo ""
  echo "Pushing to cachix with all dependencies..."
  nix-store --query --requisites --include-outputs "$STORE_PATH" | \
      sops exec-env vars/shared.yaml "cachix push \$CACHIX_CACHE_NAME"
  echo ""
  echo "Cached: $STORE_PATH"

# Build with automatic cachix push (watches store during build)
[group('nix')]
cache-watch package=".#ironstar":
  sops exec-env vars/shared.yaml "cachix watch-exec \$CACHIX_CACHE_NAME -- nom build {{package}}"

# Build and cache all flake packages for current system
[group('nix')]
cache-all:
  #!/usr/bin/env bash
  set -euo pipefail
  SYSTEM=$(nix eval --impure --raw --expr 'builtins.currentSystem')
  echo "Caching all packages for $SYSTEM..."
  for pkg in $(nix eval ".#packages.$SYSTEM" --apply 'builtins.attrNames' --json | jq -r '.[]'); do
    echo ""
    echo "━━━ Caching $pkg ━━━"
    just cache ".#packages.$SYSTEM.$pkg"
  done
  echo ""
  echo "All packages cached."

# Build and cache all flake checks for current system (includes deps)
[group('nix')]
cache-checks:
  #!/usr/bin/env bash
  set -euo pipefail
  SYSTEM=$(nix eval --impure --raw --expr 'builtins.currentSystem')
  echo "Caching all checks for $SYSTEM..."
  for check in $(nix eval ".#checks.$SYSTEM" --apply 'builtins.attrNames' --json | jq -r '.[]'); do
    echo ""
    echo "━━━ Caching $check ━━━"
    just cache ".#checks.$SYSTEM.$check"
  done
  echo ""
  echo "All checks cached."

# Test cachix push/pull with a simple derivation
[group('nix')]
test-cachix:
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Testing cachix push/pull..."
  STORE_PATH=$(nix build nixpkgs#hello --no-link --print-out-paths)
  echo "Built: $STORE_PATH"
  echo "Pushing to cachix..."
  sops exec-env vars/shared.yaml "cachix push \$CACHIX_CACHE_NAME $STORE_PATH"
  CACHE_NAME=$(sops exec-env vars/shared.yaml 'echo $CACHIX_CACHE_NAME')
  echo "Pushed. Verify at: https://app.cachix.org/cache/$CACHE_NAME"

## Rust

# Check Rust formatting (without modifying)
[group('rust')]
rust-fmt-check:
  cargo fmt --all -- --check

# Format Rust code
[group('rust')]
rust-fmt:
  cargo fmt --all

# Run Rust clippy lints
[group('rust')]
rust-clippy:
  cargo clippy --workspace --all-targets -- -D warnings

# Run Rust tests
[group('rust')]
rust-test:
  cargo nextest run --workspace

# Build Rust documentation
[group('rust')]
rust-doc:
  cargo doc --workspace --no-deps --document-private-items

# Build Rust workspace (release)
[group('rust')]
rust-build:
  cargo build --workspace --release

# Run all Rust checks (fmt, clippy, test)
[group('rust')]
rust-check: rust-fmt-check rust-clippy rust-test

# Build Rust package with Nix
[group('rust')]
rust-nix-build:
  nix build .#ironstar

## Spec (Idris2)

# Typecheck the Idris2 specification
[group('spec')]
spec-check:
  idris2 --typecheck spec/ironstar.ipkg

# Typecheck with timing (level 1: per-module, 2: phases, 3: per-definition)
[group('spec')]
spec-check-timing level="1": spec-clean
  idris2 --typecheck spec/ironstar.ipkg --timing {{ level }}

# Clean Idris2 build artifacts (rm -r because idris2 --clean doesn't remove TTC cache)
[group('spec')]
spec-clean:
  rm -rf spec/build/ttc

# Force full rebuild (clean + check)
[group('spec')]
spec-rebuild: spec-clean spec-check

# List modules in the spec package
[group('spec')]
spec-modules:
  @grep -E '^\s+\w+\.' spec/ironstar.ipkg | tr -d ' ,'

# List crates in JSON format for CI matrix
[group('CI/CD')]
list-crates-json:
  #!/usr/bin/env bash
  cd crates
  crates=()
  for dir in */; do
    crate_name="${dir%/}"
    if [ -f "$dir/Cargo.toml" ]; then
      crates+=("{\"name\":\"$crate_name\",\"path\":\"crates/$crate_name\"}")
    fi
  done
  echo "[$(IFS=,; echo "${crates[*]}")]"

## Release

# Test semantic release (dry run) for specific package
[group('release')]
test-release package="docs":
  cd packages/{{ package }} && bun run test-release

# Test semantic release for all packages
[group('release')]
test-release-all:
  @for pkg in packages/*; do \
    echo "Testing release for $pkg"; \
    cd "$pkg" && bun run test-release && cd ../..; \
  done

# Preview semantic-release version after merging current branch to target
[group('release')]
preview-version target="main" package="":
  #!/usr/bin/env bash
  set -euo pipefail
  if [ -n "{{package}}" ]; then
    ./scripts/preview-version.sh "{{target}}" "{{package}}"
  else
    ./scripts/preview-version.sh "{{target}}"
  fi

# Release specific package with semantic-release
[group('release')]
release-package package dry_run="false":
  #!/usr/bin/env bash
  set -euo pipefail
  cd packages/{{ package }}
  if [ "{{ dry_run }}" = "true" ]; then
    bun run test-release
  else
    echo "This will create a real release. Use dry_run=true for testing."
    bun run semantic-release
  fi

## Secrets

# Scan repository for secrets
[group('secrets')]
scan-secrets:
  gitleaks detect --verbose --redact

# Scan staged changes for secrets (pre-commit)
[group('secrets')]
scan-staged:
  gitleaks protect --staged --verbose --redact

# Show existing secrets using sops
[group('secrets')]
show-secrets:
  @echo "=== Shared secrets (vars/shared.yaml) ==="
  @sops -d vars/shared.yaml
  @echo

# Edit shared secrets file
[group('secrets')]
edit-secrets:
  @sops vars/shared.yaml

# Create a new sops encrypted file
[group('secrets')]
new-secret file:
  @sops {{ file }}

# Export secrets to dotenv format using sops
[group('secrets')]
export-secrets:
  @echo "# Exported from sops secrets" > .secrets.env
  @sops -d vars/shared.yaml | grep -E '^[A-Z_]+:' | sed 's/: /=/' >> .secrets.env
  @sort -u .secrets.env -o .secrets.env

# Run command with all shared secrets as environment variables
[group('secrets')]
run-with-secrets +command:
  @sops exec-env vars/shared.yaml '{{ command }}'

# Check secrets are available in sops environment
[group('secrets')]
check-secrets:
  @printf "Check sops environment for secrets\n\n"
  @sops exec-env vars/shared.yaml 'env | grep -E "GITHUB|CACHIX|CLOUDFLARE" | sed "s/=.*$/=***REDACTED***/"'

# Show specific secret value from shared secrets
[group('secrets')]
get-secret key:
  @sops -d vars/shared.yaml | grep "^{{ key }}:" | cut -d' ' -f2-

# Validate all sops encrypted files can be decrypted
[group('secrets')]
validate-secrets:
  @echo "Validating sops encrypted files..."
  @for file in $(find vars -name "*.yaml" -o -name "*.json"); do \
    echo "Testing: $file"; \
    sops -d "$file" > /dev/null && echo "  ✅ Valid" || echo "  ❌ Failed"; \
  done

# Initialize sops age key for new developers
[group('secrets')]
sops-init:
  @echo "Checking sops configuration..."
  @if [ ! -f ~/.config/sops/age/keys.txt ]; then \
    echo "Generating age key..."; \
    mkdir -p ~/.config/sops/age; \
    age-keygen -o ~/.config/sops/age/keys.txt; \
    echo ""; \
    echo "✅ Age key generated. Add this public key to .sops.yaml:"; \
    grep "public key:" ~/.config/sops/age/keys.txt; \
  else \
    echo "✅ Age key already exists"; \
    grep "public key:" ~/.config/sops/age/keys.txt; \
  fi

# Add existing age key to local configuration
[group('secrets')]
sops-add-key:
  #!/usr/bin/env bash
  set -euo pipefail

  # Ensure keys.txt exists and has proper permissions
  mkdir -p ~/.config/sops/age
  touch ~/.config/sops/age/keys.txt
  chmod 600 ~/.config/sops/age/keys.txt

  # Prompt for key description
  printf "Enter age key description (e.g., 'ironstar [dev|ci]'): "
  read -r key_description
  [[ -z "${key_description}" ]] && { echo "❌ Description cannot be empty"; exit 1; }

  # Prompt for public key
  printf "Enter age public key (age1...): "
  read -r public_key
  if [[ ! "${public_key}" =~ ^age1[a-z0-9]{58}$ ]]; then
    echo "❌ Invalid age public key format (must start with 'age1' and be 62 chars)"
    exit 1
  fi

  # Prompt for private key (hidden input)
  printf "Enter age private key (AGE-SECRET-KEY-...): "
  read -rs private_key
  echo  # New line after hidden input
  if [[ ! "${private_key}" =~ ^AGE-SECRET-KEY-[A-Z0-9]{59}$ ]]; then
    echo "❌ Invalid age private key format"
    exit 1
  fi

  # Check if key already exists
  if grep -q "${private_key}" ~/.config/sops/age/keys.txt 2>/dev/null; then
    echo "⚠️  This private key already exists in keys.txt"
    exit 1
  fi

  # Append to keys.txt with proper formatting
  {
    echo ""
    echo "# ${key_description}"
    echo "# public key: ${public_key}"
    echo "${private_key}"
  } >> ~/.config/sops/age/keys.txt

  echo "✅ Age key added successfully for: ${key_description}"
  echo "   Public key: ${public_key}"

# Add or update a secret non-interactively
[group('secrets')]
set-secret secret_name secret_value:
  @sops set vars/shared.yaml '["{{ secret_name }}"]' '"{{ secret_value }}"'
  @echo "✅ {{ secret_name }} has been set/updated"

# Rotate a specific secret interactively
[group('secrets')]
rotate-secret secret_name:
  @echo "Rotating {{ secret_name }}..."
  @echo "Enter new value for {{ secret_name }}:"
  @read -s NEW_VALUE && \
    sops set vars/shared.yaml '["{{ secret_name }}"]' "\"$NEW_VALUE\"" && \
    echo "✅ {{ secret_name }} rotated successfully"

# Update keys for existing secrets files after adding new recipients
[group('secrets')]
updatekeys:
  @for file in $(find vars -name "*.yaml" -o -name "*.json"); do \
    echo "Updating keys for: $file"; \
    sops updatekeys -y "$file"; \
  done
  @echo "✅ Keys updated for all secrets files"

# Bootstrap or rotate SOPS age keys (unified recipe for first-time and rotation)
[group('secrets')]
sops-bootstrap role='dev' method='ssh':
  @scripts/sops-bootstrap.sh "{{ role }}" "{{ method }}"

# Upload SOPS_AGE_KEY to GitHub (separate from other secrets to avoid chicken-and-egg)
[group('secrets')]
sops-upload-github-key repo="":
  @scripts/sops-upload-github-key.sh "{{ repo }}"

# Enumerate required secrets by parsing GitHub workflows
[group('secrets')]
sops-check-requirements:
  @scripts/sops-check-requirements.sh

# Comprehensive GitHub setup (uploads everything except SOPS_AGE_KEY)
[group('secrets')]
sops-setup-github repo="":
  @scripts/sops-setup-github.sh "{{ repo }}"

# Finalize key rotation by removing old keys
[group('secrets')]
sops-finalize-rotation role='dev':
  @scripts/sops-finalize-rotation.sh "{{ role }}"

# Quick rotation workflow (combines bootstrap + finalize after verification)
[group('secrets')]
sops-rotate role='dev' method='ssh':
  @scripts/sops-rotate.sh "{{ role }}" "{{ method }}"


## Slides

# List available slide decks
[group('slides')]
slides-list:
  #!/usr/bin/env bash
  echo "Available slide decks:"
  for dir in docs/slides/*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir")
    echo "  - $name"
  done

# Build all slide decks to PDF
[group('slides')]
slides-build:
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Building all slide decks..."
  for dir in docs/slides/*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir")
    if [ -f "$dir/main.typ" ]; then
      echo "  $name -> $dir$name.pdf"
      typst compile "$dir/main.typ" "$dir/$name.pdf"
    fi
  done
  echo "Done."

# Build specific slide deck
[group('slides')]
slides-compile name:
  #!/usr/bin/env bash
  set -euo pipefail
  dir="docs/slides/{{ name }}"
  if [ ! -f "$dir/main.typ" ]; then
    echo "Error: $dir/main.typ not found"
    exit 1
  fi
  echo "Compiling {{ name }}..."
  typst compile "$dir/main.typ" "$dir/{{ name }}.pdf"
  echo "Output: $dir/{{ name }}.pdf"

# Watch slide deck for changes (auto-recompile)
[group('slides')]
slides-watch name:
  typst watch "docs/slides/{{ name }}/main.typ" "docs/slides/{{ name }}/{{ name }}.pdf"

# Export slide deck to PNG images (one per slide)
[group('slides')]
slides-export-png name:
  #!/usr/bin/env bash
  set -euo pipefail
  dir="docs/slides/{{ name }}"
  out="$dir/png"
  mkdir -p "$out"
  echo "Exporting {{ name }} to PNG..."
  typst compile "$dir/main.typ" "$out/slide-{n}.png"
  echo "Output: $out/slide-*.png"

# Export slide deck to SVG (for web embedding)
[group('slides')]
slides-export-svg name:
  #!/usr/bin/env bash
  set -euo pipefail
  dir="docs/slides/{{ name }}"
  out="$dir/svg"
  mkdir -p "$out"
  echo "Exporting {{ name }} to SVG..."
  typst compile --format svg "$dir/main.typ" "$out/slide-{n}.svg"
  echo "Output: $out/svg/slide-*.svg"


## Testing

# Run all tests in all packages
[group('testing')]
test:
  bun run --filter '@ironstar/*' test

# Run tests in specific package
[group('testing')]
test-pkg package:
  bun run --filter '@ironstar/{{ package }}' test

# Run unit tests in docs
[group('testing')]
test-unit:
  bun run --filter '@ironstar/docs' test:unit

# Run E2E tests in docs
[group('testing')]
test-e2e:
  bun run --filter '@ironstar/docs' test:e2e

# Run vitest in watch mode
[group('testing')]
test-watch:
  bun run --filter '@ironstar/docs' test:watch

# Run playwright in UI mode
[group('testing')]
test-ui:
  bun run --filter '@ironstar/docs' test:ui

# Generate test coverage report
[group('testing')]
test-coverage:
  bun run --filter '@ironstar/docs' test:coverage

# Run all tests for specific package (CI workflow)
[group('testing')]
test-package package:
  cd packages/{{ package }} && bun run test:unit && bun run test:coverage && bun run test:e2e

# Install playwright browsers (only needed outside Nix environment)
# The Nix devshell provides browsers via playwright-driver.browsers
# See: nix/modules/devshell.nix lines 36, 47-48
[group('testing')]
playwright-install:
  @echo "Note: When using 'nix develop', browsers are provided by Nix"
  @echo "This command is only needed in non-Nix environments"
  bunx playwright install --with-deps

## Template

# Verify template functionality by creating and checking a test project
# Uses system temp directory to avoid nested git repo issues
# Falls back to nix run if om command is not installed
[group('template')]
template-verify:
  #!/usr/bin/env bash
  set -euo pipefail
  FLAKE_DIR="{{ justfile_directory() }}"
  TEMP_DIR=$(mktemp -d)
  trap 'rm -rf "$TEMP_DIR"' EXIT
  echo "Verifying template from $FLAKE_DIR"
  echo "Using temp directory: $TEMP_DIR"

  # Test parameters for non-interactive verification
  TEST_PARAMS='{
    "project-name": "test-project",
    "npm-scope": "@test-project",
    "git-org": "test-org",
    "author": "Test Author",
    "author-email": "test@example.com",
    "project-description": "Test project for template verification",
    "cloudflare-worker-name": "test-docs",
    "production-url": "test.example.com"
  }'

  # Use om if available, otherwise fall back to nix run with --accept-flake-config
  # to use omnix binary cache (https://cache.nixos.asia/oss) and avoid recompilation
  if command -v om &> /dev/null; then
    echo "Using installed om command"
    om init "$FLAKE_DIR" -o "$TEMP_DIR/test-project" --non-interactive --params "$TEST_PARAMS"
  else
    echo "om not found, using nix run (fetching from cache.nixos.asia)"
    nix --accept-flake-config run github:juspay/omnix/v1.3.0 -- \
      init "$FLAKE_DIR" -o "$TEMP_DIR/test-project" --non-interactive --params "$TEST_PARAMS"
  fi

  cd "$TEMP_DIR/test-project"
  git init
  git config user.email "test@example.com"
  git config user.name "Test User"
  git commit --allow-empty -m "initial commit (empty)"
  git add .
  nix flake check --accept-flake-config
  echo "Template verification successful"
