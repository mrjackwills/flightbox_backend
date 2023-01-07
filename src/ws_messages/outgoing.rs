use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::{adsbdb_response::CombinedResponse, system_info::SysInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "message", content = "data")]
pub enum Response {
    Status(SysInfo),
    Flights(Vec<CombinedResponse>),
}

/// These get sent to the websocket server when in structured_data mode,
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct StructuredResponse {
    data: Option<Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Response>,
    unique: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache: Option<bool>,
}

impl StructuredResponse {
    /// Convert a ResponseMessage into a Tokio message of StructureResponse
    pub fn data(data: Response, cache: Option<bool>, unique: String) -> Message {
        let x = Self {
            data: Some(data),
            error: None,
            unique,
            cache,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default())
    }

    /// Convert a ErrorResponse into a Tokio message of StructureResponse
    pub fn _error(data: Response, unique: String) -> Message {
        let x = Self {
            error: Some(data),
            data: None,
            unique,
            cache: None,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default())
    }
}
