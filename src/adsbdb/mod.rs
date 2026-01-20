use std::fmt::Write;

use reqwest::Client;
mod response;

use crate::{
    C,
    adsbdb::response::{AdsbdbResponse, Response, Tar1090Aircraft, Tar1090Response},
    app_env::AppEnv,
    app_error::AppError,
};

pub use response::CombinedResponse;

#[derive(Debug, Clone)]
pub struct Adsbdb {
    aircraft_url: String,
    adsbdb_url: String,
}

impl Adsbdb {
    pub fn new(app_env: &AppEnv) -> Self {
        Self {
            aircraft_url: C!(app_env.url_tar0190),
            adsbdb_url: C!(app_env.url_adsbdb),
        }
    }

    fn get_client() -> Result<Client, AppError> {
        Ok(reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_millis(5000))
            .gzip(true)
            .brotli(true)
            .user_agent(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .build()?)
    }

    async fn aircraft_json(&self) -> Result<Tar1090Response, AppError> {
        Ok(Self::get_client()?
            .get(&self.aircraft_url)
            .send()
            .await?
            .json::<Tar1090Response>()
            .await?)
    }

    async fn adsbdb_data(
        aircraft: Tar1090Aircraft,
        adsbdb_url: String,
    ) -> Result<CombinedResponse, AppError> {
        let mut url = format!("{adsbdb_url}/aircraft/{}", aircraft.hex);

        // if callsign add callsign to url
        if let Some(callsign) = aircraft.flight.as_ref() {
            write!(&mut url, "?callsign={callsign}").ok();
        }

        let response = Self::get_client()?
            .get(&url)
            .send()
            .await?
            .json::<Response<AdsbdbResponse>>()
            .await?
            .response;

        Ok(CombinedResponse {
            aircraft: response.aircraft,
            flightroute: response.flightroute,
            // callsign separate here
            callsign: aircraft.flight,
            altitude: aircraft.alt_baro.unwrap_or_default(),
        })
    }

    pub async fn get_current_flights(&self) -> Result<Vec<CombinedResponse>, AppError> {
        let current_flights = self.aircraft_json().await?;
        let mut handles = vec![];
        for aircraft in current_flights.aircraft {
            handles.push(tokio::spawn(Self::adsbdb_data(
                aircraft,
                C!(self.adsbdb_url),
            )));
        }
        let mut result = vec![];
        for request in handles {
            result.push(request.await?);
        }
        Ok(result.into_iter().flatten().collect::<Vec<_>>())
    }
}
