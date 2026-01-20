use async_channel::Sender;
use std::{process, time::Instant};

use crate::C;
use crate::adsbdb::Adsbdb;
use crate::message_handler::Msg;
use crate::app_env::AppEnv;
use crate::system_info::SysInfo;
use crate::ws_messages::to_struct;
use crate::ws_messages::{MessageValues, ParsedMessage, Response};

#[derive(Debug, Clone)]
pub struct WSSender {
    app_envs: AppEnv,
    adsbdb: Adsbdb,
    connected_instant: Instant,
    tx: Sender<Msg>,
}

impl WSSender {
    pub fn new(app_envs: &AppEnv, tx: &Sender<Msg>) -> Self {
        Self {
            adsbdb: Adsbdb::new(app_envs),
            app_envs: C!(app_envs),
            connected_instant: std::time::Instant::now(),
            tx: C!(tx),
        }
    }

    /// Update the connected_instance time
    pub fn on_connection(&mut self) {
        self.connected_instant = std::time::Instant::now();
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&self, message: String) {
        if let Some(data) = to_struct(&message) {
            match data {
                MessageValues::Invalid(error) => tracing::error!("{error:?}"),
                MessageValues::Valid(message, unique) => match message {
                    ParsedMessage::Status => self.send_status(Some(unique)).await,
                    ParsedMessage::Flights => match self.adsbdb.get_current_flights().await {
                        Ok(data) => {
                            self.send_ws_response(Response::Flights(data), None, Some(unique))
                                .await;
                        }
                        Err(e) => {
                            tracing::error!("get_current_flights::{e:?}");
                        }
                    },
                },
            }
        }
    }

    /// Send a message to the socket
    /// cache could just be Option<()>, and if some then send true?
    async fn send_ws_response(
        &self,
        response: Response,
        cache: Option<bool>,
        unique: Option<String>,
    ) {
        match self.tx.send(Msg::ToSend((response, cache, unique))).await {
            Ok(()) => (),
            Err(e) => {
                tracing::error!("{e}");
                process::exit(1);
            }
        }
    }

    /// Send status of flightbox backend machine to client
    pub async fn send_status(&self, unique: Option<String>) {
        let info = SysInfo::new(&self.app_envs, &self.connected_instant).await;
        let response = Response::Status(info);
        self.send_ws_response(response, Some(true), unique).await;
    }
}
