use crate::app_error::AppError;
use std::{collections::HashMap, env, fs, time::SystemTime};
type EnvHashMap = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub location_ip_address: String,
    pub log_level: tracing::Level,
    pub start_time: SystemTime,
    pub url_adsbdb: String,
    pub url_tar0190: String,
    pub wayland: bool,
    pub ws_address: String,
    pub ws_api_key: String,
    pub ws_password: String,
    pub ws_token_address: String,
}

impl AppEnv {
    /// Check a given file actually exists on the file system
    fn check_file_exists(filename: String) -> Result<String, AppError> {
        match fs::metadata(&filename) {
            Ok(_) => Ok(filename),
            Err(_) => Err(AppError::FileNotFound(filename)),
        }
    }

    /// Parse "true" or "false" to bool, else false
    fn parse_boolean(key: &str, map: &EnvHashMap) -> bool {
        map.get(key).is_some_and(|value| value == "true")
    }

    /// Parse debug and/or trace into tracing level
    fn parse_log(map: &EnvHashMap) -> tracing::Level {
        if Self::parse_boolean("LOG_TRACE", map) {
            tracing::Level::TRACE
        } else if Self::parse_boolean("LOG_DEBUG", map) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }

    fn parse_string(key: &str, map: &EnvHashMap) -> Result<String, AppError> {
        map.get(key).map_or(
            Err(AppError::NotFound(key.into())),
            |value| Ok(value.into()),
        )
    }

    /// Load, and parse .env file, return AppEnv
    fn generate() -> Result<Self, AppError> {
        let env_map = env::vars()
            .map(|i| (i.0, i.1))
            .collect::<HashMap<String, String>>();

        Ok(Self {
            location_ip_address: Self::check_file_exists(Self::parse_string(
                "LOCATION_IP_ADDRESS",
                &env_map,
            )?)?,
            log_level: Self::parse_log(&env_map),
            start_time: SystemTime::now(),
            url_adsbdb: Self::parse_string("URL_ADSBDB", &env_map)?,
            url_tar0190: Self::parse_string("URL_TAR1090", &env_map)?,
            wayland: Self::parse_boolean("WAYLAND", &env_map),
            ws_address: Self::parse_string("WS_ADDRESS", &env_map)?,
            ws_api_key: Self::parse_string("WS_APIKEY", &env_map)?,
            ws_password: Self::parse_string("WS_PASSWORD", &env_map)?,
            ws_token_address: Self::parse_string("WS_TOKEN_ADDRESS", &env_map)?,
        })
    }

    pub fn get_env() -> Self {
        let local_env = ".env";
        let app_env = "/app_env/.env";

        let env_path = if std::fs::metadata(app_env).is_ok() {
            app_env
        } else if std::fs::metadata(local_env).is_ok() {
            local_env
        } else {
            println!("\n\x1b[31munable to load env file\x1b[0m\n");
            std::process::exit(1);
        };

        dotenvy::from_path(env_path).ok();
        match Self::generate() {
            Ok(s) => s,
            Err(e) => {
                println!("\n\x1b[31m{e}\x1b[0m\n");
                std::process::exit(1);
            }
        }
    }
}

/// Run tests with
///
/// cargo watch -q -c -w src/ -x 'test env_ -- --nocapture'
#[expect(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use dotenvy::from_path;

    use crate::S;

    use super::*;

    #[test]
    fn env_missing_env() {
        let mut map = HashMap::new();
        map.insert(S!("not_fish"), S!("not_fish"));
        // ACTION
        let result = AppEnv::parse_string("fish", &map);

        // CHECK
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "missing env: 'fish'");
    }

    #[test]
    fn env_parse_string_valid() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert(S!("RANDOM_STRING"), S!("123"));

        // ACTION
        let result = AppEnv::parse_string("RANDOM_STRING", &map).unwrap();

        // CHECK
        assert_eq!(result, "123");

        // FIXTURES
        let mut map = HashMap::new();
        map.insert(S!("RANDOM_STRING"), S!("hello_world"));

        // ACTION
        let result = AppEnv::parse_string("RANDOM_STRING", &map).unwrap();

        // CHECK
        assert_eq!(result, "hello_world");
    }
    #[test]
    fn env_parse_log_valid() {
        // FIXTURES
        let map = HashMap::from([(S!("RANDOM_STRING"), S!("123"))]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([(S!("LOG_DEBUG"), S!("false"))]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([(S!("LOG_TRACE"), S!("false"))]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([
            (S!("LOG_DEBUG"), S!("false")),
            (S!("LOG_TRACE"), S!("false")),
        ]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([
            (S!("LOG_DEBUG"), S!("true")),
            (S!("LOG_TRACE"), S!("false")),
        ]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::DEBUG);

        // FIXTURES
        let map = HashMap::from([(S!("LOG_DEBUG"), S!("true")), (S!("LOG_TRACE"), S!("true"))]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::TRACE);

        // FIXTURES
        let map = HashMap::from([
            (S!("LOG_DEBUG"), S!("false")),
            (S!("LOG_TRACE"), S!("true")),
        ]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::TRACE);
    }

    #[test]
    fn env_parse_boolean_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert(S!("valid_true"), S!("true"));
        map.insert(S!("valid_false"), S!("false"));
        map.insert(S!("invalid_but_false"), S!("as"));

        // ACTION
        let result01 = AppEnv::parse_boolean("valid_true", &map);
        let result02 = AppEnv::parse_boolean("valid_false", &map);
        let result03 = AppEnv::parse_boolean("invalid_but_false", &map);
        let result04 = AppEnv::parse_boolean("missing", &map);

        // CHECK
        assert!(result01);
        assert!(!result02);
        assert!(!result03);
        assert!(!result04);
    }

    #[test]
    fn env_return_appenv() {
        from_path("./docker/.env").ok();

        // ACTION
        let result = AppEnv::generate();

        assert!(result.is_ok());
    }
}
