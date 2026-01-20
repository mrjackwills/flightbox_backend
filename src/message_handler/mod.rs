use async_channel::{Receiver, Sender};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{
    // alarm_schedule::AlarmSchedule,
    app_error::AppError,
    app_env::AppEnv,
    ws::{self, ConnectionDetails, Socket, WSSender, open_connection},
    ws_messages::Response,
};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WSReader =
    futures_util::stream::SplitStream<Box<WebSocketStream<MaybeTlsStream<TcpStream>>>>;
pub type WSWriter = futures_util::stream::SplitSink<
    Box<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tokio_tungstenite::tungstenite::Message,
>;

#[derive(Debug)]
pub enum Msg {
    Exit,
    Ping,
    Received(String),
    ToSend((Response, Option<bool>, Option<String>)),
    WsClose,
    WsConnected(Box<WsStream>),
}

#[derive(Debug)]
pub struct MessageHandler {
    app_env: AppEnv,
    connection_details: ConnectionDetails,
    rx: Receiver<Msg>,
    socket: Option<Socket>,
    tx: Sender<Msg>,
    ws_sender: WSSender,
}

impl MessageHandler {
    /// Send a status update, will be spawned in own thread before sending back to message handler here
    fn send_status(&self, unique: Option<String>) {
        let ws_sender = self.ws_sender.clone();
        tokio::spawn(async move {
            ws_sender.send_status(unique).await;
        });
    }

    /// Start the message handler
    pub async fn start(&mut self) -> Result<(), AppError> {
        open_connection(&self.app_env, &self.tx, &mut self.connection_details).await;

        while let Ok(msg) = self.rx.recv().await {
            match msg {
                Msg::Exit => {
                    if let Some(socket) = &mut self.socket {
                        socket.close().await;
                    }
                }
                Msg::Ping => {
                    if let Some(socket) = &mut self.socket {
                        socket.on_ping(&self.tx);
                    }
                }
                Msg::Received(msg) => {
                    let ws_sender = self.ws_sender.clone();
                    tokio::spawn(async move {
                        ws_sender.on_text(msg).await;
                    });
                }
                Msg::ToSend((response, cache, unique)) => {
                    if let Some(socket) = &mut self.socket {
                        socket.send(response, cache, unique).await;
                    }
                }
                Msg::WsClose => {
                    if let Some(socket) = &mut self.socket {
                        socket.close().await;
                    }
                    open_connection(&self.app_env, &self.tx, &mut self.connection_details).await;
                    self.ws_sender.on_connection();
                }
                Msg::WsConnected(stream) => {
                    self.socket = Some(Socket::new(stream, &self.tx));
                    self.send_status(None);
                }
            }
        }
        Ok(())
    }

    pub fn new(app_env: AppEnv, rx: Receiver<Msg>, tx: Sender<Msg>) -> Self {
        let ws_sender = ws::WSSender::new(&app_env, &tx);

        Self {
            app_env,
            connection_details: ConnectionDetails::new(),
            rx,
            socket: None,
            tx,
            ws_sender,
        }
    }
}
