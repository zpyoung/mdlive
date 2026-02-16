# Building a Plugin

Extend the Acme CLI with custom commands via the plugin system.

## Plugin Structure

A plugin is a standalone binary that follows the `acme-<name>` naming convention. When you run `acme foo`, the CLI looks for `acme-foo` in your PATH.

```
acme-hello/
  src/
    main.rs
  Cargo.toml
```

## Minimal Example

Create a simple plugin that greets the user:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "acme-hello", about = "Say hello")]
struct Cli {
    /// Name to greet
    #[arg(short, long, default_value = "world")]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u32,
}

fn main() {
    let cli = Cli::parse();
    for _ in 0..cli.count {
        println!("Hello, {}!", cli.name);
    }
}
```

Build and install:

```bash
cargo build --release
cp target/release/acme-hello /usr/local/bin/
```

Now it works as a subcommand:

```bash
$ acme hello --name "Acme Team" --count 3
Hello, Acme Team!
Hello, Acme Team!
Hello, Acme Team!
```

## Accessing the Platform API

Plugins can read the user's credentials from `~/.acme/credentials`:

```rust
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct Credentials {
    access_token: String,
    expires_at: String,
}

fn load_credentials() -> Result<Credentials, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let path = format!("{home}/.acme/credentials");
    let content = fs::read_to_string(path)?;
    let creds: Credentials = serde_json::from_str(&content)?;
    Ok(creds)
}
```

Then use the token to call the API:

```rust
let creds = load_credentials()?;
let client = reqwest::blocking::Client::new();
let response = client
    .get("https://api.acme.internal/v1/services")
    .bearer_auth(&creds.access_token)
    .send()?;
```

## Plugin Manifest

For discoverability, add an `acme-plugin.json` manifest:

```json
{
  "name": "hello",
  "version": "1.0.0",
  "description": "A greeting plugin for Acme CLI",
  "author": "platform-team@acme.co",
  "min_cli_version": "2.0.0"
}
```

## Publishing

Share plugins via the internal registry:

```bash
acme plugin publish ./acme-hello
```

Others install with:

```bash
acme plugin install hello
```
