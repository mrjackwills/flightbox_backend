use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("'{0}' - file not found'")]
    FileNotFound(String),
    #[error("missing env: '{0}'")]
    NotFound(String),
    #[error("Invalid request")]
    ReqwestError(#[from] reqwest::Error),
    #[error("thread error")]
    ThreadError(#[from] JoinError),
	 #[error("WS Connect - '{0}'")]
    TungsteniteConnect(String),
    #[error("Invalid WS Status Code")]
    WsStatus,
}
