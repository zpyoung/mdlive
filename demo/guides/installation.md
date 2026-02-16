# Installation

## System Requirements

| OS | Min Version | Architecture |
|----|-------------|--------------|
| macOS | 13 (Ventura) | arm64, x86_64 |
| Ubuntu | 22.04 LTS | x86_64 |
| Debian | 12 | x86_64 |
| RHEL | 9 | x86_64 |

Additional requirements:

- Docker 24+ (for local builds)
- Git 2.40+
- 4 GB free disk space

## Install via Script

```bash
curl -sSL https://acme.internal/install | sh
```

The script detects your OS and architecture, downloads the correct binary, and places it in `/usr/local/bin`.

## Install via Homebrew

```bash
brew tap acme/tap
brew install acme-cli
```

## Install from Source

```bash
git clone https://gitlab.internal/acme/cli.git
cd cli
cargo build --release
cp target/release/acme /usr/local/bin/
```

## Shell Completions

```bash
# bash
acme completions bash > /etc/bash_completion.d/acme

# zsh
acme completions zsh > ~/.zfunc/_acme

# fish
acme completions fish > ~/.config/fish/completions/acme.fish
```

## Verify Installation

```bash
$ acme version
acme-cli 2.1.0 (darwin/arm64)

$ acme doctor
Checking Docker...       OK (27.0.1)
Checking kubectl...      OK (1.29.2)
Checking network...      OK (registry reachable)
Checking credentials...  OK (token valid until 2026-03-15)
```

## Uninstall

```bash
rm /usr/local/bin/acme
rm -rf ~/.acme
```
