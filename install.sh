#!/usr/bin/env bash
# ============================================================
#  Enspect — Install Script
#  https://github.com/codewithalphadotcom/Enspect
# ============================================================
set -euo pipefail

# ── Colors ───────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# ── Config ───────────────────────────────────────────────────
BINARY_NAME="enspect"
REPO="codewithalphadotcom/Enspect"
REPO_URL="https://github.com/${REPO}"
VERSION="${ENSPECT_VERSION:-latest}"

# ── Helpers ──────────────────────────────────────────────────
info()    { echo -e "  ${BLUE}•${NC} $1"; }
success() { echo -e "  ${GREEN}✓${NC} $1"; }
warn()    { echo -e "  ${YELLOW}!${NC} $1"; }
error()   { echo -e "  ${RED}✗${NC} $1"; exit 1; }
step()    { echo -e "\n${BOLD}${CYAN}▶ $1${NC}"; }

print_banner() {
  echo ""
  echo -e "${BOLD}${BLUE}  ███████╗███╗  ██╗███████╗██████╗ ███████╗ ██████╗████████╗${NC}"
  echo -e "${BOLD}${BLUE}  ██╔════╝████╗ ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝${NC}"
  echo -e "${BOLD}${BLUE}  █████╗  ██╔██╗██║███████╗██████╔╝█████╗  ██║        ██║${NC}"
  echo -e "${BOLD}${BLUE}  ██╔══╝  ██║╚████║╚════██║██╔═══╝ ██╔══╝  ██║        ██║${NC}"
  echo -e "${BOLD}${BLUE}  ███████╗██║ ╚███║███████║██║     ███████╗╚██████╗   ██║${NC}"
  echo -e "${BOLD}${BLUE}  ╚══════╝╚═╝  ╚══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝${NC}"
  echo ""
  echo -e "  ${DIM}Environment Variable Auditor — Installer${NC}"
  echo ""
}

# ── Platform Detection ───────────────────────────────────────
detect_platform() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "$OS" in
    Linux*)  OS_NAME="linux" ;;
    Darwin*) OS_NAME="macos" ;;
    CYGWIN*|MINGW*|MSYS*) error "Windows is not supported by this script. Download the .exe from ${REPO_URL}/releases" ;;
    *) error "Unsupported OS: $OS" ;;
  esac

  case "$ARCH" in
    x86_64)        ARCH_NAME="x86_64" ;;
    aarch64|arm64) ARCH_NAME="aarch64" ;;
    *) ARCH_NAME="unknown" ;;
  esac

  info "Detected: $OS_NAME / $ARCH_NAME"
}

# ── Rust / Cargo ─────────────────────────────────────────────
has_cargo() { command -v cargo &>/dev/null; }

install_rust() {
  info "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --quiet
  # Source cargo env for rest of script
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
  success "Rust installed: $(rustc --version)"
}

ensure_rust() {
  if has_cargo; then
    success "Rust found: $(cargo --version)"
    return
  fi
  echo ""
  warn "Rust / Cargo not found."
  printf "  Install Rust now? [Y/n] "
  read -r answer </dev/tty
  case "$answer" in
    [Nn]*) error "Rust is required to build Enspect. Visit https://rustup.rs to install." ;;
    *) install_rust ;;
  esac
}

# ── Build ─────────────────────────────────────────────────────
build_from_source() {
  step "Building Enspect from source"
  info "Running: cargo build --release"
  echo ""
  cargo build --release
  echo ""
  success "Build complete → target/release/enspect"
}

