#!/usr/bin/env bash
# ironstar bootstrap script
# https://github.com/sciexp/ironstar
#
# One-liner installation:
#   bash <(curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/sciexp/ironstar/main/scripts/bootstrap.sh)

set -euo pipefail

# Script metadata
readonly VERSION="1.0.0"
readonly NIX_INSTALLER_VERSION="3.11.3"
SCRIPT_NAME=""
SCRIPT_NAME="$(basename "$0")"
readonly SCRIPT_NAME

# Global flags
DRY_RUN=false
VERBOSE=false
QUIET=false
AUTO_YES=false
SKIP_DIRENV=false
SKIP_AGE_KEY=false

# Color codes (will be empty if not a TTY)
RED=""
GREEN=""
YELLOW=""
BLUE=""
BOLD=""
RESET=""

# Exit codes
readonly EXIT_SUCCESS=0
readonly EXIT_ERROR=1
readonly EXIT_INVALID_ARGS=2

# Cleanup tracking
CLEANUP_FILES=()
CLEANUP_DIRS=()

################################################################################
# Utility Functions
################################################################################

# setup_colors: Initialize color codes if output is to a TTY
setup_colors() {
  if [[ -t 1 ]] && [[ "${NO_COLOR:-}" != "1" ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    RESET='\033[0m'
  fi
}

# info: Print informational message with blue prefix
info() {
  if [[ "${QUIET}" != "true" ]]; then
    printf "${BLUE}==>${RESET} ${BOLD}%s${RESET}\n" "$*" >&2
  fi
}

# success: Print success message with green prefix
success() {
  if [[ "${QUIET}" != "true" ]]; then
    printf "${GREEN}==>${RESET} ${BOLD}%s${RESET}\n" "$*" >&2
  fi
}

# warn: Print warning message with yellow prefix
warn() {
  printf "${YELLOW}Warning:${RESET} %s\n" "$*" >&2
}

# error: Print error message with red prefix to stderr
error() {
  printf "${RED}Error:${RESET} %s\n" "$*" >&2
}

# die: Print error message and exit with error code
die() {
  error "$@"
  exit "${EXIT_ERROR}"
}

# verbose: Print message only if verbose mode is enabled
verbose() {
  if [[ "${VERBOSE}" == "true" ]]; then
    printf "${BLUE}[verbose]${RESET} %s\n" "$*" >&2
  fi
}

# run: Execute command, respecting dry-run mode
# In dry-run, prints the command with [DRY-RUN] prefix
# Otherwise, executes the command
run() {
  if [[ "${DRY_RUN}" == "true" ]]; then
    printf "${YELLOW}[DRY-RUN]${RESET} %s\n" "$*" >&2
  else
    verbose "Executing: $*"
    if [[ "${VERBOSE}" == "true" ]]; then
      "$@"
    else
      "$@" > /dev/null 2>&1
    fi
  fi
}

# run_quiet: Execute command, suppress stdout unless verbose
run_quiet() {
  if [[ "${DRY_RUN}" == "true" ]]; then
    printf "${YELLOW}[DRY-RUN]${RESET} %s\n" "$*" >&2
  else
    verbose "Executing: $*"
    if [[ "${VERBOSE}" == "true" ]]; then
      "$@"
    else
      "$@" 2>&1 | grep -v "^$" || true
    fi
  fi
}

# confirm: Prompt user for yes/no confirmation
# Returns 0 for yes, 1 for no
# Auto-returns yes if AUTO_YES is true or if not a TTY
confirm() {
  local prompt="${1}"
  local default="${2:-n}"

  # Auto-confirm in non-interactive mode or if --yes flag is set
  if [[ "${AUTO_YES}" == "true" ]] || ! [[ -t 0 ]]; then
    verbose "Auto-confirming: ${prompt}"
    return 0
  fi

  local yn_prompt
  if [[ "${default}" == "y" ]]; then
    yn_prompt="[Y/n]"
  else
    yn_prompt="[y/N]"
  fi

  while true; do
    printf "${BOLD}%s${RESET} %s " "${prompt}" "${yn_prompt}" >&2
    read -r response

    # Use default if response is empty
    if [[ -z "${response}" ]]; then
      response="${default}"
    fi

    case "${response}" in
      [Yy]|[Yy][Ee][Ss])
        return 0
        ;;
      [Nn]|[Nn][Oo])
        return 1
        ;;
      *)
        warn "Please answer yes or no"
        ;;
    esac
  done
}

