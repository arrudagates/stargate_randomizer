#![recursion_limit = "512"]

use std::{env, fmt::Display};

use rand::{prelude::SliceRandom, thread_rng, Rng};
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
    current_episode: (Shows, i32, i32),
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
    ReceiveResponse(Result<Episode, anyhow::Error>),
}

#[derive(Copy, Clone)]
enum Shows {
    SG1,
    Atlantis,
    Universe,
}

impl Display for Shows {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shows::SG1 => write!(f, "SG-1"),
            Shows::Atlantis => write!(f, "Atlantis"),
            Shows::Universe => write!(f, "Universe"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Episode {
    name: String,
    overview: String,
    still_path: Option<String>,
}

impl Model {
    fn view_episode(&self) -> Html {
        match &self.episode {
            Some(episode) => {
                html! {
                    <>
                        <div class="card" style="max-width: 40%; height: 70vh; display: flex; flex-direction: column;">
                        <div class="card-image">
                        <figure class="image">
                        <img src=format!("https://image.tmdb.org/t/p/w500{}", if let Some(still) = &episode.still_path {still} else {""}) alt="Placeholder image"/>
                        </figure>
                        </div>
                        <div class="card-content" style="display: flex; flex-direction: column; flex: 1;">
                        <div class="media">
                        <div class="media-content">
                        <p class="title is-4">{&episode.name}</p>
                        </div>
                        </div>

                        <div class="content" style="font-size: 19px; display: flex; flex-direction: column; justify-content: space-between; flex: 1;">
                    {&episode.overview}
                        <br/>
                        <p>{format!("Stargate {}, Season: {}, Episode: {}", self.current_episode.0, self.current_episode.1, self.current_episode.2)}</p>
                        </div>
                        </div>
                        </div>
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
            current_episode: (Shows::SG1, 0, 0),
            fetch_task: None,
            episode: None,
            error: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetRandom => {
                let key: &str = env!("TMDB_KEY");
                let mut rng = thread_rng();
                let mut show = vec![];
                if self.sg1 {
                    show.push(Shows::SG1)
                }
                if self.atlantis {
                    show.push(Shows::Atlantis)
                }
                if self.universe {
                    show.push(Shows::Universe)
                }
                show.shuffle(&mut rng);
                let show = show[0];
                let season = match show {
                    Shows::SG1 => rng.gen_range(1..10),
                    Shows::Atlantis => rng.gen_range(1..5),
                    Shows::Universe => rng.gen_range(1..2),
                };
                let episode = match (show, season.clone()) {
                    (Shows::SG1, 1) => (show, season, rng.gen_range(1..21)),
                    (Shows::SG1, 2)
                    | (Shows::SG1, 3)
                    | (Shows::SG1, 4)
                    | (Shows::SG1, 5)
                    | (Shows::SG1, 6)
                    | (Shows::SG1, 7) => (show, season, rng.gen_range(1..22)),
                    (Shows::SG1, 8)
                    | (Shows::SG1, 9)
                    | (Shows::SG1, 10)
                    | (Shows::Atlantis, 2)
                    | (Shows::Atlantis, 3)
                    | (Shows::Atlantis, 4)
                    | (Shows::Atlantis, 5)
                    | (Shows::Universe, _) => (show, season, rng.gen_range(1..20)),
                    (Shows::Atlantis, 1) => (show, season, rng.gen_range(1..19)),
                    _ => (Shows::SG1, 0, 0),
                };
                self.current_episode = episode;
                console::ConsoleService::log(
                    format!(
                        "Show: {}, Season: {}, episode: {}",
                        episode.0, episode.1, episode.2
                    )
                    .as_str(),
                );

                let request = Request::get(format!(
                    "https://api.themoviedb.org/3/tv/{}/season/{}/episode/{}?api_key={}&language=en-US",
                    match episode.0 {
                        Shows::SG1 => "4629",
                        Shows::Atlantis => "2290",
                        Shows::Universe => "5148",
                    },
                    episode.1,
                    episode.2,
                    key
                ))
                .body(Nothing)
                .expect("Could not build request.");

                let callback = self.link.callback(
                    |response: Response<Json<Result<Episode, anyhow::Error>>>| {
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
                        self.episode = Some(episode);
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
                <div style="flex: 1;">
                <h1 class="title" style="color: white; padding-top: 10px">
            { "Stargate Randomizer" }
            </h1>

                <nav class="level">
                <div class="level-item has-text-centered" style="margin: 10px;">
            <label class="checkbox">
                    <input type="checkbox"
                    onclick= self.link.callback(|_| Msg::Toggle(Shows::SG1))
            checked= self.sg1                        />
        {"SG-1"}
            </label>
                </div>

                <div class="level-item has-text-centered" style="margin: 10px;">
            <label class="checkbox">
                    <input type="checkbox"
                    onclick=self.link.callback(|_| Msg::Toggle(Shows::Atlantis))
            checked= self.atlantis
            />
        {"Atlantis"}
            </label>
                </div>

                <div class="level-item has-text-centered" style="margin: 10px;">
            <label class="checkbox">
                    <input type="checkbox"
                    onclick= self.link.callback(|_| Msg::Toggle(Shows::Universe))
            checked= self.universe
            />
        {"Universe"}
            </label>
                </div>
                </nav>
                </div>


            { self.view_fetching() }
            { self.view_episode() }
            { self.view_error() }

                    <div style="flex: 1;">
                    <button onclick=self.link.callback(|_| Msg::GetRandom)>{ "Jaunt through the orifice" }</button>
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
