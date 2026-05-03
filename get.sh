#!/usr/bin/env bash
# ============================================================
#  Enspect — One-command Installer (macOS / Linux)
#  Usage: curl -fsSL https://raw.githubusercontent.com/codewithalphadotcom/Enspect/main/get.sh | bash
# ============================================================
set -euo pipefail

REPO_URL="https://github.com/codewithalphadotcom/Enspect.git"

# ── Colors ───────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
ORANGE='\033[38;5;214m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

info()    { echo -e "  ${ORANGE}•${NC} $1"; }
success() { echo -e "  ${GREEN}✓${NC} $1"; }
warn()    { echo -e "  ${YELLOW}!${NC} $1"; }
error()   { echo -e "  ${RED}✗${NC} $1"; exit 1; }
step()    { echo -e "\n${BOLD}${CYAN}▶ $1${NC}"; }

print_banner() {
  echo ""
  echo -e "${BOLD}${ORANGE}  ███████╗███╗  ██╗███████╗██████╗ ███████╗ ██████╗████████╗${NC}"
  echo -e "${BOLD}${ORANGE}  ██╔════╝████╗ ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝${NC}"
  echo -e "${BOLD}${ORANGE}  █████╗  ██╔██╗██║███████╗██████╔╝█████╗  ██║        ██║${NC}"
  echo -e "${BOLD}${ORANGE}  ██╔══╝  ██║╚████║╚════██║██╔═══╝ ██╔══╝  ██║        ██║${NC}"
  echo -e "${BOLD}${ORANGE}  ███████╗██║ ╚███║███████║██║     ███████╗╚██████╗   ██║${NC}"
  echo -e "${BOLD}${ORANGE}  ╚══════╝╚═╝  ╚══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝${NC}"
  echo ""
  echo -e "  ${DIM}Environment Variable Auditor — Remote Installer${NC}"
  echo ""
}

# ── Platform Guard ────────────────────────────────────────────
check_platform() {
  local os
  os="$(uname -s)"
  case "$os" in
    Linux*|Darwin*) : ;;
    CYGWIN*|MINGW*|MSYS*)
      error "Windows detected. Use the PowerShell installer instead:\n\n  ${CYAN}irm https://raw.githubusercontent.com/codewithalphadotcom/Enspect/main/get.ps1 | iex${NC}"
      ;;
    *)
      error "Unsupported OS: $os"
      ;;
  esac
}

# ── Dependency Check ──────────────────────────────────────────
check_deps() {
  step "Checking dependencies"
  if ! command -v git &>/dev/null; then
    error "git is required but not installed.\n\n  macOS:  brew install git\n  Linux:  sudo apt install git  (or your distro's equivalent)"
  fi
  success "git found: $(git --version)"
}

# ── Clone ─────────────────────────────────────────────────────
clone_repo() {
  TMP_DIR="$(mktemp -d)"
  # Always clean up the temp dir on exit (success or error)
  trap 'rm -rf "$TMP_DIR"' EXIT

  step "Fetching Enspect"
  info "Cloning into temporary directory..."
  git clone --depth=1 "$REPO_URL" "$TMP_DIR" --quiet
  success "Cloned successfully"
}

# ── Hand off to install.sh ────────────────────────────────────
run_installer() {
  step "Starting installer"
  info "Handing off to install.sh inside the cloned repo..."
  cd "$TMP_DIR"
  export ENSPECT_FROM_REMOTE=1
  bash install.sh
}

# ── Main ──────────────────────────────────────────────────────
main() {
  print_banner
  check_platform
  check_deps
  clone_repo
  run_installer
}

main "$@"
