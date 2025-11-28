# Installation

Git-Iris requires Git 2.23.0+ and an API key for your chosen LLM provider.

## Prerequisites

| Requirement              | Version       | Notes                                   |
| ------------------------ | ------------- | --------------------------------------- |
| **Git**                  | 2.23.0+       | Check with `git --version`              |
| **Rust** (Cargo install) | Latest stable | Get from [rustup.rs](https://rustup.rs) |
| **LLM API Key**          | —             | OpenAI, Anthropic, or Google            |

## Installation Methods

### Via Cargo (Recommended)

```bash
cargo install git-iris
```

This installs the latest stable release from [crates.io](https://crates.io/crates/git-iris). The binary will be available system-wide as `git-iris`.

**Verify installation:**

```bash
git-iris --version
```

### Via Docker

Pull the official image:

```bash
docker pull hyperb1iss/git-iris:latest
```

**Run in your repository:**

```bash
docker run --rm -v "$(pwd):/git-repo" hyperb1iss/git-iris gen
```

**With environment variables:**

```bash
docker run --rm -v "$(pwd):/git-repo" \
  -e GITIRIS_PROVIDER="openai" \
  -e GITIRIS_API_KEY="your-api-key" \
  hyperb1iss/git-iris gen
```

**Persistent configuration:**

```bash
docker run --rm -v "$(pwd):/git-repo" \
  -v git-iris-config:/root/.config/git-iris \
  hyperb1iss/git-iris config --provider openai --api-key your-api-key
```

Docker is excellent for CI/CD workflows—no installation required on your build agents.

### Manual Build

Clone and build from source:

```bash
git clone https://github.com/hyperb1iss/git-iris.git
cd git-iris
cargo build --release
cargo install --path .
```

The release binary will be in `target/release/git-iris` and installed to your Cargo bin directory.

## Platform-Specific Notes

### macOS

If using Homebrew's Git, ensure it's up to date:

```bash
brew upgrade git
```

### Linux

Most distributions ship recent Git versions. If yours is outdated:

```bash
# Debian/Ubuntu
sudo add-apt-repository ppa:git-core/ppa
sudo apt update && sudo apt install git

# Fedora/RHEL
sudo dnf install git
```

### Windows

Git-Iris works in WSL2, Git Bash, or PowerShell. We recommend WSL2 for the best experience, especially with Studio's TUI features.

## Verify Installation

Check that Git-Iris is properly installed:

```bash
git-iris --version
```

You should see version information and the build metadata.

## What's Next?

Head to [Configuration](./configuration.md) to set up your API key, or jump straight to the [Quick Start](./quick-start.md) to generate your first AI commit.
