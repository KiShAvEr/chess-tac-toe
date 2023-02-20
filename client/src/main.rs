#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_router::{Route, Router};
use helpers::chesstactoe::game_client::GameClient;
use include_dir::{include_dir, File};
use pages::{GameScreen::GameScreen, MainScreen::MainScreen};
use tokio::sync::Mutex;

mod components;
mod pages;

const URL: &str = "http://localhost:50051";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dioxus_desktop::launch(app);
  Ok(())
}

fn app(cx: Scope) -> Element {
  let client = use_future(cx, (), |_| async move { GameClient::connect(URL).await });

  let style = include_dir!("$CARGO_MANIFEST_DIR/src/components/styles/")
    .files()
    .fold("".to_owned(), |last: String, next: &File| {
      last + next.contents_utf8().unwrap()
    });

  match client.value() {
    Some(client) => match client {
      Ok(client) => {
        cx.provide_context(Arc::new(Mutex::new(client.clone())));

        cx.render(rsx! {
            style { "{style}" }
            Router {
                Route { to: "/", MainScreen {} },
                Route { to: "/game", GameScreen {} }
                Route { to: "/game/:id", GameScreen { } }
            }
        })
      }
      Err(_err) => cx.render(rsx!("Server not found (probably)")),
    },
    None => cx.render(rsx!("Still connecting")),
  }
}
