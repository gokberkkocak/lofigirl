pub mod config;
pub mod track;
pub mod api;

use std::time::Duration;

use once_cell::sync::Lazy;

pub static REGULAR_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(15));
pub static FAST_TRY_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));
pub const CHILL_API_END_POINT: &str = "chill";
pub const SLEEP_API_END_POINT: &str = "sleep";