# cleanup: Remove temporary files and directories
cleanup() {
  local exit_code=$?

  if [[ "${#CLEANUP_FILES[@]}" -gt 0 ]]; then
    verbose "Cleaning up temporary files"
    for file in "${CLEANUP_FILES[@]}"; do
      if [[ -f "${file}" ]]; then
        rm -f "${file}" 2>/dev/null || true
      fi
    done
  fi

  if [[ "${#CLEANUP_DIRS[@]}" -gt 0 ]]; then
    verbose "Cleaning up temporary directories"
    for dir in "${CLEANUP_DIRS[@]}"; do
      if [[ -d "${dir}" ]]; then
        rm -rf "${dir}" 2>/dev/null || true
      fi
    done
  fi

  exit "${exit_code}"
}

# setup_traps: Register cleanup handlers
setup_traps() {
  trap cleanup EXIT
  trap 'error "Script interrupted"; exit 130' INT
  trap 'error "Script terminated"; exit 143' TERM
  trap 'error "Script error on line $LINENO"; exit 1' ERR
}

################################################################################
# Platform Detection
################################################################################

# detect_platform: Detect OS and architecture
# Sets global variables: OS_TYPE, ARCH, NIX_PLATFORM
detect_platform() {
  local os arch

  # Detect OS
  os="$(uname -s)"
  case "${os}" in
    Linux)
      OS_TYPE="linux"
      ;;
    Darwin)
      OS_TYPE="darwin"
      ;;
    *)
      die "Unsupported operating system: ${os}"
      ;;
  esac

  # Detect architecture
  arch="$(uname -m)"
  case "${arch}" in
    x86_64|amd64)
      ARCH="x86_64"
      ;;
    arm64|aarch64)
      ARCH="aarch64"
      ;;
    *)
      die "Unsupported architecture: ${arch}"
      ;;
  esac

  # Construct nix platform identifier
  NIX_PLATFORM="${ARCH}-${OS_TYPE}"

  verbose "Detected platform: ${NIX_PLATFORM}"
}

################################################################################
# Installation Functions
################################################################################

# check_xcode_cli_tools: Check if Xcode Command Line Tools are installed
check_xcode_cli_tools() {
  if xcode-select -p &>/dev/null; then
    verbose "Xcode Command Line Tools already installed"
    return 0
  else
    return 1
  fi
}

# install_xcode_cli_tools: Install Xcode Command Line Tools
install_xcode_cli_tools() {
  info "Installing Xcode Command Line Tools"
  info "You may see a GUI prompt to confirm installation"

  if [[ "${DRY_RUN}" == "true" ]]; then
    printf "%s[DRY-RUN]%s xcode-select --install\n" "${YELLOW}" "${RESET}" >&2
    return 0
  fi

  # Trigger installation dialog
  xcode-select --install 2>/dev/null || true

  # Wait for installation to complete
  info "Waiting for Xcode Command Line Tools installation to complete"
  until xcode-select -p &>/dev/null; do
    sleep 5
  done

  success "Xcode Command Line Tools installed"
}

# check_homebrew: Check if Homebrew is installed
check_homebrew() {
  local brew_path
  if command -v brew &>/dev/null; then
    brew_path=$(command -v brew) || true
    verbose "Homebrew already installed at ${brew_path}"
    return 0
  else
    return 1
  fi
}