# ── Install Binary ────────────────────────────────────────────
install_binary() {
  local src="$1"

  # Prefer cargo install — resolves to $CARGO_HOME/bin (rustup) or cargo's own bin dir
  if has_cargo; then
    step "Installing via cargo install"
    info "Running: cargo install --path ."
    cargo install --path . --quiet
    # Resolve the actual install dir from the freshly written binary
    INSTALL_DIR="$(cargo_bin_dir)"
    success "Installed to $INSTALL_DIR/$BINARY_NAME"
    return
  fi

  # Fallback: copy binary manually into a writable dir
  local target_dirs=("/usr/local/bin" "$HOME/.local/bin" "$HOME/bin")

  for dir in "${target_dirs[@]}"; do
    if [[ -d "$dir" && -w "$dir" ]]; then
      cp "$src" "$dir/$BINARY_NAME"
      chmod +x "$dir/$BINARY_NAME"
      success "Installed to $dir/$BINARY_NAME"
      INSTALL_DIR="$dir"
      return
    fi
  done

  # /usr/local/bin exists but needs sudo
  if [[ -d "/usr/local/bin" ]] && command -v sudo &>/dev/null; then
    info "Copying to /usr/local/bin (requires sudo)..."
    sudo cp "$src" "/usr/local/bin/$BINARY_NAME"
    sudo chmod +x "/usr/local/bin/$BINARY_NAME"
    success "Installed to /usr/local/bin/$BINARY_NAME"
    INSTALL_DIR="/usr/local/bin"
    return
  fi

  # Last resort: ~/.local/bin (create if missing)
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
  cp "$src" "$INSTALL_DIR/$BINARY_NAME"
  chmod +x "$INSTALL_DIR/$BINARY_NAME"
  success "Installed to $INSTALL_DIR/$BINARY_NAME"
}

# Return the directory where cargo installs binaries
cargo_bin_dir() {
  if [[ -n "${CARGO_HOME:-}" ]]; then
    echo "${CARGO_HOME}/bin"
  elif [[ -d "$HOME/.cargo/bin" ]]; then
    echo "$HOME/.cargo/bin"
  else
    # Homebrew cargo or system cargo — derive from the binary itself
    dirname "$(command -v cargo)"
  fi
}

# ── Auto-add dir to PATH (writes shell config + exports in-session) ───────────
add_to_path() {
  local dir="$1"

  # Export for the remainder of this script so verify_install can find the binary
  export PATH="$dir:$PATH"

  # Already persisted in shell config? Nothing to do
  local shell_name shell_config export_line
  shell_name="$(basename "$SHELL")"

  case "$shell_name" in
    zsh)  shell_config="$HOME/.zshrc" ;;
    bash) shell_config="$HOME/.bash_profile" ;;
    fish) shell_config="$HOME/.config/fish/config.fish" ;;
    *)    shell_config="" ;;
  esac

  if [[ "$shell_name" == "fish" ]]; then
    export_line="fish_add_path $dir"
  else
    export_line="export PATH=\"$dir:\$PATH\""
  fi

  # Check if the dir is already in the config file
  if [[ -n "$shell_config" ]] && grep -qF "$dir" "$shell_config" 2>/dev/null; then
    success "$dir already in $shell_config"
    return
  fi

  if [[ -n "$shell_config" ]]; then
    # Append with a comment so users know where it came from
    {
      echo ""
      echo "# Added by Enspect installer"
      echo "$export_line"
    } >> "$shell_config"
    success "Added $dir to PATH in $shell_config"
    info "Run \"source $shell_config\" or open a new terminal to use enspect globally."
  else
    warn "Unknown shell ($SHELL). Add manually to your shell config:"
    echo -e "  ${CYAN}$export_line${NC}"
  fi
}

