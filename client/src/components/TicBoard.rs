use std::time::Duration;

use dioxus::{prelude::*, html::style};
use dioxus_router::Link;
use helpers::{chesstactoe::{Color, SubscribeBoardRequest}, TicTacToe};
use utils::set_uuid;
use dioxus_free_icons::{Icon, icons::io_icons::IoArrowBack};

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

  let selected_board = use_state(cx,  || None::<usize>);

  println!("{:?}", selected_board);

  match board.get() {
    Some(board) => match selected_board.get() {
        Some(board_num) => cx.render(rsx!{
          div {
            class: "board-view",
            div {
              onclick: |_| {selected_board.set(None)},
              class: "back-icon",
              Icon {
                icon: IoArrowBack,
              },
            }
            ChessBoard {side: Color::from_i32(**side).unwrap(), chess: &board.chesses[board_num/3][board_num%3], board_num: (*board_num).try_into().unwrap()}
          }
        }),
        None => cx.render(rsx!{
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
                        ChessBoard {onclick: move |_| {println!("he?"); selected_board.set(Some(col*3+row))}, side: Color::from_i32(**side).unwrap(), chess: &board.chesses[col][row], board_num: (col*3+row).try_into().unwrap()}
                      }
                    }
                  })
                }
              }
            })
          }
        }),
    } ,
    None => cx.render(rsx!{
      div {
        "Loading shit"
      }
    }),
  }
}