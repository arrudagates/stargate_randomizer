#![recursion_limit = "256"]

use rand::{thread_rng, Rng};
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use yew::services::{
    fetch::{FetchTask, Request},
    FetchService,
};
use yew::{
    format::{Json, Nothing},
    services::fetch::Response,
};
use yew::{prelude::*, services::console};

struct Model {
    link: ComponentLink<Self>,
    current_episode: (i32, i32),
    sg1: bool,
    atlantis: bool,
    universe: bool,
    fetch_task: Option<FetchTask>,
    episode: Option<Episode>,
    error: Option<anyhow::Error>,
}

enum Msg {
    GetRandom,
    Toggle(Shows),
    ReceiveResponse(Result<Vec<Episode>, anyhow::Error>),
}

enum Shows {
    SG1,
    Atlantis,
    Universe,
}

#[derive(Deserialize, Debug, Clone)]
struct Episode {
    title: String,
    overview: String,
    language: String,
}

impl Model {
    fn view_episode(&self) -> Html {
        match &self.episode {
            Some(episode) => {
                html! {
                    <>
                        <p>{ format!("Title: {}", episode.title) }</p>
                        <p>{ format!("Overview: {}", episode.overview) }</p>
                    </>
                }
            }
            None => {
                html! {
                     <div></div>
                }
            }
        }
    }
    fn view_fetching(&self) -> Html {
        if self.fetch_task.is_some() {
            html! { <p>{ "Fetching episode..." }</p> }
        } else {
            html! { <p></p> }
        }
    }
    fn view_error(&self) -> Html {
        if let Some(ref error) = self.error {
            html! { <p>{ error.clone() }</p> }
        } else {
            html! {}
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            sg1: true,
            atlantis: false,
            universe: false,
            current_episode: (0, 0),
            fetch_task: None,
            episode: None,
            error: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetRandom => {
                let mut rng = thread_rng();
                let season = rng.gen_range(1..10);
                let episode = match season.clone() {
                    1 => (season, rng.gen_range(1..21)),
                    2 | 3 | 4 | 5 | 6 | 7 => (season, rng.gen_range(1..22)),
                    8 | 9 | 10 => (season, rng.gen_range(1..20)),
                    _ => (0, 0),
                };
                self.current_episode = episode;
                console::ConsoleService::log(
                    format!("Season: {}, episode: {}", episode.0, episode.1).as_str(),
                );

                let request = Request::get(format!(
                    "https://api.trakt.tv/shows/tt0118480/seasons/{}/episodes/{}/translations/en",
                    episode.0, episode.1
                ))
                .header(
                    "trakt-api-key",
                    "838d2d9cb16844abe679e743052404feec745399649976ba725e04927986a73c", // this is the client id, not the client secret
                )
                .header("trakt-api-version", 2)
                .body(Nothing)
                .expect("Could not build request.");

                let callback = self.link.callback(
                    |response: Response<Json<Result<Vec<Episode>, anyhow::Error>>>| {
                        let Json(data) = response.into_body();
                        Msg::ReceiveResponse(data)
                    },
                );

                let task = FetchService::fetch(request, callback).expect("failed to start request");
                self.fetch_task = Some(task);

                true
            }

            Msg::ReceiveResponse(response) => {
                match response {
                    Ok(episode) => {
                        self.episode = Some(episode.first().unwrap().to_owned());
                    }
                    Err(error) => self.error = Some(error),
                }
                self.fetch_task = None;
                true
            }

            Msg::Toggle(show) => {
                match show {
                    Shows::SG1 => self.sg1 = !self.sg1,
                    Shows::Atlantis => self.atlantis = !self.atlantis,
                    Shows::Universe => self.universe = !self.universe,
                }
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
                <div class="app">
                    <header class="app-header">

            <label class="checkbox">
                    <input type="checkbox"
                    onclick= self.link.callback(|_| Msg::Toggle(Shows::SG1))
            checked= self.sg1                        />
        {"SG-1"}
        </label>

            <label class="checkbox">
                    <input type="checkbox"
                    onclick=self.link.callback(|_| Msg::Toggle(Shows::Atlantis))
            checked= self.atlantis
            />
        {"Atlantis"}
        </label>

            <label class="checkbox">
                    <input type="checkbox"
                    onclick= self.link.callback(|_| Msg::Toggle(Shows::Universe))
            checked= self.universe
            />
        {"Universe"}
        </label>
                             <p>
                    { "Stargate Randomizer" }
             </p>

            { self.view_fetching() }
            { self.view_episode() }
            { self.view_error() }

                    <div>
                    <button onclick=self.link.callback(|_| Msg::GetRandom)>{ "Jaunt through the orifice" }</button>
                    <p>{ format!("Season {}, Episode: {}", self.current_episode.0, self.current_episode.1) }</p>
                    </div>

                    </header>
                    </div>
            }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
