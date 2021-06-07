use seed::prelude::*;
use seed::*;

use crate::{LofiStream, Model, Msg, Page};

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
pub(crate) fn view(model: &Model) -> Vec<Node<Msg>> {
    let top = div![div![
        // "ROOT",
        button!["Home", ev(Ev::Click, |_| Msg::UrlChanged(Page::Root)),],
        button!["Config", ev(Ev::Click, |_| Msg::UrlChanged(Page::Config)),]
    ],];
    let body = match model.page {
        Page::Root => match &model.current_track {
            Some(t) => {
                div![
                    div![button![
                        "STOP scrobbling",
                        ev(Ev::Click, |_| Msg::StopPlaying),
                    ]],
                    div![format!("Current song: {}", t)]
                ]
            }
            None => {
                div![
                    div![button![
                        "Start scrobbling - CHILL",
                        ev(Ev::Click, |_| Msg::UpdatePlayingStatus(
                            LofiStream::Chill,
                            1
                        )),
                    ]],
                    div![button![
                        "Start scrobbling - SLEEP",
                        ev(Ev::Click, |_| Msg::UpdatePlayingStatus(
                            LofiStream::Sleep,
                            1
                        )),
                    ]]
                ]
            }
        },
        Page::Config => {
            let lastfm = div![
                div!["LastFM"],
                match &model.lastfm_config {
                    Some(l) => {
                        div![
                            format!("Stored LastFM session_key:  {}", l.session_key),
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
                            div![button![
                                "Submit",
                                ev(Ev::Click, |_| Msg::LastFMFormSubmitted),
                            ]]
                        ]
                    }
                }
            ];
            let listenbrainz = div![
                div!["ListenBrainz"],
                match &model.listenbrainz_config {
                    Some(t) => {
                        div![
                            format!("Stored ListenBrainz token:  {}", t.token),
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
                }
            ];
            let server = div![
                div!["Server"],
                match &model.server_url {
                    Some(s) => {
                        div![
                            format!("Using {}", s),
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
                }
            ];

            let token = div![
                div!["Session Info"],
                match &model.session_token {
                    Some(s) => {
                        div![
                            format!("Using session token: {}", s),
                            button!["CLEAN", ev(Ev::Click, |_| Msg::CleanToken),],
                        ]
                    }
                    None => {
                        div!["No session"]
                    }
                }
            ];
            div![div![server], div![lastfm], div![listenbrainz], div![token]]
        }
    };
    nodes![top, body]
}
