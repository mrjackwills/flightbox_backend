use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{app_error::AppError, parse_env::AppEnv};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Aircraft {
    #[serde(rename = "type")]
    pub aircraft_type: String,
    pub icao_type: String,
    pub manufacturer: String,
    pub mode_s: String,
    pub registration: String,
    pub registered_owner_country_iso_name: String,
    pub registered_owner_country_name: String,
    pub registered_owner_operator_flag_code: String,
    pub registered_owner: String,
    pub url_photo: Option<String>,
    pub url_photo_thumbnail: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Airport {
    pub country_iso_name: String,
    pub country_name: String,
    pub elevation: i32,
    pub iata_code: String,
    pub icao_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub municipality: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Flightroute {
    pub callsign: String,
    pub origin: Airport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midpoint: Option<Airport>,
    pub destination: Airport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Response<T> {
    response: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct AdsbdbResponse {
    aircraft: Aircraft,
    #[serde(skip_serializing_if = "Option::is_none")]
    flightroute: Option<Flightroute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombinedResponse {
    aircraft: Aircraft,
    #[serde(skip_serializing_if = "Option::is_none")]
    flightroute: Option<Flightroute>,
    callsign: Option<String>,
    altitude: u64,
}
#[derive(Debug, Deserialize, Serialize)]
struct Tar1090Aircraft {
    #[serde(rename(serialize = "mode_s"))]
    hex: String,
    #[serde(rename(serialize = "altitude"))]
    alt_baro: Option<u64>,
    #[serde(
        default,
        deserialize_with = "trim_flight",
        rename(serialize = "callsign"),
        skip_serializing_if = "Option::is_none"
    )]
    flight: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Tar1090Response {
    aircraft: Vec<Tar1090Aircraft>,
}

// test this, by passing in a Some("xxxx "), and then making sure is matches Some("xxxx")
fn trim_flight<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let callsign = String::deserialize(deserializer)?;
    Ok(Some(callsign.trim_end().to_owned()))
}

#[derive(Debug, Clone)]
pub struct Adsbdb {
    aircraft_url: String,
    adsbdb_url: String,
}

impl Adsbdb {
    pub fn new(app_env: &AppEnv) -> Self {
        Self {
            aircraft_url: app_env.url_tar0190.clone(),
            adsbdb_url: app_env.url_adsbdb.clone(),
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
            url.push_str(&format!("?callsign={callsign}"));
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
            // callsign seperate here
            callsign: aircraft.flight,
            altitude: aircraft.alt_baro.unwrap_or(0),
        })
    }

    pub async fn get_current_flights(&self) -> Result<Vec<CombinedResponse>, AppError> {
        let current_flights = self.aircraft_json().await?;
        let mut handles = vec![];
        for aircraft in current_flights.aircraft {
            handles.push(tokio::spawn(Self::adsbdb_data(
                aircraft,
                self.adsbdb_url.clone(),
            )));
        }
        let mut result = vec![];
        for request in handles {
            result.push(request.await?);
        }
        Ok(result.into_iter().flatten().collect::<Vec<_>>())
    }
}
