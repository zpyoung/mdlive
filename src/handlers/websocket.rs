use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};

use crate::state::{ClientMessage, SharedMarkdownState};

pub(crate) async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedMarkdownState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: SharedMarkdownState) {
    let (mut sender, mut receiver) = socket.split();

    let mut change_rx = {
        let state = state.lock().await;
        state.change_tx.subscribe()
    };

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Ping | ClientMessage::RequestRefresh => {}
                        }
                    }
                }
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
