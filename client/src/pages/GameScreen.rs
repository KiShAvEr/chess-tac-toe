use std::{sync::Arc};

use dioxus::prelude::*;
use dioxus_desktop::{tao::clipboard};
use dioxus_router::{use_router, use_route};
use helpers::chesstactoe::{game_client::GameClient, JoinRequest, JoinLobbyRequest, MakeLobbyRequest};

use tokio::sync::Mutex;
use tonic::{transport::Channel};

use crate::components::TicBoard::TicBoard;

pub fn GameScreen(cx: Scope) -> Element {
  let client = cx.use_hook(|| cx.consume_context::<Arc<Mutex<GameClient<Channel>>>>());

  let lobby_code = use_route(cx).parse_segment_or_404::<String>("id");

  let is_new_lobby = use_route(cx).last_segment() == Some("new");

  match client.is_none() {
    true => cx.render(rsx!(
      div {
        "Something went to shit"
      }
    )),
    false => {
      let client = client.clone().unwrap();

      let invite_code = use_state(cx, || None);
      let set_invite_code = invite_code.setter();

      let future = use_future(cx, (), |_| async move {
        
        let mut client = client.lock().await;

        println!("{lobby_code:?}");

        if is_new_lobby {
          let cli = client.make_lobby(MakeLobbyRequest {}).await;
          drop(client);

          match cli {
            Ok(res) => {
              let mut res = res.into_inner();
              while let Some(response) = res.message().await.unwrap() {
                utils::set_uuid(&response.join_response.unwrap().uuid);
                set_invite_code(Some(response.room_id))
              }
              Ok(())
            },
            Err(er) => {
              Err(er)
            }
          }

        } else {
          let cli = match &lobby_code {
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
              Ok(())
            },
            Err(er) => {
              Err(er)
            }
          }
        }


      });

      match future.value() {
        Some(res) => {
          match res {
            Ok(_) => cx.render(rsx!(
              div {
                class: "main-container",
                TicBoard {},
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
        None => cx.render(rsx!{"Waiting for opponent", if invite_code.is_some() {
          let invite_code = invite_code.as_ref().unwrap();
          rsx!(div {
            class: "lobby-code",
            "{invite_code}",
            button {
              onclick: move |_| { clipboard::Clipboard::new().write_text(invite_code); }, "Copy"
            }
          })
        },}),
      }
    }
  }
}
