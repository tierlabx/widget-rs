# Widget-RS CLI (`widget-cli`)

<p align="center">
  <strong>English</strong> | <a href="README.md">简体中文</a>
</p>

`widget-cli` is the auxiliary build, management, and release automation CLI tool for `widget-rs`. As an independent command-line utility, it simplifies source-level plugin management, automates semantic version calculations and releases, maintains the `CHANGELOG.md`, and integrates seamlessly with CI/CD packaging workflows.

---

## Core Capabilities

1. **Automated Version Release (`release`)**: Analyzes Git commit history based on [Conventional Commits](https://www.conventionalcommits.org/), computes the next SemVer version, updates `Cargo.toml` and `Cargo.lock` across workspaces, generates `CHANGELOG.md`, and commits and pushes Git tags.
2. **Source-Level Plugin Management (`plugin`)**: Injects or removes plugin dependencies and source-level registry entries with a single command, enabling effortless plugin extension.
3. **CI/CD Pipeline Integration**: When a release tag is pushed, GitHub Actions automatically triggers to build Windows installers (NSIS / WiX MSI) and draft a GitHub Release.

---

## Command Line Usage Guide

### 1. Release Management (`release`)

```bash
# Calculate version from commit history since last tag and execute release
cargo run -p widget-cli -- release

# Preview next version and CHANGELOG content without making changes (dry-run mode)
cargo run -p widget-cli -- release --dry-run

# Manually specify a target version for release
cargo run -p widget-cli -- release --version 0.7.0
```

#### Version Bump Rules

`widget-cli` automatically determines the SemVer (`MAJOR.MINOR.PATCH`) upgrade level from commit message prefixes:

| Commit Prefix / Marker | Meaning | SemVer Bump | CHANGELOG Group |
| :--- | :--- | :--- | :--- |
| Contains `BREAKING CHANGE` or `!` (e.g. `feat!:`) | Breaking change | **MAJOR** (`+1.0.0`) | `Changed` |
| `feat:` / `feat(scope):` | New feature | **MINOR** (`0.+1.0`) | `Added` |
| `fix:` / `fix(scope):` | Bug fix | **PATCH** (`0.0.+1`) | `Fixed` |
| `perf:` / `perf(scope):` | Performance optimization | **PATCH** (`0.0.+1`) | `Fixed` |
| `refactor:` / `refactor(scope):` | Code refactoring | **PATCH** (`0.0.+1`) | `Changed` |
| `chore:` / `docs:` / `ci:` / `style:` / `test:` | Build, docs, CI, tests | **None** (no version bump) | `Skip` (not logged) |
| Non-conforming conventional commit | General commit | **PATCH** (`0.0.+1`) | `Other` |

> **Highest Priority Rule**: When a release cycle contains multiple commits, the highest priority rule takes effect (`MAJOR > MINOR > PATCH`). For instance, 1 `feat` and 2 `fix` commits will trigger a **MINOR** bump (e.g., `v0.6.1` -> `v0.7.0`).

#### Automated Release Steps

1. Scans all commits since the latest Git tag on the current branch and determines the new version number.
2. Batch-updates `version` fields in workspace root and all member crates/plugins `Cargo.toml`.
3. Runs `cargo check` to automatically sync `Cargo.lock`.
4. Prepends formatted changelog entries to the top of `CHANGELOG.md`.
5. Executes `git commit -m "chore: release vX.Y.Z"` with the changes.
6. Creates a local Git tag (e.g., `v0.7.0`).
7. Pushes the branch and tag to the remote repository via `git push origin <branch> --tags`.

---

### 2. Plugin Management (`plugin`)

`widget-cli` supports fast installation and uninstallation of native plugins directly at the source code level:

```bash
# Add a local plugin (injects dependency into crates/app/Cargo.toml and registers in plugin_registry.rs)
cargo run -p widget-cli -- plugin add <plugin_name> --path <local_path>
# Example:
cargo run -p widget-cli -- plugin add custom_widget --path ../plugins/custom_widget

# Remove an installed plugin
cargo run -p widget-cli -- plugin remove <plugin_name>
# Example:
cargo run -p widget-cli -- plugin remove custom_widget
```

---

## CI/CD Packaging & Deployment

When `widget-cli release` pushes a new `v*` tag, the GitHub Actions workflow (`.github/workflows/packager.yml`) is triggered automatically:

1. **Environment Setup**: Sets up the stable Rust toolchain, `cargo-packager`, and WiX Toolset on `windows-latest`.
2. **Packaging**: Executes `cargo packager --release -f nsis -f wix` to generate installers:
   - NSIS Setup executable: `target/release/*-setup.exe`
   - WiX MSI Installer: `target/release/*.msi`
3. **GitHub Release**: Automatically creates a GitHub Release, attaches installer assets, and publishes Release Notes.
