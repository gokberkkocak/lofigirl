// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

mod storage;
use lofigirl_shared::{
    api::SendInfo,
    config::{LastFMConfig, ListenBrainzConfig},
    track::Track,
    REGULAR_INTERVAL,
};
use seed::prelude::web_sys::HtmlInputElement;
use seed::prelude::*;
use seed::*;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let lastfm = storage::get_lastfm_config();
    let listenbrainz = storage::get_listenbrainz_token();
    let server = storage::get_server_url();
    Model {
        lastfm_form: Default::default(),
        listenbrainz_form: Default::default(),
        lastfm_config: lastfm,
        listenbrainz_config: listenbrainz,
        server_form: Default::default(),
        server_url: server,
        page: Page::Root,
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    lastfm_form: LastFMForm,
    listenbrainz_form: ListenBrainzForm,
    lastfm_config: Option<LastFMConfig>,
    listenbrainz_config: Option<ListenBrainzConfig>,
    server_form: ServerForm,
    server_url: Option<String>,
    page: Page,
}

#[derive(Debug, Default)]
struct LastFMForm {
    api_key_input: ElRef<HtmlInputElement>,
    api_secret_input: ElRef<HtmlInputElement>,
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
#[derive(Copy, Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    LastFMFormSubmitted,
    ListenBrainzFormSubmitted,
    ServerFormSubmitted,
    CleanLastFM,
    CleanListenbrainz,
    CleanServer,
    UpdatePlayingStatus(Stream),
    UrlChanged(Page),
}
#[derive(Debug, Clone, Copy)]
enum Stream {
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
            let api_key = form.api_key_input.get().unwrap().value();
            let api_secret = form.api_secret_input.get().unwrap().value();
            let lastfm_config = LastFMConfig {
                api_key,
                api_secret,
                username,
                password,
            };
            storage::set_lastfm_config(&lastfm_config);
            model.lastfm_config = Some(lastfm_config);
        }
        Msg::ListenBrainzFormSubmitted => {
            let form = &model.listenbrainz_form;
            let token = form.token.get().unwrap().value();
            storage::set_listenbrainz_token(&token);
            model.listenbrainz_config = Some(ListenBrainzConfig { token });
        }
        Msg::ServerFormSubmitted => {
            let form = &model.server_form;
            let url = form.server.get().unwrap().value();
            storage::set_server_url(&url);
            model.server_url = Some(url);
        }
        Msg::UpdatePlayingStatus(s) => {
            // do request
            let l = model.lastfm_config.clone();
            let ls = model.listenbrainz_config.clone();
            let server = model.server_url.clone();
            orders.perform_cmd(async move {
                send_info(l, ls, server.unwrap(), s).await.unwrap();
                std::thread::sleep(*REGULAR_INTERVAL);
                Msg::UpdatePlayingStatus(s)
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
    }
}

async fn send_info(
    l: Option<LastFMConfig>,
    ls: Option<ListenBrainzConfig>,
    server: String,
    s: Stream,
) -> fetch::Result<()> {
    let base = server;
    let url = format!(
        "{}/{}",
        base,
        match s {
            Stream::Chill => "chill",
            Stream::Sleep => "sleep",
        }
    );
    let track: Track = Request::new(url)
        .method(Method::Get)
        .fetch()
        .await?
        .check_status()?
        .json()
        .await?;
    Request::new(format!("{}/{}", base, "send"))
        .method(Method::Post)
        .json(&SendInfo {
            lastfm: l,
            listenbrainz: ls,
            action: lofigirl_shared::api::Action::PlayingNow,
            track,
        })?
        .fetch()
        .await?
        .check_status()?
        .json()
        .await?;
    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum Page {
    Root,
    Config,
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Vec<Node<Msg>> {
    let top = div![div![
        // "ROOT",
        button!["ROOT", ev(Ev::Click, |_| Msg::UrlChanged(Page::Root)),],
        button!["Config", ev(Ev::Click, |_| Msg::UrlChanged(Page::Config)),]
    ],];

    let body = match model.page {
        Page::Root => {
            div![
                div![button![
                    "Start scrobbling - CHILL",
                    ev(Ev::Click, |_| Msg::UpdatePlayingStatus(Stream::Chill)),
                ]],
                div![button![
                    "Start scrobbling - SLEEP",
                    ev(Ev::Click, |_| Msg::UpdatePlayingStatus(Stream::Sleep)),
                ]]
            ]
        }
        Page::Config => {
            let lastfm = match &model.lastfm_config {
                Some(l) => {
                    div![
                        format!("LASTFM - Logged in as {}", l.username),
                        button!["CLEAN", ev(Ev::Click, |_| Msg::CleanLastFM),],
                    ]
                }
                None => {
                    div![
                        div![input![
                            el_ref(&model.lastfm_form.username_input),
                            attrs! {
                                At::Type => "text",
                                At::Placeholder => "Username",
                            },
                        ]],
                        div![input![
                            el_ref(&model.lastfm_form.password_input),
                            attrs! {
                                At::Type => "password",
                                At::Placeholder => "Password"
                            },
                        ]],
                        div![input![
                            el_ref(&model.lastfm_form.api_key_input),
                            attrs! {
                                At::Type => "password",
                                At::Placeholder => "api_key"
                            },
                        ]],
                        div![input![
                            el_ref(&model.lastfm_form.api_secret_input),
                            attrs! {
                                At::Type => "password",
                                At::Placeholder => "api_secret"
                            },
                        ]],
                        div![button![
                            "Submit",
                            ev(Ev::Click, |_| Msg::LastFMFormSubmitted),
                        ]]
                    ]
                }
            };
            let listenbrainz = match &model.listenbrainz_config {
                Some(_) => {
                    div![
                        "LISTENBRAINZ - A token is registered",
                        button!["CLEAN", ev(Ev::Click, |_| Msg::CleanListenbrainz),],
                    ]
                }
                None => {
                    div![
                        div![input![
                            el_ref(&model.listenbrainz_form.token),
                            attrs! {
                                At::Type => "password",
                                At::Placeholder => "Token",
                            },
                        ]],
                        div![button![
                            "Submit",
                            ev(Ev::Click, |_| Msg::ListenBrainzFormSubmitted),
                        ]]
                    ]
                }
            };
            let server = match &model.server_url {
                Some(s) => {
                    div![
                        format!("SERVER - Using {}", s),
                        button!["CLEAN", ev(Ev::Click, |_| Msg::CleanServer),],
                    ]
                }
                None => {
                    div![
                        div![input![
                            el_ref(&model.server_form.server),
                            attrs! {
                                At::Type => "text",
                                At::Placeholder => "url",
                            },
                        ]],
                        div![button![
                            "Submit",
                            ev(Ev::Click, |_| Msg::ServerFormSubmitted),
                        ]]
                    ]
                }
            };
            div![div![lastfm], div![listenbrainz], div![server]]
        }
    };
    nodes![top, body]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
