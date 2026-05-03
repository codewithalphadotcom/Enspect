# ============================================================
#  Enspect — Windows Installer (PowerShell)
#  Usage: irm https://raw.githubusercontent.com/codewithalphadotcom/Enspect/main/get.ps1 | iex
# ============================================================
$ErrorActionPreference = "Stop"

$RepoUrl  = "https://github.com/codewithalphadotcom/Enspect.git"
$BinName  = "enspect.exe"
$CargoDir = "$env:USERPROFILE\.cargo\bin"

# ── Helpers ──────────────────────────────────────────────────
function Write-Banner {
  Write-Host ""
  Write-Host "  ███████╗███╗  ██╗███████╗██████╗ ███████╗ ██████╗████████╗" -ForegroundColor Blue
  Write-Host "  ██╔════╝████╗ ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝" -ForegroundColor Blue
  Write-Host "  █████╗  ██╔██╗██║███████╗██████╔╝█████╗  ██║        ██║   " -ForegroundColor Blue
  Write-Host "  ██╔══╝  ██║╚████║╚════██║██╔═══╝ ██╔══╝  ██║        ██║   " -ForegroundColor Blue
  Write-Host "  ███████╗██║ ╚███║███████║██║     ███████╗╚██████╗   ██║   " -ForegroundColor Blue
  Write-Host "  ╚══════╝╚═╝  ╚══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝   " -ForegroundColor Blue
  Write-Host ""
  Write-Host "  Environment Variable Auditor — Windows Installer" -ForegroundColor DarkGray
  Write-Host ""
}

function Write-Step { param($msg) Write-Host "`n  ▶ $msg" -ForegroundColor Cyan }
function Write-Info { param($msg) Write-Host "  • $msg" -ForegroundColor Blue }
function Write-OK   { param($msg) Write-Host "  ✓ $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "  ! $msg" -ForegroundColor Yellow }
function Write-Err  { param($msg) Write-Host "  ✗ $msg" -ForegroundColor Red; exit 1 }

# ── Check git ─────────────────────────────────────────────────
function Ensure-Git {
  Write-Step "Checking dependencies"
  if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Err "git is required but not installed.`n`n  Download from: https://git-scm.com/download/win"
  }
  Write-OK "git found: $(git --version)"
}

# ── Check / Install Rust ──────────────────────────────────────
function Ensure-Rust {
  Write-Step "Checking Rust / Cargo"

  # Refresh PATH so a freshly installed rustup is visible
  $env:PATH = "$CargoDir;$env:PATH"

  if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-OK "Rust found: $(cargo --version)"
    return
  }

  Write-Warn "Rust / Cargo not found."
  $answer = Read-Host "  Install Rust now? [Y/n]"
  if ($answer -match "^[Nn]") {
    Write-Err "Rust is required to build Enspect. Visit https://rustup.rs to install."
  }

  Write-Info "Downloading rustup-init.exe..."
  $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
  Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe -UseBasicParsing

  Write-Info "Running rustup installer (this may take a minute)..."
  & $rustupExe -y --quiet --no-modify-path
  Remove-Item $rustupExe -Force

  # Make cargo available in this session
  $env:PATH = "$CargoDir;$env:PATH"

  if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Err "Rust installation failed. Please install manually from https://rustup.rs"
  }
  Write-OK "Rust installed: $(cargo --version)"
}

# ── Clone Repo ────────────────────────────────────────────────
function Clone-Repo {
  Write-Step "Fetching Enspect"
  $script:TmpDir = Join-Path $env:TEMP "enspect-install-$(Get-Random)"
  Write-Info "Cloning into $($script:TmpDir)..."
  git clone --depth=1 $RepoUrl $script:TmpDir --quiet
  Write-OK "Cloned successfully"
}

# ── Build ─────────────────────────────────────────────────────
function Build-Enspect {
  Write-Step "Building Enspect from source"
  Write-Info "Running: cargo build --release"
  Write-Host ""
  Set-Location $script:TmpDir
  cargo build --release
  Write-Host ""
  Write-OK "Build complete"
}

# ── Install Binary ────────────────────────────────────────────
function Install-Binary {
  Write-Step "Installing Enspect"

  # Prefer cargo install to reuse rustup's managed bin dir
  if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Info "Running: cargo install --path ."
    cargo install --path . --quiet
    Write-OK "Installed via cargo to $CargoDir\$BinName"
  } else {
    # Fallback: copy binary manually
    if (-not (Test-Path $CargoDir)) { New-Item -ItemType Directory -Path $CargoDir | Out-Null }
    Copy-Item "$($script:TmpDir)\target\release\$BinName" "$CargoDir\$BinName" -Force
    Write-OK "Copied binary to $CargoDir\$BinName"
  }

  # Persist $CargoDir in the user PATH if not already there
  $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
  if ($userPath -notlike "*$CargoDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$CargoDir;$userPath", "User")
    Write-OK "Added $CargoDir to user PATH"
    Write-Info "Restart your terminal (or run: `$env:PATH = `"$CargoDir;`$env:PATH`") to use enspect."
  } else {
    Write-OK "$CargoDir already in PATH"
  }
}

# ── Check Existing ────────────────────────────────────────────
function Check-Existing {
  $env:PATH = "$CargoDir;$env:PATH"
  if (Get-Command enspect -ErrorAction SilentlyContinue) {
    $current = & enspect --version 2>&1
    Write-Host ""
    Write-Warn "Enspect is already installed ($current)."
    $answer = Read-Host "  Reinstall / update? [y/N]"
    if ($answer -notmatch "^[Yy]") {
      Write-Host ""
      Write-Info "Nothing changed. Run 'enspect --help' to get started."
      Write-Host ""
      exit 0
    }
  }
}

# ── Verify Install ────────────────────────────────────────────
function Verify-Install {
  $env:PATH = "$CargoDir;$env:PATH"
  $binPath = "$CargoDir\$BinName"
  if (Test-Path $binPath) {
    $ver = & $binPath --version 2>&1
    Write-Host ""
    Write-Host "  Enspect installed successfully!  ($ver)" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Quick start:" -ForegroundColor White
    Write-Host "    enspect         " -NoNewline -ForegroundColor Cyan
    Write-Host "# Interactive REPL" -ForegroundColor DarkGray
    Write-Host "    enspect audit   " -NoNewline -ForegroundColor Cyan
    Write-Host "# Full env var audit" -ForegroundColor DarkGray
    Write-Host "    enspect secrets " -NoNewline -ForegroundColor Cyan
    Write-Host "# Scan for leaked secrets" -ForegroundColor DarkGray
    Write-Host "    enspect --help  " -NoNewline -ForegroundColor Cyan
    Write-Host "# All commands" -ForegroundColor DarkGray
    Write-Host ""
  } else {
    Write-Warn "Could not verify installation. Binary expected at: $binPath"
  }
}

# ── Cleanup ───────────────────────────────────────────────────
function Cleanup {
  if ($script:TmpDir -and (Test-Path $script:TmpDir)) {
    Set-Location $env:USERPROFILE
    Remove-Item $script:TmpDir -Recurse -Force -ErrorAction SilentlyContinue
  }
}

# ── Main ──────────────────────────────────────────────────────
try {
  Write-Banner
  Check-Existing
  Ensure-Git
  Ensure-Rust
  Clone-Repo
  Build-Enspect
  Install-Binary
  Verify-Install
} finally {
  Cleanup
}
