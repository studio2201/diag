use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use common::WsMessage;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Router::new().route("/ws", get(ws_handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                match ws_msg {
                    WsMessage::RunPing(target) => {
                        let msg = WsMessage::Output(format!("Pinging {}...\n", target));
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                        
                        match Command::new("ping")
                            .arg("-c")
                            .arg("4")
                            .arg(&target)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                        {
                            Ok(mut child) => {
                                if let Some(stdout) = child.stdout.take() {
                                    let mut reader = BufReader::new(stdout).lines();
                                    while let Ok(Some(line)) = reader.next_line().await {
                                        let out = WsMessage::Output(format!("{}\n", line));
                                        if let Ok(json) = serde_json::to_string(&out) {
                                            if socket.send(Message::Text(json.into())).await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                }
                                let _ = child.wait().await;
                            }
                            Err(e) => {
                                let err = WsMessage::Error(format!("Failed to start ping: {}", e));
                                if let Ok(json) = serde_json::to_string(&err) {
                                    let _ = socket.send(Message::Text(json.into())).await;
                                }
                            }
                        }
                        
                        if let Ok(json) = serde_json::to_string(&WsMessage::Completed) {
                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
