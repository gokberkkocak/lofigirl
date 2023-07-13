mod storage;
mod view;

use lofigirl_shared_common::{
    api::{Action, ScrobbleRequest, SessionRequest, SessionResponse, TokenRequest, TokenResponse},
    config::{LastFMClientPasswordConfig, LastFMClientSessionConfig, ListenBrainzConfig},
    track::Track,
    CHILL_TRACK_API_END_POINT, HEALTH_END_POINT, LASTFM_SESSION_END_POINT, REGULAR_INTERVAL,
    SEND_END_POINT, SLEEP_TRACK_API_END_POINT, TOKEN_END_POINT, TRACK_END_POINT,
};

#[cfg(debug_assertions)]
use gloo_console::log;
use gloo_net::http::{Method, Request};
use seed::{
    prelude::{cmds, wasm_bindgen, web_sys::HtmlInputElement, Orders},
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
    current_track: Option<Track>,
    is_scrobbling: bool,
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
    StartPlaying(LofiStream),
    UpdatePlayingStatus(LofiStream, i32),
    LastFMSessionReceived(LastFMClientSessionConfig),
    UrlChanged(Page),
    SubmitTrack(Track, LofiStream, i32),
    StopPlaying,
    ServerHealthResponded(String),
    UpdateTokenThenSubmit(String, LofiStream, i32),
}
#[derive(Debug, Clone, Copy)]
enum LofiStream {
    Chill,
    Sleep,
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
        Msg::UpdatePlayingStatus(stream, count) => {
            if model.is_scrobbling {
                let server = model.server_url.clone().unwrap();
                let token = model.session_token.clone();
                let l = match &model.lastfm_config {
                    Some(l) => Some(l.session_key.to_owned()),
                    None => None,
                };
                let ls = match &model.listenbrainz_config {
                    Some(l) => Some(l.token.to_owned()),
                    None => None,
                };
                orders.perform_cmd(async move {
                    match token {
                        Some(_) => {
                            let track = fetch_track(&server, stream).await.unwrap();
                            Msg::SubmitTrack(track, stream, count)
                        }
                        None => {
                            let token = fetch_session_token(&server, l, ls).await.unwrap();
                            Msg::UpdateTokenThenSubmit(token.token, stream, count)
                        }
                    }
                });
            }
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
        Msg::SubmitTrack(track, s, mut count) => {
            // update model track
            let server = model.server_url.clone().unwrap();
            let token = model.session_token.clone();
            let token = token.unwrap();
            let current_track = model.current_track.take();
            if let Some(t) = current_track.filter(|t| *t != track) {
                if count > 3 {
                    let cloned_token = token.clone();
                    orders.perform_cmd(async move {
                        #[cfg(debug_assertions)]
                        log!("Scrobbled");
                        post_track_action(&cloned_token, &server, t, Action::Listened)
                            .await
                            .unwrap();
                    });
                }
                count = 0;
            }
            model.current_track = Some(track.clone());
            let server = model.server_url.clone().unwrap();
            #[cfg(debug_assertions)]
            log!("Count", count);
            orders.perform_cmd(async move {
                if count == 1 {
                    post_track_action(&token, &server, track, Action::PlayingNow)
                        .await
                        .unwrap();
                }
                cmds::timeout(
                    REGULAR_INTERVAL.as_millis().try_into().unwrap(),
                    move || Msg::UpdatePlayingStatus(s, count + 1),
                )
                .await
            });
        }
        Msg::StopPlaying => {
            model.is_scrobbling = false;
            model.current_track = None;
        }
        Msg::LastFMSessionReceived(s) => {
            storage::set_lastfm_config(&s);
            model.lastfm_config = Some(s);
        }
        Msg::ServerHealthResponded(url) => {
            storage::set_server_url(&url);
            model.server_url = Some(url);
        }
        Msg::UpdateTokenThenSubmit(token, stream, count) => {
            storage::set_session_token(&token);
            model.session_token = Some(token);
            orders.perform_cmd(async move { Msg::UpdatePlayingStatus(stream, count) });
        }
        Msg::CleanToken => {
            model.session_token = None;
            storage::remove_session_token();
        }
        Msg::StartPlaying(stream) => {
            model.is_scrobbling = true;
            orders.perform_cmd(async move { Msg::UpdatePlayingStatus(stream, 1) });
        }
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

async fn fetch_track(server: &str, stream: LofiStream) -> anyhow::Result<Track> {
    let url = format!(
        "{}{}{}",
        server,
        TRACK_END_POINT,
        match stream {
            LofiStream::Chill => CHILL_TRACK_API_END_POINT,
            LofiStream::Sleep => SLEEP_TRACK_API_END_POINT,
        }
    );
    let track = Request::get(&url)
        .method(Method::GET)
        .send()
        .await?
        .json()
        .await?;
    Ok(track)
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
