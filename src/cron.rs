use std::time::{Duration, Instant};

use tracing::{debug, error};

use crate::{
    adsbdb_response::{self, Adsbdb},
    parse_env::AppEnv,
};

pub struct Cron {
    adsbdb: Adsbdb,
    sleep_duration: Duration,
}

const ONE_MINUTE_IN_SEC: u64 = 60;

impl Cron {
    /// Create a basic cron job, spawn into own tokio thread
    pub async fn init(app_env: &AppEnv) {
        let adsbdb = adsbdb_response::Adsbdb::new(app_env);
        let mut inner = Self {
            adsbdb,
            sleep_duration: std::time::Duration::from_secs(ONE_MINUTE_IN_SEC * 5),
        };
        tokio::spawn(async move {
            inner.croner().await;
        });
    }

    /// Ignore, other than log, any errors from get_current_flights
    async fn execute(&self) {
        match self.adsbdb.get_current_flights().await {
            Ok(_) => debug!("cron executed correctly"),
            Err(e) => {
                error!("croner::{e:?}");
            }
        }
    }

    /// Get current flights every 5 minutes, just to build up the adsbdb database and cache
    async fn croner(&mut self) {
        loop {
            let now = Instant::now();
            self.execute().await;
            let to_sleep = std::time::Duration::from_millis(
                u64::try_from(
                    self.sleep_duration
                        .as_millis()
                        .saturating_sub(now.elapsed().as_millis()),
                )
                .unwrap_or_default(),
            );
            tokio::time::sleep(to_sleep).await;
        }
    }
}
