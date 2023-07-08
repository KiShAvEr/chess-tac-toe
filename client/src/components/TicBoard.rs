use std::sync::Arc;

use base64::Engine;
use dioxus::prelude::*;
use dioxus_free_icons::{icons::io_icons::IoArrowBack, Icon};

use helpers::{
  chesstactoe::{chess::EndResult, game_client::GameClient, Color, SubscribeBoardRequest},
  tictactoe::TicTacToe,
};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::components::ChessBoard::ChessBoard;

pub fn TicBoard(cx: Scope) -> Element {
  let client = cx.use_hook(|| cx.consume_context::<Arc<Mutex<GameClient<Channel>>>>());

  let side = use_state(cx, || Color::White as i32);
  let set_side = side.setter();

  let board = use_state(cx, || None);
  let set_board = board.setter();

  let last_move = use_state(cx, || "".to_owned());
  let set_last_move = last_move.setter();

  let next = use_state(cx, || Color::White as i32);
  let set_next = next.setter();

  let client = client.clone();

  use_future(cx, (), |_| async move {
    if client.is_some() {
      let mut res = client
        .unwrap()
        .lock()
        .await
        .subscribe_board(SubscribeBoardRequest {
          uuid: utils::get_uuid().unwrap(),
        })
        .await
        .unwrap()
        .into_inner();

      while let Ok(Some(msg)) = res.message().await {
        // println!("{msg:?}");

        set_board(Some(TicTacToe::from(msg.game.as_ref().unwrap().clone())));
        set_side(msg.color);
        set_last_move(msg.game.as_ref().unwrap().last_move.clone());
        set_next(msg.game.as_ref().unwrap().next);
      }
    }
  });

  let selected_board = use_state(cx, || None::<usize>);

  static O: Lazy<String> = Lazy::new(|| {
    base64::engine::general_purpose::STANDARD.encode(include_bytes!("../assets/Blue_O.svg"))
  });
  static X: Lazy<String> = Lazy::new(|| {
    base64::engine::general_purpose::STANDARD.encode(include_bytes!("../assets/Red_X.svg"))
  });

  match board.get() {
    Some(board) => match selected_board.get() {
        Some(board_num) => cx.render(rsx!{
            div { class: "board-view",
                div { onclick: |_| { selected_board.set(None) }, class: "back-icon",
                    Icon { width: 50, height: 50, icon: IoArrowBack }
                }
                ChessBoard {
                    last: Color::from_i32(1 - **next).unwrap(),
                    side: Color::from_i32(**side).unwrap(),
                    chess: &board.chesses[board_num / 3][board_num % 3],
                    board_num: (*board_num).try_into().unwrap(),
                    last_move: (*last_move).to_string()
                }
            }
        }),
        None => cx.render(rsx!{
            div { class: "tic-container",
                (0..3).map(|col| {
              let o_src = O.clone();
              let x_src = X.clone();
              rsx!{
                div {
                  class: "tic-col",
                  (0..3).map(|row| {
                    let class = format!("tic-cell {}", if (col+row)%2 == 1 {"light"} else {"dark"});
                    let chess_board = rsx!(ChessBoard {last: Color::from_i32(1-**next).unwrap(), onclick: move |_| {selected_board.set(Some(col*3+row))}, side: Color::from_i32(**side).unwrap(), chess: &board.chesses[col][row], board_num: (col*3+row).try_into().unwrap(), last_move: (*last_move).to_string()});
                    let o_src = format!(
                      "data:image/svg+xml;base64, {}",
                      o_src.clone()
                    );
                    let x_src = format!(
                      "data:image/svg+xml;base64, {}",
                      x_src.clone()
                    );
                    let child = match &board.chesses[col][row].end {
                      EndResult::Color(color) => {
                        match Color::from_i32(*color).unwrap() {
                          Color::White => rsx!(img { class: "won_img", src: "{x_src}" }),
                          Color::Black => rsx!(img { class: "won_img", src: "{o_src}" })
                        }
                      },
                      EndResult::Draw(_) => rsx!(""),
                      EndResult::None(_) => rsx!(""),
                    };
                    rsx!{
                      div {
                        class: "{class}",
                        child,
                        chess_board
                      }
                    }
                  })
                }
              }
            })
            }
        }),
    } ,
    None => cx.render(rsx!{ div { "Loading shit" } }),
  }
}
