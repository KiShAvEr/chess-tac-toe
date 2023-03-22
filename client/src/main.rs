#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_desktop::{
  tao::menu::{MenuBar, MenuItem},
  use_window, Config, WindowBuilder,
};
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
  // dioxus_desktop::launch(app);
  let mut menu = MenuBar::new();

  let mut edit = MenuBar::new();
  edit.add_native_item(MenuItem::Paste).unwrap();
  edit.add_native_item(MenuItem::Copy).unwrap();
  edit.add_native_item(MenuItem::Cut).unwrap();

  menu.add_submenu("Edit", true, edit);

  let config = Config::default().with_window(
    WindowBuilder::default()
      .with_title("Chess-Tac-Toe")
      .with_menu(menu),
  );

  dioxus_desktop::launch_cfg(app, config);
  Ok(())
}

fn app(cx: Scope) -> Element {
  let client = use_future(cx, (), |_| async move { GameClient::connect(URL).await });

  use_window(cx).hide_menu();

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
                Route { to: "/", MainScreen {} }
                Route { to: "/game", GameScreen {} }
                Route { to: "/game/new", GameScreen {} }
                Route { to: "/game/:id", GameScreen {} }
            }
        })
      }
      Err(_err) => cx.render(rsx!("Server not found (probably)")),
    },
    None => cx.render(rsx!("Still connecting")),
  }
}
