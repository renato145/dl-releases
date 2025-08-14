# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

This is `dl-releases`, a Rust CLI utility for downloading and installing
binaries from GitHub releases. It automates the process of checking for new
versions, downloading appropriate assets based on patterns, and
extracting/installing binaries to specified locations.

## Architecture

The codebase follows a modular structure:

- **`main.rs`**: CLI entry point using `clap` for argument parsing. Supports
  both config-driven and ad-hoc execution modes
- **`config.rs`**: Configuration management using TOML files, handles
  repository configurations and system paths (config, data, binaries locations)
- **`domain/`**: Core domain models
  - `repo.rs`: Repository parsing and validation (`user/repo_name` format)
  - `release.rs`: GitHub release and asset models with substring matching logic
- **`github_client.rs`**: HTTP client for GitHub API interactions, handles
- **`utils.rs`**: File extraction utilities supporting `.gz` and `.tar.gz`
  release fetching and asset downloading with progress tracking
  formats, version extraction from binaries

## Key Dependencies

- **HTTP/Async**: `reqwest`, `tokio` for GitHub API calls and file downloads
- **CLI/UX**: `clap` for argument parsing, `dialoguer` for user prompts,
  `indicatif` for progress bars
- **File handling**: `flate2`, `tar` for archive extraction
- **Configuration**: `config` crate with TOML support for persistent settings

## Common Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Code quality checks (linting and formatting)
just checks

# More strict clippy linting
just clippy-pedantic

# Security audit
just deny-check

# Run with arguments (single repository)
cargo run -- --repo user/repo_name --pat "linux-x86_64"

# Run with config file (processes all configured repos)
cargo run
```

## Configuration

The tool uses a TOML configuration file stored at
`~/.config/dl-releases/config.toml` with the following structure:

```toml
[[repos]]
repo = "user/repo_name"
pat = "asset_substring_pattern"
```

## Data Flow

1. **Input**: Either command-line args (`--repo`, `--pat`) or config file with
   multiple repositories
2. **Version Check**: Compares local binary version (via `--version` flag) with
   latest GitHub release
3. **Asset Selection**: Uses substring matching to find matching assets from
   release
4. **Download**: Streams asset with progress tracking to data directory
5. **Extraction**: Handles `.gz` and `.tar.gz` formats, extracts to binaries
   location
6. **Verification**: Confirms extracted binary version matches expected release
   version

## Testing

Tests use the `googletest` framework. Test files are located in
`src/test_files/` and `src/domain/test_files/` directories. Focus areas
include:

- Version extraction from binary output
- Asset pattern matching
- Archive extraction functionality
