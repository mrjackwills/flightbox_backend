use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug)]
pub enum MessageValues {
    Valid(ParsedMessage, String),
    Invalid(ErrorData),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "message", content = "body")]
pub enum ParsedMessage {
    Status,
    Flights,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct StructuredMessage {
    data: Option<ParsedMessage>,
    error: Option<ErrorData>,
    unique: String,
}

// TODO
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "error", content = "message")]
pub enum ErrorData {
    Something(String),
}

pub fn to_struct(input: &str) -> Option<MessageValues> {
    if let Ok(data) = serde_json::from_str::<StructuredMessage>(input) {
        if let Some(data) = data.error {
            return Some(MessageValues::Invalid(data));
        }
        if let Some(parsed) = data.data {
            return Some(MessageValues::Valid(parsed, data.unique));
        }
        None
    } else if let Ok(data) = serde_json::from_str::<ErrorData>(input) {
        debug!("Matched error_serialized data");
        Some(MessageValues::Invalid(data))
    } else {
        debug!("not a known input message");
        None
    }
}

// message_incoming

// cargo watch -q -c -w src/ -x 'test message_incoming -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use std::time::UNIX_EPOCH;

    use super::*;

    fn now_string() -> String {
        format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        )
    }

    #[test]
    fn message_incoming_parse_invalid() {
        let data = "";
        let result = to_struct(data);
        assert!(result.is_none());

        let data = "{}";
        let result = to_struct(data);
        assert!(result.is_none());

        let result = to_struct(&now_string());
        assert!(result.is_none());
    }

    #[test]
    fn message_incoming_parse_on_unique_err() {
        let data = r#"
            {
            	"data": {
            		"message" : "on",
            	}
            }"#;
        let result = to_struct(data);
        assert!(result.is_none());
    }

    #[test]
    fn message_incoming_parse_flights_unique_err() {
        let data = r#"
            {
            	"data": {
            		"message" : "flights",
            	}
            }"#;
        let result = to_struct(data);
        assert!(result.is_none());
    }

    #[test]
    fn message_incoming_parse_flights_unique_ok() {
        let now = now_string();

        let data = format!(r#"{{"data":{{"message":"flights"}}, "unique":"{now}"}}"#);
        let result = to_struct(&data);
        assert!(result.is_some());
        let result = result.unwrap();
        match result {
            MessageValues::Valid(ParsedMessage::Flights, unique) => {
                assert_eq!(unique, now);
            }
            _ => unreachable!("Shouldn't have matched this"),
        }
    }

    #[test]
    fn message_incoming_parse_status_unique_err() {
        let data = r#"
            {
            	"data": {
            		"message" : "status",
            	}
            }"#;
        let result = to_struct(data);
        assert!(result.is_none());
    }

    #[test]
    fn message_incoming_parse_status_unique_ok() {
        let now = now_string();

        let data = format!(r#"{{"data":{{"message":"status"}}, "unique":"{now}"}}"#);
        let result = to_struct(&data);
        assert!(result.is_some());
        let result = result.unwrap();
        match result {
            MessageValues::Valid(ParsedMessage::Status, unique) => {
                assert_eq!(unique, now);
            }
            _ => unreachable!("Shouldn't have matched this"),
        }
    }
}
