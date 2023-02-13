use std::sync::Arc;

use dioxus::prelude::*;
use helpers::chesstactoe::{game_client::GameClient, JoinRequest};
use include_dir::{include_dir, File};
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request};

use crate::components::TicBoard::TicBoard;

pub fn MainScreen(cx: Scope) -> Element {
  let style = include_dir!("$CARGO_MANIFEST_DIR/src/components/styles/")
    .files()
    .fold("".to_owned(), |last: String, next: &File| {
      last + next.contents_utf8().unwrap()
    });

  let client = cx.use_hook(|| cx.consume_context::<Arc<Mutex<GameClient<Channel>>>>());

  match client.is_none() {
    true => cx.render(rsx!(
      div {
        "Something went to shit"
      }
    )),
    false => {
      let client = client.clone().unwrap();

      let future = use_future(cx, (), |_| async move {
        let mut client = client
          .lock()
          .await
          .join(Request::new(JoinRequest {}))
          .await
          .unwrap()
          .into_inner();

        while let Some(cli) = client.message().await.unwrap() {
          utils::set_uuid(&cli.uuid)
        }
      });

      match future.value() {
        Some(()) => {
          cx.render(rsx!(
            div {
              class: "main-container",
              style {"{style}"}
              TicBoard {}
              // ChessBoard {side: helpers::chesstactoe::Color::White}
            }
          ))
        }
        None => cx.render(rsx!("Waiting for opponent")),
      }
    }
  }
}
