use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Extension, Request},
    http::{header, StatusCode},
    middleware::{self, Next},
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
    let admin_token = std::env::var("ADMIN_TOKEN").expect("ADMIN_TOKEN environment variable must be set");

    let admin_routes = Router::new()
        .route("/status", get(admin_status))
        .route_layer(middleware::from_fn(auth_middleware));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest("/api/admin", admin_routes)
        .layer(Extension(admin_token));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await
}

async fn admin_status() -> &'static str {
    "Admin API OK"
}

async fn auth_middleware(
    Extension(admin_token): Extension<String>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok());

    match auth_header {
        Some(auth) if auth == format!("Bearer {}", admin_token) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
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
