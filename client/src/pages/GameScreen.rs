use std::{sync::Arc, marker::PhantomData};

use dioxus::prelude::*;
use dioxus_router::{use_router, use_route};
use helpers::chesstactoe::{game_client::GameClient, JoinRequest, JoinLobbyRequest};
use include_dir::{include_dir, File};
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request};

use crate::components::TicBoard::TicBoard;

pub fn GameScreen(cx: Scope) -> Element {
  let client = cx.use_hook(|| cx.consume_context::<Arc<Mutex<GameClient<Channel>>>>());

  let lobby_code = use_route(cx).parse_segment_or_404::<String>("id");

  match client.is_none() {
    true => cx.render(rsx!(
      div {
        "Something went to shit"
      }
    )),
    false => {
      let client = client.clone().unwrap();

      let future = use_future(cx, (), |_| async move {
        
        let mut client = client.lock().await;

        println!("{lobby_code:?}");

        let cli = match lobby_code {
          Some(code) => client.join_lobby(JoinLobbyRequest { code: code.to_string() }).await,
          None => client.join(JoinRequest {}).await
        };

        drop(client);

        match cli {
          Ok(res) => {
            let mut res = res.into_inner();
            while let Some(response) = res.message().await.unwrap() {
              utils::set_uuid(&response.uuid)
            }
            return Ok(())
          },
          Err(er) => {
            return Err(er);
          }
        }

      });

      match future.value() {
        Some(res) => {
          match res {
            Ok(_) => cx.render(rsx!(
              div {
                class: "main-container",
                TicBoard {}
                // ChessBoard {side: helpers::chesstactoe::Color::White}
              }
            )),
            Err(_er) => cx.render(rsx!{
              div {
                "Invalid code"
                button {
                  onclick: |_ev| { use_router(cx).navigate_to("/") },
                  "Go back to main menu"
                }
              }
            })
          }
        }
        None => cx.render(rsx!("Waiting for opponent")),
      }
    }
  }
}
