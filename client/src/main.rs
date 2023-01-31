#![allow(non_snake_case)]

use std::{future::Future, sync::Arc, time::Duration};

use dioxus::prelude::*;
use dioxus_router::{Route, Router};
use helpers::chesstactoe::{game_client::GameClient, JoinRequest};
use pages::MainScreen::MainScreen;
use tokio::sync::Mutex;
use tonic::{Request, Status};
use utils::set_uuid;

mod components;
mod pages;

const url: &str = "http://localhost:50051";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dioxus_desktop::launch(app);
  Ok(())
}

fn app(cx: Scope) -> Element {
  let client = use_future(cx, (), |_| async move { GameClient::connect(url).await });

  match client.value() {
    Some(client) => match client {
      Ok(client) => {
        cx.provide_context(Arc::new(Mutex::new(client.clone())));

        cx.render(rsx! {
            Router {
                Route { to: "/", MainScreen {}},
            }
        })
      }
      Err(err) => cx.render(rsx!("Server not found (probably)")),
    },
    None => cx.render(rsx!("Still connecting")),
  }
}
