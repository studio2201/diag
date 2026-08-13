use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WsMessage {
    RunPing(String),
    Output(String),
    Error(String),
    Completed,
}
