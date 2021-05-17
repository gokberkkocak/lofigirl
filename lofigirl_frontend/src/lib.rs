// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

mod storage;

use lofigirl_shared::{
    config::{LastFMConfig, ListenBrainzConfig},
    listener::Listener,
};
use seed::prelude::web_sys::HtmlInputElement;
use seed::prelude::*;
use seed::*;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let mut listener = Listener::new();
    let lastfm_config = storage::get_lastfm_config();
    if let Some(lastfm) = lastfm_config {
        listener.set_lastfm_listener(&lastfm).unwrap();
    }
    let listenbrainz_token = storage::get_listenbrainz_token();
    if let Some(token) = listenbrainz_token {
        listener
            .set_listenbrainz_listener(&ListenBrainzConfig { token })
            .unwrap();
    }

    Model {
        lastfm_form: Default::default(),
        listenbrainz_form: Default::default(),
        server_form: Default::default(),
        listener,
        counter: 0,
        server_url: Default::default(),
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
#[derive(Default)]
struct Model {
    lastfm_form: LastFMForm,
    listenbrainz_form: ListenBrainzForm,
    server_form: ServerForm,
    listener: Listener,
    server_url: Option<String>,
    counter: i32,
}

impl Model {
    fn set_lastfm_config(&mut self, lastfm: &LastFMConfig) {
        storage::set_lastfm_config(lastfm);
    }
    fn set_listenbrainz_token(&mut self, token: &str) {
        storage::set_listenbrainz_token(token);
    }
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
    Increment,
    LastFMFormSubmitted,
    ListenBrainzFormSubmitted,
    ServerFormSubmitted,
    UpdatePlayingStatus,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.counter += 1,
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
            model.listener.set_lastfm_listener(&lastfm_config).unwrap();
        }
        Msg::ListenBrainzFormSubmitted => {
            let form = &model.listenbrainz_form;
            let token = form.token.get().unwrap().value();
            model.listener.set_listenbrainz_listener(&ListenBrainzConfig{token}).unwrap();
        }
        Msg::ServerFormSubmitted => {
            let form = &model.server_form;
            let server_url = form.server.get().unwrap().value();
            model.listener.set_listenbrainz_listener(&ListenBrainzConfig{token}).unwrap();
        }
        Msg::UpdatePlayingStatus => {}
    }
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    div![
        "This is a counter: ",
        C!["counter"],
        button![model.counter, ev(Ev::Click, |_| Msg::Increment),],
    ]
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
