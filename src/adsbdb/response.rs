use serde::{Deserialize, Deserializer, Serialize, de::IntoDeserializer};

use crate::S;

/// Parse an i64, custom error if failure, if contains "ground", return 0
fn parse_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    match i64::deserialize(deserializer) {
        Ok(alt) => Ok(alt),
        Err(e) => {
            if e.to_string().contains("ground") {
                Ok(0)
            } else {
                Err(e)
            }
        }
    }
}

pub fn parse_op_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<i64>::deserialize(deserializer)? {
        Some(x) => Ok(Some(parse_i64(x.into_deserializer())?)),
        _ => Ok(None),
    }
}

// test this, by passing in a Some("xxxx "), and then making sure is matches Some("xxxx")
fn trim_flight<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let callsign = String::deserialize(deserializer)?;
    Ok(Some(S!(callsign.trim_end())))
}

// TODO just make everything pub
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Aircraft {
    #[expect(clippy::struct_field_names)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response<T> {
    pub(crate) response: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdsbdbResponse {
    pub(crate) aircraft: Aircraft,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) flightroute: Option<Flightroute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombinedResponse {
    pub(crate) aircraft: Aircraft,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) flightroute: Option<Flightroute>,
    pub(crate) callsign: Option<String>,
    pub(crate) altitude: i64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Tar1090Aircraft {
    #[serde(rename(serialize = "mode_s"))]
    pub(crate) hex: String,
    #[serde(rename(serialize = "altitude"))]
    #[serde(deserialize_with = "parse_op_i64")]
    pub(crate) alt_baro: Option<i64>,
    #[serde(
        default,
        deserialize_with = "trim_flight",
        rename(serialize = "callsign"),
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) flight: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tar1090Response {
    pub(crate) aircraft: Vec<Tar1090Aircraft>,
}