# ── Shell Completions ─────────────────────────────────────────
setup_completions() {
  local shell_name
  shell_name="$(basename "$SHELL")"
  local enspect_bin

  # Use the installed binary
  if command -v enspect &>/dev/null; then
    enspect_bin="enspect"
  elif [[ -f "target/release/enspect" ]]; then
    enspect_bin="./target/release/enspect"
  else
    warn "Could not find enspect binary for completion generation. Skipping."
    return
  fi

  step "Setting up shell completions ($shell_name)"

  case "$shell_name" in
    bash)
      local comp_dir="$HOME/.bash_completion.d"
      mkdir -p "$comp_dir"
      "$enspect_bin" completion bash > "$comp_dir/enspect" 2>/dev/null || true
      success "Completions → $comp_dir/enspect"
      info "Add to ~/.bashrc:  source ~/.bash_completion.d/enspect"
      ;;
    zsh)
      # Try to find a valid fpath directory
      local comp_dir
      if [[ -n "${ZSH_CUSTOM:-}" ]]; then
        comp_dir="$ZSH_CUSTOM/completions"
      else
        comp_dir="$HOME/.zsh/completions"
      fi
      mkdir -p "$comp_dir"
      "$enspect_bin" completion zsh > "$comp_dir/_enspect" 2>/dev/null || true
      success "Completions → $comp_dir/_enspect"
      info "Ensure $comp_dir is in fpath (add to ~/.zshrc if needed):"
      echo -e "  ${DIM}fpath=($comp_dir \$fpath) && autoload -Uz compinit && compinit${NC}"
      ;;
    fish)
      local comp_dir="$HOME/.config/fish/completions"
      mkdir -p "$comp_dir"
      "$enspect_bin" completion fish > "$comp_dir/enspect.fish" 2>/dev/null || true
      success "Completions → $comp_dir/enspect.fish"
      ;;
    *)
      info "Run manually: enspect completion <bash|zsh|fish>"
      ;;
  esac
}

# ── Already Installed Check ───────────────────────────────────
check_existing() {
  if command -v enspect &>/dev/null; then
    local current
    current="$(enspect --version 2>&1 || echo 'unknown')"
    echo ""
    warn "Enspect is already installed ($current)."
    printf "  Reinstall / update? [y/N] "
    read -r answer </dev/tty
    case "$answer" in
      [Yy]*) return ;;
      *) echo ""; info "Nothing changed. Run 'enspect --help' to get started."; echo ""; exit 0 ;;
    esac
  fi
}

# ── Verify Install ────────────────────────────────────────────
verify_install() {
  local enspect_bin="${INSTALL_DIR:-}/$BINARY_NAME"

  # Prefer an explicit path so we don't rely on the current shell's hash table
  if [[ -x "$enspect_bin" ]] || command -v enspect &>/dev/null; then
    local ver
    ver="$("${enspect_bin}" --version 2>&1 || enspect --version 2>&1)"
    echo ""
    echo -e "  ${GREEN}${BOLD}Enspect installed successfully!${NC}  ${DIM}($ver)${NC}"
    echo ""
    echo -e "  ${BOLD}Quick start:${NC}"
    echo -e "  ${CYAN}  enspect${NC}              ${DIM}# Interactive REPL${NC}"
    echo -e "  ${CYAN}  enspect audit${NC}         ${DIM}# Full env var audit${NC}"
    echo -e "  ${CYAN}  enspect secrets${NC}       ${DIM}# Scan for leaked secrets${NC}"
    echo -e "  ${CYAN}  enspect --help${NC}        ${DIM}# All commands${NC}"
    echo ""
  else
    warn "Could not verify installation. Binary should be at: $enspect_bin"
    echo ""
  fi
}

# ── Main ──────────────────────────────────────────────────────
main() {
  print_banner
  detect_platform
  check_existing

  # Verify we're in the repo root
  if [[ ! -f "Cargo.toml" ]] || ! grep -q 'name = "enspect"' Cargo.toml 2>/dev/null; then
    error "Run this script from the Enspect repository root directory.\n\n  git clone ${REPO_URL}\n  cd Enspect && bash install.sh"
  fi

  # Ensure Rust is available
  ensure_rust

  # Build and install
  build_from_source
  install_binary "target/release/enspect"

  # Always ensure INSTALL_DIR is in PATH (writes to shell config + exports in-session)
  [[ -n "${INSTALL_DIR:-}" ]] && add_to_path "$INSTALL_DIR"

  # Optional: shell completions
  echo ""
  printf "  Set up shell tab completions? [Y/n] "
  read -r comp_answer </dev/tty
  case "$comp_answer" in
    [Nn]*) info "Skipping completions. Run 'enspect completion <shell>' later." ;;
    *) setup_completions ;;
  esac

  verify_install
}

main "$@"
