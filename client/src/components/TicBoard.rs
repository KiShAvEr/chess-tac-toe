use dioxus::prelude::*;

use crate::components::ChessBoard::ChessBoard;

pub fn TicBoard(cx: Scope) -> Element {

  let style = include_str!("./styles/tic.css");

  cx.render(rsx!{
    style { "{style}" }
    div {
      class: "tic-container",
      (0..3).map(|a| {
        rsx!{
          div {
            class: "tic-col",
            (0..3).map(|b| {
              rsx!{
                div {
                  class: "tic-cell",
                  ChessBoard {}
                }
              }
            })
          }
        }
      })
    }
  })
}