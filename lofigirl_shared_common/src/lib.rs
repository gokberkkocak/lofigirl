pub mod api;
pub mod config;
pub mod track;

use std::{sync::LazyLock, time::Duration};

pub static REGULAR_INTERVAL: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(15));
pub static FAST_TRY_INTERVAL: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(5));
pub static STREAM_LAST_READ_TIMEOUT: LazyLock<Duration> =
    LazyLock::new(|| Duration::from_secs(300));
pub const SEND_END_POINT: &str = "/send";
pub const TRACK_END_POINT: &str = "/track";
pub const TRACK_SOCKET_END_POINT: &str = "/track_ws";
pub const LASTFM_SESSION_END_POINT: &str = "/session";
pub const TOKEN_END_POINT: &str = "/token";
pub const HEALTH_END_POINT: &str = "/health";
pub const CHILL_TRACK_API_END_POINT: &str = "/1";
pub const SLEEP_TRACK_API_END_POINT: &str = "/2";
