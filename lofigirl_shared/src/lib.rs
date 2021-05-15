pub mod track;
pub mod config;

use std::time::Duration;

use once_cell::sync::Lazy;

pub static REGULAR_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(15));
pub static FAST_TRY_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));
