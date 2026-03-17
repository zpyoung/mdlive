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

            // keep tokio runtime alive for the lifetime of the app
            std::mem::forget(rt);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
