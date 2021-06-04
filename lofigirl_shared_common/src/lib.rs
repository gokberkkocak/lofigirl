pub mod api;
pub mod config;
pub mod track;

use std::time::Duration;

use once_cell::sync::Lazy;

pub static REGULAR_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(15));
pub static FAST_TRY_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));
pub const SEND_END_POINT: &str = "/send";
pub const TRACK_END_POINT: &str = "/track";
pub const SESSION_END_POINT: &str = "/session";
pub const TOKEN_END_POINT: &str = "/token";
pub const HEALTH_END_POINT: &str = "/health";
pub const CHILL_TRACK_API_END_POINT: &str = "chill";
pub const SLEEP_TRACK_API_END_POINT: &str = "sleep";
