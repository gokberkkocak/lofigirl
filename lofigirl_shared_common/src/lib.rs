pub mod api;
pub mod config;
pub mod track;
pub mod jwt;

mod encrypt;

use std::{sync::LazyLock, time::Duration};

pub static REGULAR_INTERVAL: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(15));
pub static FAST_TRY_INTERVAL: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(5));
pub static STREAM_LAST_READ_TIMEOUT: LazyLock<Duration> =
    LazyLock::new(|| Duration::from_secs(300));
pub static CLIENT_PING_INTERVAL: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(30));
pub static SERVER_PING_TIMEOUT_INTERVAL: LazyLock<Duration> =
    LazyLock::new(|| Duration::from_secs(60));

pub const SEND_END_POINT: &str = "/send";
pub const TRACK_END_POINT: &str = "/track";
pub const TRACK_SOCKET_END_POINT: &str = "/track_ws";
pub const LASTFM_SESSION_END_POINT: &str = "/session";
pub const TOKEN_END_POINT: &str = "/token";
pub const HEALTH_END_POINT: &str = "/health";

pub const ENCRYPTION_KEY_BASE64: &[u8; 44] = b"MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2Nzg=";