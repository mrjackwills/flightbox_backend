// Only allow when debugging
// #![allow(unused)]

use parse_env::AppEnv;

mod adsbdb_response;
mod app_error;
mod cron;
mod parse_env;
mod system_info;
mod ws;
mod ws_messages;

use cron::Cron;
use ws::open_connection;

fn setup_tracing(app_env: &AppEnv) {
    tracing_subscriber::fmt()
        .with_max_level(app_env.log_level)
        .init();
}

#[tokio::main]
async fn main() {
    let app_env = parse_env::AppEnv::get_env();
    setup_tracing(&app_env);
    tracing::info!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    Cron::init(&app_env);
    open_connection(app_env).await;
}
