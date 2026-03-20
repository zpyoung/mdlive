use std::path::PathBuf;
use std::sync::OnceLock;

use mdlive::AppConfig;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::Manager;

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

fn switch_workspace(window: &tauri::WebviewWindow, path: &str) {
    let escaped = path.replace('\\', "\\\\").replace('\'', "\\'");
    let js = format!(
        "function __sw(){{fetch('/api/workspace/switch',\
         {{method:'POST',headers:{{'Content-Type':'application/json'}},\
         body:JSON.stringify({{path:'{escaped}'}})}}).then(function(){{window.location.href='/'}});}}\
         if(document.readyState==='complete')__sw();\
         else window.addEventListener('load',__sw);"
    );
    let _ = window.eval(&js);
}

fn home_prefix() -> String {
    dirs::home_dir()
        .map(|h| h.display().to_string())
        .unwrap_or_default()
}

fn shorten_path(path: &str) -> String {
    let home = home_prefix();
    if !home.is_empty() && path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    }
}

fn build_menu(app: &tauri::App) -> tauri::Result<()> {
    let config = AppConfig::load();

    let open_file = MenuItemBuilder::new("Open File...")
        .id("open_file")
        .accelerator("CmdOrCtrl+O")
        .build(app)?;

    let open_folder = MenuItemBuilder::new("Open Folder...")
        .id("open_folder")
        .accelerator("CmdOrCtrl+Shift+O")
        .build(app)?;

    // recent submenu -- files first, separator, then directories
    let mut recent_builder = SubmenuBuilder::new(app, "Open Recent");

    let files: Vec<_> = config.recent.iter().filter(|r| r.mode == "file").collect();
    let directories: Vec<_> = config
        .recent
        .iter()
        .filter(|r| r.mode == "directory")
        .collect();

    for entry in &files {
        let label = shorten_path(&entry.path);
        let item = MenuItemBuilder::new(&label)
            .id(format!("recent:{}", entry.path))
            .build(app)?;
        recent_builder = recent_builder.item(&item);
    }

    if !files.is_empty() && !directories.is_empty() {
        recent_builder = recent_builder.separator();
    }

    for entry in &directories {
        let label = shorten_path(&entry.path);
        let item = MenuItemBuilder::new(&label)
            .id(format!("recent:{}", entry.path))
            .build(app)?;
        recent_builder = recent_builder.item(&item);
    }

    if !config.recent.is_empty() {
        recent_builder = recent_builder.separator();
        let clear = MenuItemBuilder::new("Clear Recent")
            .id("clear_recent")
            .build(app)?;
        recent_builder = recent_builder.item(&clear);
    }

    let recent_menu = recent_builder.build()?;

    let app_menu = SubmenuBuilder::new(app, "mdlive")
        .about(None)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&open_file)
        .item(&open_folder)
        .separator()
        .item(&recent_menu)
        .separator()
        .item(&PredefinedMenuItem::close_window(app, None)?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View").fullscreen().build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&app_menu)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .build()?;

    app.set_menu(menu)?;
    Ok(())
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

            build_menu(app)?;

            app.on_menu_event(|app_handle, event| {
                let id = event.id().as_ref().to_string();

                if id == "open_file" {
                    let win = app_handle.get_webview_window("main");
                    std::thread::spawn(move || {
                        let file = rfd::FileDialog::new()
                            .add_filter("Supported", &["md", "markdown", "txt", "json"])
                            .pick_file();
                        if let Some(path) = file {
                            if let Some(ref window) = win {
                                switch_workspace(window, &path.display().to_string());
                            }
                        }
                    });
                } else if id == "open_folder" {
                    let win = app_handle.get_webview_window("main");
                    std::thread::spawn(move || {
                        let folder = rfd::FileDialog::new().pick_folder();
                        if let Some(path) = folder {
                            if let Some(ref window) = win {
                                switch_workspace(window, &path.display().to_string());
                            }
                        }
                    });
                } else if let Some(path) = id.strip_prefix("recent:") {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        switch_workspace(&window, path);
                    }
                } else if id == "clear_recent" {
                    let mut config = AppConfig::load();
                    config.recent.clear();
                    let _ = config.save();
                }
            });

            install_cli();

            // keep tokio runtime alive for the lifetime of the app
            std::mem::forget(rt);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Opened { urls } = event {
                for url in urls {
                    if url.scheme() == "file" {
                        if let Ok(path) = url.to_file_path() {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                switch_workspace(&window, &path.display().to_string());
                            }
                        }
                    }
                }
            }
        });
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
