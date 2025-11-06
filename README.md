# dl-releases

A CLI utility for downloading and installing binaries from GitHub releases.
Automates the process of checking for new versions, downloading appropriate
assets, and extracting binaries to your system.

> TODO: add warning about personal usage here

![Demo GIF](assets/demo.gif)
_Placeholder: Add a GIF showing the tool downloading and installing a binary (e.g., running `dl-releases --repo cli/cli --pat linux-amd64` and showing the progress bars)_

## ✨ Features

- 🔄 **Automatic Updates**: Checks current binary version against latest GitHub releases
- 📦 **Smart Asset Matching**: Uses pattern matching to find the right binary for your system
- ⚡ **Fast Downloads**: Concurrent downloads with progress tracking
- 🗂️ **Config-Driven**: Manage multiple repositories through a simple TOML config
- 📁 **Archive Support**: Handles `.gz` and `.tar.gz` compression formats
- 🔐 **Version Verification**: Ensures downloaded binary matches expected version

## 📥 Installation

### From Source

```bash
git clone https://github.com/your-username/dl-releases
cd dl-releases
cargo install --path .
```

### Pre-built Binaries

Download the latest binary from [releases](https://github.com/your-username/dl-releases/releases):

![Installation Screenshot](assets/installation.png)
_Placeholder: Add screenshot showing how to download and install the binary from GitHub releases page_

## 🚀 Quick Start

### One-time Download

Download and install a single binary:

```bash
# Download GitHub CLI for Linux
dl-releases --repo cli/cli --pat linux-amd64

# Download lazygit for macOS
dl-releases --repo jesseduffield/lazygit --pat Darwin_x86_64
```

### Config-Based Management

Create a config file to manage multiple repositories:

```bash
# Run without arguments to create initial config
dl-releases

# This creates ~/.config/dl-releases/config.toml
```

## 📖 Usage

### Command Line Arguments

```bash
dl-releases [OPTIONS]

Options:
  -r, --repo <REPO>                    Repository name in format user/repo_name
  -p, --pat <PATTERN>                  Pattern to match in asset names
  -o, --outpath <PATH>                 Download directory (default: ~/.local/share/dl-releases)
  -b, --binaries-location <PATH>       Install directory (default: ~/.local/bin)
  -h, --help                          Print help
  -V, --version                       Print version
```

### Examples

```bash
# Basic usage
dl-releases --repo cli/cli --pat linux-amd64

# Custom paths
dl-releases --repo neovim/neovim --pat linux64 --outpath ./downloads --binaries-location ./bin

# The tool will prompt to add successful downloads to your config
```

![Usage Example](assets/usage-example.png)
_Placeholder: Add screenshot showing the interactive prompt asking whether to add the repo to config_

## ⚙️ Configuration

The configuration file is stored at `~/.config/dl-releases/config.toml`:

```toml
[[repos]]
repo = "cli/cli"
pat = "linux-amd64"

[[repos]]
repo = "jesseduffield/lazygit"
pat = "Linux_x86_64"

[[repos]]
repo = "neovim/neovim"
pat = "linux64"

[[repos]]
repo = "BurntSushi/ripgrep"
pat = "x86_64-unknown-linux-gnu"
```

### Pattern Matching

The `pat` field uses substring matching to find the right asset:

| Repository              | Asset Name                           | Pattern        | Match |
| ----------------------- | ------------------------------------ | -------------- | ----- |
| `cli/cli`               | `gh_2.40.1_linux_amd64.tar.gz`       | `linux-amd64`  | ✅    |
| `jesseduffield/lazygit` | `lazygit_0.40.2_Linux_x86_64.tar.gz` | `Linux_x86_64` | ✅    |
| `neovim/neovim`         | `nvim-linux64.tar.gz`                | `linux64`      | ✅    |

### Directory Structure

```
~/.config/dl-releases/
└── config.toml              # Configuration file

~/.local/share/dl-releases/   # Download cache
├── gh_2.40.1_linux_amd64.tar.gz
└── lazygit_0.40.2_Linux_x86_64.tar.gz

~/.local/bin/                 # Installed binaries
├── gh
├── lazygit
└── nvim
```

## 🔄 Update Workflow

![Update Workflow](assets/workflow-diagram.png)
_Placeholder: Add flowchart showing: Check current version → Fetch latest release → Compare versions → Download if newer → Extract → Verify → Install_

1. **Version Check**: Runs `binary --version` to get current version
2. **Release Fetch**: Queries GitHub API for latest release
3. **Version Compare**: Uses semantic versioning comparison
4. **Asset Download**: Downloads matching asset with progress tracking
5. **Extraction**: Extracts binary from `.gz` or `.tar.gz` archives
6. **Verification**: Confirms extracted binary version matches release
7. **Installation**: Moves binary to target directory with executable permissions

## 📊 Example Output

```bash
$ dl-releases
[1/3] ✓ [cli] Downloaded to ~/.local/share/dl-releases (15.2 MB)
[2/3] ✓ [cli] Extracted to ~/.local/bin/gh
[3/3] ✓ [cli] Updated to version 2.40.1

[1/3] ✓ [lazygit] is up to date: 0.40.2
```

![Progress Bars](assets/progress-demo.gif)
_Placeholder: Add GIF showing the three-stage progress bars during download/extract/verify process_

## 🛠️ Development

### Prerequisites

- Rust 1.70+
- Git

### Building

```bash
git clone https://github.com/your-username/dl-releases
cd dl-releases
cargo build --release
```

### Testing

```bash
cargo test
```

### Code Quality

```bash
# Run all checks
just checks

# Strict linting
just clippy-pedantic

# Security audit
just deny-check
```

## 🗂️ Supported Archive Formats

- ✅ `.gz` - Single gzipped files
- ✅ `.tar.gz` - Gzipped tar archives
- ❌ `.zip` - Not yet supported
- ❌ `.7z` - Not yet supported

## ⚠️ Limitations

- Only works with public GitHub repositories
- Requires binaries to support `--version` flag for version detection
- Pattern matching is case-sensitive substring matching
- No support for GitHub authentication (rate limits may apply)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [GitHub API](https://docs.github.com/en/rest) for release data
- [clap](https://github.com/clap-rs/clap) for CLI argument parsing
- [tokio](https://github.com/tokio-rs/tokio) for async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client

---

<p align="center">
  <sub>Built with ❤️ and Rust</sub>
</p>
