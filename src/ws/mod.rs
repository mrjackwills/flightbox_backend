mod connect;
mod connection_details;

use connect::ws_upgrade;
use connection_details::ConnectionDetails;
use futures_util::{
    lock::Mutex,
    stream::{SplitSink, SplitStream},
    StreamExt, TryStreamExt,
};
use std::sync::Arc;
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{self, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

use crate::parse_env::AppEnv;

use crate::ws::ws_sender::WSSender;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WSReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WSWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

mod ws_sender;

#[derive(Debug, Default)]
struct AutoClose(Option<JoinHandle<()>>);

/// Will close the connection after 40 seconds unless a ping message is received
impl AutoClose {
    fn on_ping(&mut self, ws_sender: &WSSender) {
        if let Some(handle) = self.0.as_ref() {
            handle.abort();
        };
        let ws_sender = ws_sender.clone();
        self.0 = Some(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(40)).await;
            ws_sender.close().await;
        }));
    }
}

/// handle each incoming ws message
async fn incoming_ws_message(
    mut reader: WSReader,
    ws_sender: WSSender,
    mut auto_close: AutoClose,
) {
    while let Ok(Some(message)) = reader.try_next().await {
        match message {
            Message::Text(message) => {
                let ws_sender = ws_sender.clone();
                tokio::spawn(async move {
                    ws_sender.on_text(message).await;
                });
            }
            Message::Ping(_) => auto_close.on_ping(&ws_sender),
            Message::Close(_) => {
                ws_sender.close().await;
                break;
            }
            _ => (),
        };
    }
}

pub async fn open_connection(app_env: AppEnv) {
    let mut connection_details = ConnectionDetails::new();
    loop {
        info!("in connection loop, awaiting delay then try to connect");
        connection_details.reconnect_delay().await;

        match ws_upgrade(&app_env).await {
            Ok(socket) => {
                info!("connected in ws_upgrade match");
                connection_details.valid_connect();

                let (writer, reader) = socket.split();

                let ws_sender = WSSender::new(
                    &app_env,
                    connection_details.get_connect_instant(),
                    Arc::new(Mutex::new(writer)),
                );
                let mut auto_close = AutoClose::default();
                auto_close.on_ping(&ws_sender);
                incoming_ws_message(reader, ws_sender, auto_close).await;

                info!("open_connection completed, will try to reconnect");
            }
            Err(e) => {
                error!("open_connection::{e:?}");
                connection_details.fail_connect();
            }
        }
    }
}
