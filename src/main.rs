use async_channel::Sender;

mod adsbdb;
mod app_error;
mod cron;
mod macros;
mod message_handler;
mod app_env;
mod system_info;
mod ws_messages;
mod ws;

use cron::Cron;
use tokio::signal;

use crate::{
    app_env::AppEnv, app_error::AppError, message_handler::{MessageHandler, Msg}
};

fn setup_tracing(app_env: &AppEnv) {
    tracing_subscriber::fmt()
        .with_max_level(app_env.log_level)
        .init();
}

#[allow(clippy::expect_used)]
fn shutdown_signal(sender: &Sender<Msg>) {
    let sender = C!(sender);
    tokio::spawn(async move {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }
        sender.send(Msg::Exit).await.ok();
        sleep!(250);
        println!("closed");
        std::process::exit(1);
    });
}

async fn start() -> Result<(), AppError> {
    let app_env = app_env::AppEnv::get_env();
    setup_tracing(&app_env);
    tracing::info!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    Cron::init(&app_env);
    let (tx, rx) = async_channel::bounded(2048);
    shutdown_signal(&tx);
    MessageHandler::new(app_env, rx, tx).start().await
}

#[tokio::main]

async fn main() -> Result<(), AppError> {
    if let Err(e) = tokio::spawn(start()).await {
        tracing::error!("{e}");
    }
    Ok(())
}
