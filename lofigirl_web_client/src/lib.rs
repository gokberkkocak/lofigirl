mod storage;
mod view;

use std::sync::Arc;

use lofigirl_shared_common::{
    api::{Action, ScrobbleRequest, SessionRequest, SessionResponse, TokenRequest, TokenResponse},
    config::{LastFMClientPasswordConfig, LastFMClientSessionConfig, ListenBrainzConfig},
    track::Track,
    CLIENT_PING_INTERVAL, HEALTH_END_POINT, LASTFM_SESSION_END_POINT, SEND_END_POINT,
    TOKEN_END_POINT, TRACK_SOCKET_END_POINT,
};

use gloo_net::{
    http::{Method, Request},
    websocket::{futures::WebSocket, Message},
};
use seed::{
    app::CmdHandle,
    futures::{lock::Mutex, stream::SplitStream, SinkExt, StreamExt as _},
    prelude::{
        cmds, wasm_bindgen,
        web_sys::{HtmlInputElement, Url as WebUrl},
        Orders,
    },
    virtual_dom::ElRef,
    App, Url,
};
// use seed::prelude::*;

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    lastfm_form: LastFMForm,
    listenbrainz_form: ListenBrainzForm,
    lastfm_config: Option<LastFMClientSessionConfig>,
    listenbrainz_config: Option<ListenBrainzConfig>,
    session_token: Option<String>,
    server_form: ServerForm,
    server_url: Option<String>,
    page: Page,
    current_track: Track,
    is_scrobbling: bool,
    url: LofiStreamUrlForm,
    tx_handle: Option<CmdHandle>,
    rx_handle: Option<CmdHandle>,
}

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let lastfm = storage::get_lastfm_config();
    let listenbrainz = storage::get_listenbrainz_token();
    let session_token = storage::get_session_token();
    let server = storage::get_server_url();
    Model {
        lastfm_form: Default::default(),
        listenbrainz_form: Default::default(),
        lastfm_config: lastfm,
        listenbrainz_config: listenbrainz,
        server_form: Default::default(),
        server_url: server,
        page: Page::Root,
        current_track: Default::default(),
        session_token,
        is_scrobbling: false,
        url: Default::default(),
        tx_handle: None,
        rx_handle: None,
    }
}

#[derive(Debug, Default)]
struct LastFMForm {
    username_input: ElRef<HtmlInputElement>,
    password_input: ElRef<HtmlInputElement>,
}

#[derive(Debug, Default)]
struct ListenBrainzForm {
    token: ElRef<HtmlInputElement>,
}

#[derive(Debug, Default)]
struct ServerForm {
    server: ElRef<HtmlInputElement>,
}

