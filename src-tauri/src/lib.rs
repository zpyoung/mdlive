use std::path::PathBuf;
use std::sync::OnceLock;

static SERVER_PORT: OnceLock<u16> = OnceLock::new();

async fn start_server() -> u16 {
    let router = mdlive::new_daemon_router();
    let (listener, port) = mdlive::bind_with_port_increment("127.0.0.1", 3000)
        .await
        .expect("failed to bind server");

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("server crashed");
    });

    port
}

#[tauri::command]
fn get_server_url() -> String {
    let port = SERVER_PORT.get().copied().unwrap_or(3000);
    format!("http://127.0.0.1:{port}")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let port = rt.block_on(start_server());
    SERVER_PORT.set(port).expect("port already set");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_server_url])
        .setup(move |app| {
            let url = format!("http://127.0.0.1:{port}");
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(url.parse().unwrap()),
            )
            .title("mdlive")
            .inner_size(1200.0, 800.0)
            .min_inner_size(600.0, 400.0)
            .build()?;

            let _ = window;

            install_cli();

            // keep tokio runtime alive for the lifetime of the app
            std::mem::forget(rt);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn install_cli() {
    let marker = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("com.beardedgiant.mdlive")
        .join(".cli-installed");

    if marker.exists() {
        return;
    }

    let exe_dir = match std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        Some(d) => d,
        None => return,
    };

    let src = exe_dir.join("mdlive-cli");
    if !src.exists() {
        return;
    }

    // find writable bin dir: /opt/homebrew/bin, /usr/local/bin
    let dest = ["/opt/homebrew/bin/mdlive", "/usr/local/bin/mdlive"]
        .iter()
        .find(|p| {
            std::path::Path::new(p)
                .parent()
                .map(|d| d.exists())
                .unwrap_or(false)
        })
        .unwrap_or(&"/usr/local/bin/mdlive");

    let cmd = format!("ln -sf '{}' '{}'", src.display(), dest);
    let script = format!(
        "do shell script \"{}\" with administrator privileges \
         with prompt \"mdlive wants to install the CLI command to {}\"",
        cmd, dest
    );

    let ok = std::process::Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        if let Some(parent) = marker.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&marker, "").ok();
    }
}
