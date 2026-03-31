use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use mdlive::{scan_supported_files, serve_daemon, serve_markdown};

#[derive(Parser)]
#[command(name = "mdlive")]
#[command(about = "Markdown workspace server for AI coding agents")]
#[command(version)]
struct Cli {
    /// Path to markdown file or directory to serve (omit for daemon mode)
    path: Option<PathBuf>,

    /// Hostname (domain or IP address) to listen on
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    hostname: String,

    /// Port to serve on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Don't open the preview in the default browser
    #[arg(long)]
    no_open: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Manage the mdlive LaunchAgent service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand)]
enum ServiceAction {
    /// Install LaunchAgent for auto-start on login
    Install,
    /// Remove LaunchAgent
    Uninstall,
    /// Start the daemon
    Start,
    /// Stop the running daemon
    Stop,
    /// Check if the service is running
    Status,
}

const LAUNCHD_LABEL: &str = "com.beardedgiant.mdlive";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Command::Service { action }) = cli.command {
        return handle_service(action, &cli.hostname, cli.port);
    }

    match cli.path {
        Some(path) => {
            let absolute_path = path.canonicalize().unwrap_or(path);

            // try handing off to a running server (daemon or Tauri app)
            if try_handoff(&absolute_path, cli.port) {
                return Ok(());
            }

            let (base_dir, tracked_files, is_directory_mode) = if absolute_path.is_file() {
                let base_dir = absolute_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .to_path_buf();
                let tracked_files = vec![absolute_path];
                (base_dir, tracked_files, false)
            } else if absolute_path.is_dir() {
                let tracked_files = scan_supported_files(&absolute_path)?;
                if tracked_files.is_empty() {
                    anyhow::bail!("No supported files found in directory (.md, .txt, .json)");
                }
                (absolute_path, tracked_files, true)
            } else {
                anyhow::bail!("Path must be a file or directory");
            };

            serve_markdown(
                base_dir,
                tracked_files,
                is_directory_mode,
                cli.hostname,
                cli.port,
                !cli.no_open,
            )
            .await?;
        }
        None => {
            serve_daemon(cli.hostname, cli.port, !cli.no_open).await?;
        }
    }

    Ok(())
}

fn try_handoff(path: &std::path::Path, port: u16) -> bool {
    use std::io::{Read as _, Write as _};

    let path_str = path.display().to_string();
    let body = format!(
        "{{\"path\":\"{}\"}}",
        path_str.replace('\\', "\\\\").replace('"', "\\\"")
    );
    let request = format!(
        "POST /api/workspace/switch HTTP/1.1\r\n\
         Host: 127.0.0.1:{port}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );

    let addr = format!("127.0.0.1:{port}");
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        std::time::Duration::from_millis(500),
    ) else {
        return false;
    };

    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let _ = stream.flush();

    let mut buf = vec![0u8; 1024];
    let _ = stream.read(&mut buf);
    let response = String::from_utf8_lossy(&buf);

    if !response.contains("\"success\":true") {
        return false;
    }

    eprintln!("Switched running mdlive instance to: {path_str}");

    // bring Tauri app to focus if installed
    let _ = std::process::Command::new("open")
        .args(["-a", "mdlive"])
        .status();

    true
}

fn handle_service(action: ServiceAction, hostname: &str, port: u16) -> Result<()> {
    let plist_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?
        .join("Library/LaunchAgents");
    let plist_path = plist_dir.join(format!("{LAUNCHD_LABEL}.plist"));

    match action {
        ServiceAction::Install => {
            let exe = std::env::current_exe()?;
            std::fs::create_dir_all(&plist_dir)?;

            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LAUNCHD_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>-H</string>
        <string>{hostname}</string>
        <string>-p</string>
        <string>{port}</string>
        <string>--no-open</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/mdlive.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/mdlive.err</string>
</dict>
</plist>"#,
                exe = exe.display()
            );

            std::fs::write(&plist_path, plist)?;

            let status = std::process::Command::new("launchctl")
                .args(["load", "-w"])
                .arg(&plist_path)
                .status()?;

            if status.success() {
                println!("Installed and started {LAUNCHD_LABEL}");
                println!("  Plist: {}", plist_path.display());
                println!("  http://{hostname}:{port}");
            } else {
                anyhow::bail!("launchctl load failed");
            }
        }
        ServiceAction::Start => {
            if !plist_path.exists() {
                anyhow::bail!("LaunchAgent not installed. Run `mdlive service install` first.");
            }
            let status = std::process::Command::new("launchctl")
                .args(["start", LAUNCHD_LABEL])
                .status()?;
            if status.success() {
                println!("Started {LAUNCHD_LABEL}");
            } else {
                anyhow::bail!("launchctl start failed");
            }
        }
        ServiceAction::Stop => {
            let status = std::process::Command::new("launchctl")
                .args(["stop", LAUNCHD_LABEL])
                .status()?;
            if status.success() {
                println!("Stopped {LAUNCHD_LABEL}");
            } else {
                println!("{LAUNCHD_LABEL}: not running or not installed");
            }
        }
        ServiceAction::Uninstall => {
            if plist_path.exists() {
                let _ = std::process::Command::new("launchctl")
                    .args(["unload"])
                    .arg(&plist_path)
                    .status();
                std::fs::remove_file(&plist_path)?;
                println!("Uninstalled {LAUNCHD_LABEL}");
            } else {
                println!("No plist found at {}", plist_path.display());
            }
        }
        ServiceAction::Status => {
            let output = std::process::Command::new("launchctl")
                .args(["list", LAUNCHD_LABEL])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("{LAUNCHD_LABEL}: running");
                for line in stdout.lines() {
                    if line.contains("PID") || line.contains("pid") {
                        println!("  {}", line.trim());
                    }
                }
            } else {
                println!("{LAUNCHD_LABEL}: not running");
            }

            if plist_path.exists() {
                println!("  Plist: {}", plist_path.display());
            } else {
                println!("  Plist: not installed");
            }
        }
    }

    Ok(())
}
