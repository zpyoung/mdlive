use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};

use crate::state::SharedMarkdownState;

pub(crate) async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedMarkdownState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: SharedMarkdownState) {
    let (mut sender, mut receiver) = socket.split();

    let (mut change_rx, initial_msg) = {
        let guard = state.lock().await;
        let rx = guard.change_tx.subscribe();
        let msg = if guard.daemon_mode && guard.has_workspace() {
            Some(crate::state::ServerMessage::WorkspaceChanged {
                base_dir: guard.base_dir.display().to_string(),
                file: None,
            })
        } else {
            None
        };
        (rx, msg)
    };

    // if a workspace is already loaded, notify this client immediately
    if let Some(ref msg) = initial_msg {
        eprintln!("[ws] client connected, workspace loaded -- sending WorkspaceChanged");
        if let Ok(json) = serde_json::to_string(msg) {
            let _ = sender.send(Message::Text(json)).await;
        }
    } else {
        eprintln!("[ws] client connected, no workspace loaded");
    }

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(_)) => {}
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Ok(reload_msg) = change_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&reload_msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }
}
