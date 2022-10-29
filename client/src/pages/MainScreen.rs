use dioxus::prelude::*;

use crate::components::{TicBoard::TicBoard, ChessBoard::ChessBoard};

pub fn MainScreen(cx: Scope) -> Element {

  let style = "* {
    margin: 0px;
  }
  
  body {
    height: 100vmin;
    width: 100vmin;
  }
  
  html {
    height: 100%;
    //overflow: hidden;
  }";

  cx.render(rsx!(
    div {
      style {"{style}"}
      TicBoard {}
    }
  ))
  
}