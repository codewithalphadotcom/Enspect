# Enspect

<!-- [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)](https://www.rust-lang.org/) -->

> Stop shipping apps with missing or leaked environment variables.

A production-ready CLI tool for auditing environment variable usage across your entire codebase. Detects missing variables before a deployment crashes, flags secrets accidentally left in `.env` files, and highlights undocumented or unused variables that confuse new team members.

**Supports:** JavaScript, TypeScript, Python, Rust, Shell (Bash/Zsh)

---

## Features

- **Smart Detection** — Find missing, unused, and leaked env vars in your code
- **Deep Analysis** — Check individual variables across all sources
- **Secret Detection** — 18+ patterns for API keys, tokens, credentials with entropy-based fallback
- **Premium UI** — Interactive REPL with color-coded findings and premium styling
- **CI/CD Ready** — GitHub Actions integration, SARIF output, JSON export
- **Git Integration** — Track `.env` file changes and gitignore status
- **Multi-format Output** — Pretty terminal UI, JSON, SARIF 2.1.0, GitHub annotations
- **Pre-commit Hook** — Block commits with critical issues automatically

---

## Preview
<img width="1571" height="504" alt="Screenshot 2026-04-14 at 5 03 15 AM" src="https://github.com/user-attachments/assets/a260469b-0574-4f61-bca4-87cd02a0ea3e" />

<!-- <img width="1579" height="979" alt="Screenshot 2026-04-14 at 5 06 26 AM" src="https://github.com/user-attachments/assets/d342772e-1859-4527-beb3-18442088f5e6" /> -->
---

## Quick Start

### Installation

Build from source (requires [Rust](https://rustup.rs) 1.85+):

```bash
git clone https://github.com/codewithalpha/enspect.git
cd enspect
cargo build --release
./target/release/enspect --version
```

### Run your first audit

```bash
enspect audit              # Full environment variable audit
enspect                    # Interactive REPL
enspect scan               # List all env var references
enspect secrets            # Scan for leaked secrets
enspect check DATABASE_URL # Deep-check a specific variable
```

---

## All Commands

| Command | Purpose |
|---------|---------|
| `enspect audit` | Full environment variable audit |
| `enspect scan` | List all env var references in code |
| `enspect secrets` | Scan `.env*` files for leaked secrets |
| `enspect check <VAR>` | Deep-check a single variable |
| `enspect diff <file1> <file2>` | Compare two `.env` files |
| `enspect init` | Generate `.Enspect.toml` config |
| `enspect hook install` | Install pre-commit git hook |
| `enspect completion <shell>` | Generate shell completions (bash, zsh, fish) |

**Examples:**

```bash
# Audit with JSON output
enspect audit --format json > report.json

# SARIF output for GitHub Code Scanning
enspect audit --format sarif > results.sarif

# GitHub Actions annotation output
enspect audit --format github

# Only fail on secrets (ignore missing vars)
enspect audit --fail-on secret

# Skip git and secret checks for faster scan
enspect audit --no-git --no-secrets

# Audit a monorepo subdirectory
enspect audit --root ./apps/api
```

---

## What Enspect Detects

### Finding Categories

| Category | What triggers it | Severity |
|----------|-----------------|----------|
| **Missing** | Variable used in code but not in any `.env*` file | Warn |
| **Unused** | Variable defined in `.env*` but never referenced | Info |
| **Leaked** | Secret pattern detected (API keys, tokens, etc.) | Critical |
| **Empty** | Variable defined but has no value | Warn |
| **Undocumented** | Variable in `.env.local` but not in `.env.example` | Info |
| **Git Tracked** | `.env*` file committed to git | Warn |

### Secret Detection (18+ patterns)

Enspect detects credentials using:

1. **Pattern Matching** — AWS keys, GitHub tokens, Stripe, Google, SendGrid, MongoDB URIs, etc.
2. **Entropy Analysis** — Detects random high-entropy values (likely real keys)
3. **Smart Masking** — Shows only first 4 characters: `sk_l***`

**Examples of caught secrets:**
```
STRIPE_SECRET_KEY=sk_live_51Abc...  → CRITICAL
AWS_ACCESS_KEY=AKIA2ABC...          → CRITICAL
DATABASE_URL=postgres://u:p@...     → HIGH (entropy-based)
RANDOM_VALUE=aB1c2D3e4F5g6H7i8J9k0  → HIGH (32 chars, entropy 5.8)
```

**Placeholder exemption** — these are never flagged:
```
<your_api_key>, <replace-me>, your_secret_here, changeme, xxx, TODO, FIXME
```

## Language Support

**Code scanning for env var references:**
- **JavaScript/TypeScript** — `process.env.VAR`, `import.meta.env.VAR`, `Deno.env.get()`
- **Python** — `os.environ["VAR"]`, `os.getenv()`, `python-dotenv`
- **Rust** — `env!()`, `std::env::var()`, `dotenv` crate
- **Shell** — `$VAR`, `${VAR}` (Bash, Zsh, Sh)

**`.env` file support:**
- `.env`, `.env.local`, `.env.development`, `.env.production`, `.env.staging`, `.env.test`
- `.env.example`, `.env.sample`, `.env.template` (treated as documentation)

---

## Contributing

Enspect is open source and contributions are welcome from everyone. Whether you're interested in fixing bugs, adding features, improving documentation, or anything else, we'd love to have you contribute.

**How to get started:**
- **Report bugs or request features:** [GitHub Issues](https://github.com/codewithalpha/enspect/issues)
- **Security issues:** Please report security vulnerabilities privately to the maintainers
- **Submit pull requests:** Fork the repo, make your changes, and open a PR

All contributions are valued and appreciated.

---

## License

MIT License — see [LICENSE](LICENSE) file for full details.

This means you can use Enspect freely in personal, commercial, and open source projects with proper attribution.

---

## Acknowledgments

Built with passion for developers. Inspired by real-world environment variable mishaps and deployment chaos.

**Made by:** [alpha](https://github.com/codewithalphadotcom)

---

If you find Enspect useful, please consider starring or forking this repository to support the project.

Thank you for using Enspect!