# install_homebrew: Install Homebrew
install_homebrew() {
  info "Installing Homebrew"
  info "This will download and run the Homebrew installation script"

  if [[ "${DRY_RUN}" == "true" ]]; then
    printf "%s[DRY-RUN]%s bash <(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\n" "${YELLOW}" "${RESET}" >&2
    return 0
  fi

  # Download and run Homebrew installer
  # Use NONINTERACTIVE=1 if auto-yes mode
  local install_script
  install_script=$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh) || die "Failed to download Homebrew installer"

  if [[ "${AUTO_YES}" == "true" ]]; then
    NONINTERACTIVE=1 bash -c "${install_script}"
  else
    bash -c "${install_script}"
  fi

  success "Homebrew installed"
}

# setup_darwin_prerequisites: Install macOS prerequisites
setup_darwin_prerequisites() {
  info "Setting up macOS prerequisites"

  # Check Xcode Command Line Tools
  if ! check_xcode_cli_tools; then
    install_xcode_cli_tools
  else
    success "Xcode Command Line Tools already installed"
  fi

  # Check Homebrew (optional)
  if ! check_homebrew; then
    if confirm "Install Homebrew? (recommended for additional tools)" "n"; then
      install_homebrew
    else
      info "Skipping Homebrew installation"
    fi
  else
    success "Homebrew already installed"
  fi
}

# check_nix: Check if Nix is already installed
check_nix() {
  local nix_path
  if command -v nix &>/dev/null; then
    nix_path=$(command -v nix) || true
    verbose "Nix already installed at ${nix_path}"
    return 0
  else
    return 1
  fi
}

# install_nix: Install Nix using the experimental-nix-installer
install_nix() {
  local installer_url="https://github.com/NixOS/experimental-nix-installer/releases/download/${NIX_INSTALLER_VERSION}/nix-installer-${NIX_PLATFORM}"
  local installer_path="/tmp/nix-installer-${NIX_PLATFORM}"

  info "Installing Nix ${NIX_INSTALLER_VERSION} for ${NIX_PLATFORM}"

  if [[ "${DRY_RUN}" == "true" ]]; then
    printf "${YELLOW}[DRY-RUN]${RESET} curl -L -o %s %s\n" "${installer_path}" "${installer_url}" >&2
    printf "${YELLOW}[DRY-RUN]${RESET} chmod +x %s\n" "${installer_path}" >&2
    printf "${YELLOW}[DRY-RUN]${RESET} %s install --no-confirm --extra-conf \"trusted-users = root @admin @wheel\"\n" "${installer_path}" >&2
    return 0
  fi

  # Download installer
  verbose "Downloading Nix installer from ${installer_url}"
  curl -L -o "${installer_path}" "${installer_url}" || die "Failed to download Nix installer"
  CLEANUP_FILES+=("${installer_path}")

  # Make executable
  chmod +x "${installer_path}" || die "Failed to make installer executable"

  # Run installer
  verbose "Running Nix installer"
  if [[ "${AUTO_YES}" == "true" ]]; then
    "${installer_path}" install --no-confirm --extra-conf "trusted-users = root @admin @wheel" || die "Nix installation failed"
  else
    "${installer_path}" install --extra-conf "trusted-users = root @admin @wheel" || die "Nix installation failed"
  fi

  success "Nix installed successfully"
}

# source_nix_profile: Source the Nix profile to make nix command available
source_nix_profile() {
  local nix_profile="/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh"

  if [[ -f "${nix_profile}" ]]; then
    verbose "Sourcing Nix profile: ${nix_profile}"
    # shellcheck source=/dev/null
    . "${nix_profile}"
  else
    warn "Nix profile not found at ${nix_profile}"
    warn "You may need to start a new shell session"
  fi
}

# check_direnv: Check if direnv is installed
check_direnv() {
  local direnv_path
  if command -v direnv &>/dev/null; then
    direnv_path=$(command -v direnv) || true
    verbose "direnv already installed at ${direnv_path}"
    return 0
  else
    return 1
  fi
}