#[derive(Debug, Default)]
struct LofiStreamUrlForm {
    url: ElRef<HtmlInputElement>,
}

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
#[derive(Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    LastFMFormSubmitted,
    ListenBrainzFormSubmitted,
    ServerFormSubmitted,
    CleanLastFM,
    CleanToken,
    CleanListenbrainz,
    CleanServer,
    LastFMSessionReceived(LastFMClientSessionConfig),
    UrlChanged(Page),
    StopPlaying,
    ServerHealthResponded(String),
    StartPlaying,
    UpdateTokenThenPlay(String),
    ListenSocket(Arc<Mutex<SplitStream<WebSocket>>>),
    NewTrackReceived(Arc<Mutex<SplitStream<WebSocket>>>, Track),
    PongReceived,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::LastFMFormSubmitted => {
            let form = &model.lastfm_form;
            let username = form.username_input.get().unwrap().value();
            let password = form.password_input.get().unwrap().value();

            let server = model.server_url.clone().unwrap();
            let url = format!("{}{}", server, LASTFM_SESSION_END_POINT);

            orders.perform_cmd(async move {
                let session_response =
                    fetch_lastfm_session(&url, LastFMClientPasswordConfig { username, password })
                        .await
                        .unwrap();
                Msg::LastFMSessionReceived(session_response.session_config)
            });
        }
        Msg::ListenBrainzFormSubmitted => {
            let form = &model.listenbrainz_form;
            let token = form.token.get().unwrap().value();
            storage::set_listenbrainz_token(&token);
            model.listenbrainz_config = Some(ListenBrainzConfig { token });
        }
        Msg::ServerFormSubmitted => {
            let form = &model.server_form;
            let server_url = form.server.get().unwrap().value();
            orders.perform_cmd(async move {
                check_server_health(&server_url).await.unwrap();
                Msg::ServerHealthResponded(server_url)
            });
        }
        Msg::UrlChanged(p) => {
            model.page = p;
        }
        Msg::CleanLastFM => {
            model.lastfm_config = None;
            storage::remove_lastfm_config();
        }
        Msg::CleanListenbrainz => {
            model.listenbrainz_config = None;
            storage::remove_listenbrainz_token();
        }
        Msg::CleanServer => {
            model.server_url = None;
            storage::remove_server_url();
        }
        Msg::StopPlaying => {
            model.is_scrobbling = false;
            model.current_track = Default::default();
            model.tx_handle = None;
            model.rx_handle = None;
        }
        Msg::LastFMSessionReceived(s) => {
            storage::set_lastfm_config(&s);
            model.lastfm_config = Some(s);
        }
        Msg::ServerHealthResponded(url) => {
            storage::set_server_url(&url);
            model.server_url = Some(url);
        }
        Msg::CleanToken => {
            model.session_token = None;
            storage::remove_session_token();
        }
        Msg::StartPlaying => {
            // if no session token, set session first
            if model.session_token.is_none() {
                let server = model.server_url.clone().unwrap();
                let l = model
                    .lastfm_config
                    .as_ref()
                    .map(|l| l.session_key.to_owned());
                let ls = model
                    .listenbrainz_config
                    .as_ref()
                    .map(|l| l.token.to_owned());
                orders.perform_cmd(async move {
                    let token = fetch_session_token(&server, l, ls).await.unwrap();
                    Msg::UpdateTokenThenPlay(token.token)
                });
            } else {
                model.is_scrobbling = true;
                let stream = model.url.url.get().unwrap().value();
                let server_url = WebUrl::new(&model.server_url.clone().unwrap()).unwrap();
                // if https > wss else ws
                let protocol = if server_url.protocol() == "https:" {
                    "wss"
                } else {
                    "ws"
                };
                let socket_url = format!(
                    "{}://{}{}",
                    protocol,
                    server_url.host(),
                    TRACK_SOCKET_END_POINT
                );
                let socket = WebSocket::open(&socket_url).unwrap();
                let (mut tx, rx) = socket.split();
                // send initial message and start pinging
                let tx_handle = orders.perform_cmd_with_handle(async move {
                    tx.send(Message::Text(stream)).await.unwrap();
                    loop {
                        cmds::timeout(CLIENT_PING_INTERVAL.as_millis().try_into().unwrap(), || {})
                            .await;
                        tx.send(Message::Bytes(vec![])).await.unwrap();
                    }
                });
                model.tx_handle = Some(tx_handle);
                orders.send_msg(Msg::ListenSocket(Arc::new(Mutex::new(rx))));
            }
        }
        Msg::ListenSocket(rx) => {
            let rx_handle = orders.perform_cmd_with_handle(async move {
                let message = rx.lock().await.next().await.unwrap().unwrap();
                match message {
                    Message::Text(track_str) => {
                        let next_track: Track = serde_json_wasm::from_str(&track_str).unwrap();
                        Msg::NewTrackReceived(rx, next_track)
                    }
                    Message::Bytes(_bytes) => Msg::PongReceived,
                }
            });
            model.rx_handle = Some(rx_handle);
        }
        Msg::NewTrackReceived(rx, next_track) => {
            let current_track = model.current_track.clone();
            model.current_track = next_track.clone();
            let token = model.session_token.clone().unwrap();
            let server = model.server_url.clone().unwrap();
            orders.perform_cmd(async move {
                if !current_track.is_empty() {
                    post_track_action(&token, &server, current_track, Action::Listened)
                        .await
                        .unwrap();
                }
                post_track_action(&token, &server, next_track, Action::PlayingNow)
                    .await
                    .unwrap();
                Msg::ListenSocket(rx)
            });
        }
        Msg::UpdateTokenThenPlay(token) => {
            model.session_token = Some(token.clone());
            storage::set_session_token(&token);
            orders.send_msg(Msg::StartPlaying);
        }
        Msg::PongReceived => {}
    }
}

async fn fetch_lastfm_session(
    url: &str,
    password_config: LastFMClientPasswordConfig,
) -> anyhow::Result<SessionResponse> {
    let session_response = Request::post(url)
        .method(Method::POST)
        .json(&SessionRequest { password_config })?
        .send()
        .await?
        .json()
        .await?;
    Ok(session_response)
}

async fn fetch_session_token(
    server: &str,
    lastfm_session_key: Option<String>,
    listenbrainz_token: Option<String>,
) -> anyhow::Result<TokenResponse> {
    let url = format!("{}{}", server, TOKEN_END_POINT);
    let token_response = Request::post(&url)
        .method(Method::POST)
        .json(&TokenRequest {
            lastfm_session_key,
            listenbrainz_token,
        })?
        .send()
        .await?
        .json()
        .await?;
    Ok(token_response)
}

async fn post_track_action(
    token: &str,
    server: &str,
    track: Track,
    action: Action,
) -> anyhow::Result<()> {
    Request::post(&format!("{}{}", server, SEND_END_POINT))
        .method(Method::POST)
        .json(&ScrobbleRequest {
            action,
            track,
            token: token.to_owned(),
        })?
        .send()
        .await?;
    Ok(())
}

async fn check_server_health(server: &str) -> anyhow::Result<()> {
    let url = format!("{}{}", server, HEALTH_END_POINT);
    Request::get(&url).method(Method::GET).send().await?;
    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum Page {
    Root,
    Config,
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view::view);
}
