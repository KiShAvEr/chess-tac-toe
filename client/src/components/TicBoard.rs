use std::time::Duration;

use dioxus::prelude::*;
use helpers::{chesstactoe::{Color, SubscribeBoardRequest}, TicTacToe};
use utils::set_uuid;

use crate::components::ChessBoard::ChessBoard;

pub fn TicBoard(cx: Scope) -> Element {

  let side = use_state(&cx, || Color::White as i32);
  let set_side = side.setter();

  let board = use_state(&cx, || None);
  let set_board = board.setter();

  let a = use_future(&cx, (), |_| async move {
    let mut client = utils::get_client().unwrap();

    let mut res = client.subscribe_board(SubscribeBoardRequest { uuid: utils::get_uuid().unwrap() }).await.unwrap().into_inner();


    while let Ok(Some(msg)) = res.message().await {
      set_board(Some(TicTacToe::from(msg.game.as_ref().unwrap().clone())));
      set_side(msg.color);
    }
  });

  match board.get() {
    Some(board) => cx.render(rsx!{
      div {
        class: "tic-container",
        (0..3).map(|col| {
          rsx!{
            div {
              class: "tic-col",
              (0..3).map(|row| {
                let class = format!("tic-cell {}", if (col+row)%2 == 1 {"light"} else {"dark"});
                rsx!{
                  div {
                    class: "{class}",
                    ChessBoard {side: Color::from_i32(**side).unwrap(), chess: &board.chesses[col][row], board_num: (col*3+row).try_into().unwrap()}
                  }
                }
              })
            }
          }
        })
      }
    }),
    None => cx.render(rsx!{
      div {
        "Loading shit"
      }
    }),
  }
}