# install_direnv: Install direnv using Nix
install_direnv() {
  info "Installing direnv"

  if [[ "${DRY_RUN}" == "true" ]]; then
    printf "%s[DRY-RUN]%s nix profile install nixpkgs#direnv\n" "${YELLOW}" "${RESET}" >&2
    return 0
  fi

  # Ensure nix is available
  if ! command -v nix &>/dev/null; then
    source_nix_profile
  fi

  # Install direnv
  verbose "Running: nix profile install nixpkgs#direnv"
  nix profile install nixpkgs#direnv || die "Failed to install direnv"

  success "direnv installed successfully"

  # Provide shell integration instructions
  info ""
  info "To enable direnv shell integration, add to your shell configuration:"
  info ""
  if [[ "${OS_TYPE}" == "darwin" ]]; then
    info "  For bash (~/.bash_profile or ~/.bashrc):"
    info "    eval \"\$(direnv hook bash)\""
    info ""
    info "  For zsh (~/.zshrc):"
    info "    eval \"\$(direnv hook zsh)\""
  else
    info "  For bash (~/.bashrc):"
    info "    eval \"\$(direnv hook bash)\""
    info ""
    info "  For zsh (~/.zshrc):"
    info "    eval \"\$(direnv hook zsh)\""
  fi
  info ""
}

# check_age_key: Check if age key exists
check_age_key() {
  local age_key_path="${HOME}/.config/sops/age/keys.txt"

  if [[ -f "${age_key_path}" ]]; then
    verbose "Age key already exists at ${age_key_path}"
    return 0
  else
    return 1
  fi
}

# generate_age_key: Generate age key for SOPS secrets
generate_age_key() {
  local age_key_path="${HOME}/.config/sops/age/keys.txt"
  local age_key_dir
  age_key_dir="$(dirname "${age_key_path}")"

  info "Generating age key for SOPS secrets"

  if [[ "${DRY_RUN}" == "true" ]]; then
    printf "${YELLOW}[DRY-RUN]${RESET} mkdir -p %s\n" "${age_key_dir}" >&2
    printf "${YELLOW}[DRY-RUN]${RESET} nix shell nixpkgs#age -c age-keygen -o %s\n" "${age_key_path}" >&2
    return 0
  fi

  # Ensure nix is available
  if ! command -v nix &>/dev/null; then
    source_nix_profile
  fi

  # Create directory
  mkdir -p "${age_key_dir}" || die "Failed to create age key directory"

  # Generate key
  verbose "Generating age key at ${age_key_path}"
  nix shell nixpkgs#age -c age-keygen -o "${age_key_path}" || die "Failed to generate age key"

  # Extract and display public key
  local public_key
  public_key=$(grep -E '^# public key: ' "${age_key_path}" | cut -d: -f2 | tr -d ' ')

  success "Age key generated successfully"
  info ""
  info "Your age public key (safe to share):"
  info "  ${public_key}"
  info ""
  info "Private key saved to: ${age_key_path}"
  info ""
}

################################################################################
# Main Installation Flow
################################################################################

# print_banner: Display script banner with version and platform info
print_banner() {
  if [[ "${QUIET}" != "true" ]]; then
    echo ""
    printf '%b\n' "${BOLD}ironstar bootstrap${RESET}"
    echo "Version: ${VERSION}"
    echo "Platform: ${NIX_PLATFORM}"
    if [[ "${DRY_RUN}" == "true" ]]; then
      printf '%b\n' "${YELLOW}Mode: DRY RUN${RESET}"
    fi
    echo ""
  fi
}

# print_success_message: Display final success message with next steps
print_success_message() {
  echo ""
  success "Bootstrap completed successfully"
  echo ""
  info "Next steps:"
  echo ""
  echo "  1. Start a new shell session or source your shell configuration"
  echo ""
  if [[ "${OS_TYPE}" == "darwin" ]]; then
    echo "     source ~/.bash_profile  # or ~/.zshrc"
  else
    echo "     source ~/.bashrc  # or ~/.zshrc"
  fi
  echo ""
  echo "  2. Clone the repository (if not already done):"
  echo ""
  echo "     git clone https://github.com/sciexp/ironstar.git"
  echo "     cd ironstar"
  echo ""
  echo "  3. Enter the development environment:"
  echo ""
  echo "     nix develop"
  echo ""
  echo "     Or if direnv is configured:"
  echo "     direnv allow"
  echo ""
  echo "  4. Install dependencies and start developing:"
  echo ""
  echo "     bun install"
  echo "     just --list"
  echo ""
}

