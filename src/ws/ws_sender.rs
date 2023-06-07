use futures_util::lock::Mutex;
use futures_util::SinkExt;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, trace};

use crate::adsbdb_response::Adsbdb;
use crate::system_info::SysInfo;
use crate::ws_messages::{MessageValues, ParsedMessage, Response, StructuredResponse};
use crate::{parse_env::AppEnv, ws_messages::to_struct};

use super::WSWriter;

#[derive(Debug, Clone)]
pub struct WSSender {
    adsbdb: Adsbdb,
    app_env: AppEnv,
    connected_instant: Instant,
    writer: Arc<Mutex<WSWriter>>,
}

impl WSSender {
    pub fn new(app_env: &AppEnv, connected_instant: Instant, writer: Arc<Mutex<WSWriter>>) -> Self {
        let adsbdb = Adsbdb::new(app_env);
        Self {
            adsbdb,
            app_env: app_env.clone(),
            connected_instant,
            writer,
        }
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&mut self, message: String) {
        if let Some(data) = to_struct(&message) {
            match data {
                MessageValues::Invalid(error) => error!("{error:?}"),
                MessageValues::Valid(message, unique) => match message {
                    ParsedMessage::Status => self.send_status(unique).await,
                    ParsedMessage::Flights => match self.adsbdb.get_current_flights().await {
                        Ok(data) => {
                            self.send_ws_response(Response::Flights(data), None, unique)
                                .await;
                        }
                        Err(e) => {
                            error!("get_current_flights::{e:?}");
                        }
                    },
                    ParsedMessage::On => SysInfo::toggle_screen(&self.app_env, true),
                    ParsedMessage::Off => SysInfo::toggle_screen(&self.app_env, false),
                },
            }
        }
    }

    /// Restart by force quitting, and assuming running in an auto-restart container or systemd
    // async fn restart(&mut self) {
    //     self.close().await;
    //     process::exit(0);
    // }

    /// Send a message to the socket
    async fn send_ws_response(&mut self, response: Response, cache: Option<bool>, unique: String) {
        match self
            .writer
            .lock()
            .await
            .send(StructuredResponse::data(response, cache, unique))
            .await
        {
            Ok(_) => trace!("Message sent"),
            Err(e) => {
                error!("send_ws_response::SEND-ERROR::{e:?}");
                self.writer.lock().await.close().await.ok();
            }
        }
    }

    /// Send status of flightbox backend machine to client
    pub async fn send_status(&mut self, unique: String) {
        let info = SysInfo::new(&self.app_env, &self.connected_instant).await;
        let response = Response::Status(info);
        self.send_ws_response(response, Some(true), unique).await;
    }

    /// close connection, uses a 2 second timeout
    pub async fn close(&mut self) {
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            self.writer.lock().await.close(),
        )
        .await
        .ok();
    }
}
