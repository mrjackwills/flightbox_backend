use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use tracing::{error, trace};

use crate::{S, parse_env::AppEnv};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SysInfo {
    pub uptime: usize,
    pub api_version: String,
    pub internal_ip: String,
    pub uptime_app: u64,
    pub screen_on: bool,
    pub uptime_ws: u64,
}

// busctl --user set-property org.gnome.Mutter.DisplayConfig /org/gnome/Mutter/DisplayConfig org.gnome.Mutter.DisplayConfig PowerSaveMode i 0
// Common arguments for toggling screen power, and getting screen status
const MUTTER: &str = "org.gnome.Mutter.DisplayConfig";
const WAYLAND_ARGS: [&str; 4] = [
    MUTTER,
    "/org/gnome/Mutter/DisplayConfig",
    MUTTER,
    "PowerSaveMode",
];

// X11 stdout search term
const X11_SEARCH_TERM: &str = "Monitor is O";
// THIS needs to match the length of above
const X11_SEARCH_TERM_LEN: u8 = 12;

const BUSCTL: &str = "busctl";
const XSET: &str = "xset";

const NA: &str = "N/A";

impl SysInfo {
    async fn get_ip(app_env: &AppEnv) -> String {
        let ip = read_to_string(&app_env.location_ip_address)
            .await
            .unwrap_or_else(|_| NA.into());
        if ip.len() > 1 {
            ip.trim().to_owned()
        } else {
            NA.into()
        }
    }

    /// Wayland, get screen status, defaults to false
    /// use app_env to see whether in WAYLAND or X11
    pub async fn screen_status(app_env: &AppEnv) -> bool {
        if app_env.wayland {
            match tokio::process::Command::new(BUSCTL)
                .args(["--user", "get-property"].into_iter().chain(WAYLAND_ARGS))
                .output()
                .await
            {
                Ok(output) => String::from_utf8(output.stdout).unwrap_or_default().trim() == "i 0",
                Err(e) => {
                    error!("wayland::{e:?}");
                    false
                }
            }
        } else {
            match tokio::process::Command::new(XSET)
                .args(["-display", ":0.0", "q"])
                .output()
                .await
            {
                Ok(output) => {
                    let stdout = String::from_utf8(output.stdout).unwrap_or_default();
                    // This will either be "Monitor is On" or "Monitor is Off", so just take the first char after "O"
                    // if "n" monitor is on, else off
                    let next_char = stdout
                        .match_indices(X11_SEARCH_TERM)
                        .map(|i| {
                            stdout
                                .split_at(i.0 + usize::from(X11_SEARCH_TERM_LEN))
                                .1
                                .chars()
                                .take(1)
                                .collect::<String>()
                                .trim()
                                .to_owned()
                        })
                        .collect::<Vec<_>>();
                    next_char.first() == Some(&S!("n"))
                }
                Err(e) => {
                    error!("XSET error:: {e:?}");
                    false
                }
            }
        }
    }

    /// Turn screen on or off, use app_env to see whether in WAYLAND or X11
    pub fn toggle_screen(app_env: &AppEnv, on: bool) {
        if app_env.wayland {
            let suffix = if on { "0" } else { "1" };
            match std::process::Command::new(BUSCTL)
                .args(
                    ["--user", "set-property"]
                        .into_iter()
                        .chain(WAYLAND_ARGS)
                        .chain(["i", suffix]),
                )
                .spawn()
            {
                Ok(_) => trace!("screen status changed"),
                Err(e) => {
                    error!("toggle::{e:?}");
                }
            }
        } else {
            let switch = if on { "on" } else { "off" };
            match std::process::Command::new(XSET)
                .args(["-display", ":0.0", "dpms", "force", switch])
                .spawn()
            {
                Ok(_) => trace!("screen status changed"),
                Err(e) => {
                    error!("toggle::{e:?}");
                }
            }
        }
    }

    /// Read sysfile to get computer uptime, returns 0 if any errors
    async fn get_uptime() -> usize {
        let uptime = read_to_string("/proc/uptime").await.unwrap_or_default();
        let (uptime, _) = uptime.split_once('.').unwrap_or_default();
        uptime.parse::<usize>().unwrap_or_default()
    }

    pub async fn new(app_env: &AppEnv, ws_connect_at: &Instant) -> Self {
        Self {
            internal_ip: Self::get_ip(app_env).await,
            uptime: Self::get_uptime().await,
            uptime_app: std::time::SystemTime::now()
                .duration_since(app_env.start_time)
                .map_or(0, |value| value.as_secs()),
            uptime_ws: ws_connect_at.elapsed().as_secs(),
            api_version: env!("CARGO_PKG_VERSION").into(),
            screen_on: Self::screen_status(app_env).await,
        }
    }
}

// SysInfo tests
//
/// cargo watch -q -c -w src/ -x 'test sysinfo -- --test-threads=1 --nocapture'
#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::{C, S};

    use super::*;

    fn setup_test_env(location_ip_address: String) -> AppEnv {
        let na = S!("na");
        AppEnv {
            location_ip_address,
            log_level: tracing::Level::INFO,
            start_time: SystemTime::now(),
            url_adsbdb: C!(na),
            url_tar0190: C!(na),
            wayland: true,
            ws_address: C!(na),
            ws_api_key: C!(na),
            ws_password: C!(na),
            ws_token_address: na,
        }
    }

    #[tokio::test]
    async fn sysinfo_getuptime_ok() {
        // FIXTURES
        setup_test_env(S!());

        // ACTIONS
        let result = SysInfo::get_uptime().await;

        // CHECK
        // Assumes ones computer has been turned on for one minute
        assert!(result > 60);
        // cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_ip_na() {
        // FIXTURES
        let app_env = setup_test_env(S!());

        // ACTIONS
        let result = SysInfo::get_ip(&app_env).await;

        // CHECK
        assert_eq!(result, "N/A");
    }

    #[tokio::test]
    async fn sysinfo_get_ip_ok() {
        // FIXTURES
        let app_env = setup_test_env(S!("./ip.addr"));

        // ACTIONS
        let result = SysInfo::get_ip(&app_env).await;

        // CHECK
        assert_eq!(result, "123.123.123.123");
    }

    #[tokio::test]
    async fn sysinfo_screen_status_ok() {
        // FIXTURES
        let app_env = setup_test_env(S!());

        // ACTIONS
        let result = SysInfo::screen_status(&app_env).await;

        println!("{result}");
        // CHECK
        // Assumes ones computer has been turned on for one minute
        // assert!(result > 60);
        // cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_sysinfo_ok() {
        // FIXTURES
        let app_env = setup_test_env(S!("./ip.addr"));
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // let now = Instant::now();

        // ACTIONS
        let result = SysInfo::new(&app_env, &Instant::now()).await;

        // CHECK
        assert_eq!(result.internal_ip, "123.123.123.123");
        assert_eq!(result.api_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(result.uptime_app, 1);
        assert!(result.uptime > 60);
    }
}
