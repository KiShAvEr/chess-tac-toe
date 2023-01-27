use std::time::Duration;

use dioxus::{prelude::*, html::style};
use dioxus_desktop::{tao::window::Window, use_window};
use dioxus_router::Link;
use helpers::{chesstactoe::{Color, SubscribeBoardRequest, chess::EndResult}, TicTacToe};
use utils::set_uuid;
use dioxus_free_icons::{Icon, icons::io_icons::IoArrowBack};

use crate::components::ChessBoard::ChessBoard;

pub fn TicBoard(cx: Scope) -> Element {

  let side = use_state(&cx, || Color::White as i32);
  let set_side = side.setter();

  let board = use_state(&cx, || None);
  let set_board = board.setter();
  
  let last_move = use_state(&cx, || "".to_owned());
  let set_last_move = last_move.setter();

  let next = use_state(&cx, || Color::White as i32);
  let set_next = next.setter();

  let a = use_future(&cx, (), |_| async move {
    let mut client = utils::get_client().unwrap();

    let mut res = client.subscribe_board(SubscribeBoardRequest { uuid: utils::get_uuid().unwrap() }).await.unwrap().into_inner();


    while let Ok(Some(msg)) = res.message().await {
      set_board(Some(TicTacToe::from(msg.game.as_ref().unwrap().clone())));
      set_side(msg.color);
      set_last_move(msg.game.as_ref().unwrap().last_move.clone());
      set_next(msg.game.as_ref().unwrap().next.clone());
    }
  });

  let selected_board = use_state(cx,  || None::<usize>);

  println!("{:?}", selected_board);

  // use_window(&cx).webview.open_devtools();

  match board.get() {
    Some(board) => match selected_board.get() {
        Some(board_num) => cx.render(rsx!{
          div {
            class: "board-view",
            div {
              onclick: |_| {selected_board.set(None)},
              class: "back-icon",
              Icon {
                width: 50,
                height: 50,
                icon: IoArrowBack,
              },
            }
            ChessBoard {last: Color::from_i32(1-**next).unwrap(), side: Color::from_i32(**side).unwrap(), chess: &board.chesses[board_num/3][board_num%3], board_num: (*board_num).try_into().unwrap(), last_move: (*last_move).to_string()}
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
                    let class = format!("tic-cell {} {}", if (col+row)%2 == 1 {"light"} else {"dark"}, if &board.chesses[col][row].end != &EndResult::None(true) {"over"} else {""});
                    rsx!{
                      div {
                        class: "{class}",
                        ChessBoard {last: Color::from_i32(1-**next).unwrap(), onclick: move |_| {println!("he?"); selected_board.set(Some(col*3+row))}, side: Color::from_i32(**side).unwrap(), chess: &board.chesses[col][row], board_num: (col*3+row).try_into().unwrap(), last_move: (*last_move).to_string()}
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