# run_bootstrap: Execute the main bootstrap flow
run_bootstrap() {
  print_banner

  # Step 1: Platform-specific prerequisites
  if [[ "${OS_TYPE}" == "darwin" ]]; then
    setup_darwin_prerequisites
  else
    verbose "Skipping Darwin prerequisites on Linux"
  fi

  # Step 2: Install Nix
  if check_nix; then
    success "Nix already installed"
    local nix_version
    nix_version=$(nix --version 2>&1 | head -n 1)
    info "Nix version: ${nix_version}"
  else
    install_nix
    source_nix_profile
  fi

  # Step 3: Install direnv
  if [[ "${SKIP_DIRENV}" != "true" ]]; then
    if check_direnv; then
      success "direnv already installed"
    else
      install_direnv
    fi
  else
    info "Skipping direnv installation (--no-direnv)"
  fi

  # Step 4: Generate age key
  if [[ "${SKIP_AGE_KEY}" != "true" ]]; then
    if check_age_key; then
      success "Age key already exists"
    else
      generate_age_key
    fi
  else
    info "Skipping age key generation (--no-age-key)"
  fi

  print_success_message
}

################################################################################
# Argument Parsing
################################################################################

# usage: Print help message
usage() {
  cat <<EOF
Usage: ${SCRIPT_NAME} [OPTIONS]

Bootstrap script for ironstar development environment.
Installs Nix, direnv, and generates age keys for SOPS secrets.

Options:
  -h, --help         Show this help message and exit
  -n, --dry-run      Print commands without executing (implies --verbose)
  -y, --yes          Skip all confirmation prompts
  -v, --verbose      Show detailed output
  -q, --quiet        Minimal output (errors only)
  --no-direnv        Skip direnv installation
  --no-age-key       Skip age key generation for secrets
  --version          Show script version and exit

Examples:
  ${SCRIPT_NAME}                    # Interactive installation
  ${SCRIPT_NAME} --yes              # Non-interactive (auto-confirm all prompts)
  ${SCRIPT_NAME} --dry-run          # Preview what would be installed
  ${SCRIPT_NAME} --no-age-key       # Skip age key generation

For more information, visit:
  https://github.com/sciexp/ironstar

EOF
}

# parse_args: Parse command-line arguments
parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -h|--help)
        usage
        exit "${EXIT_SUCCESS}"
        ;;
      -n|--dry-run)
        DRY_RUN=true
        VERBOSE=true  # Dry-run implies verbose
        shift
        ;;
      -y|--yes)
        AUTO_YES=true
        shift
        ;;
      -v|--verbose)
        VERBOSE=true
        shift
        ;;
      -q|--quiet)
        QUIET=true
        shift
        ;;
      --no-direnv)
        SKIP_DIRENV=true
        shift
        ;;
      --no-age-key)
        SKIP_AGE_KEY=true
        shift
        ;;
      --version)
        echo "ironstar bootstrap v${VERSION}"
        exit "${EXIT_SUCCESS}"
        ;;
      *)
        error "Unknown option: $1"
        echo ""
        usage
        exit "${EXIT_INVALID_ARGS}"
        ;;
    esac
  done

  # Validate flag combinations
  if [[ "${QUIET}" == "true" ]] && [[ "${VERBOSE}" == "true" ]]; then
    error "Cannot use --quiet and --verbose together"
    exit "${EXIT_INVALID_ARGS}"
  fi
}

################################################################################
# Main Entry Point
################################################################################

main() {
  # Initialize
  setup_colors
  setup_traps
  parse_args "$@"

  # Detect platform
  detect_platform

  # Run bootstrap
  run_bootstrap

  exit "${EXIT_SUCCESS}"
}

# Run main function
main "$@"
