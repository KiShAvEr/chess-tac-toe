use dioxus::prelude::*;
use helpers::chesstactoe::{JoinRequest, JoinResponse};
use include_dir::{include_dir, File};
use tonic::{Request, Status, Response, Streaming};
use utils::get_client;


use crate::components::{TicBoard::TicBoard, ChessBoard::ChessBoard};

pub fn MainScreen(cx: Scope) -> Element {

  let mut style = "* {
    margin: 0px;
  }
  
  body {
    height: 100vmin;
    width: 100vmin;
  }
  
  html {
    height: 100%;
    //overflow: hidden;
  }".to_owned();

  let style_ext = include_dir!("$CARGO_MANIFEST_DIR/src/components/styles/").files().fold("".to_owned(), |last: String, next: &File| last + next.contents_utf8().unwrap());

  style += &style_ext;

  let future = use_future(&cx, (),|_| async move {
    let client = get_client();

    if client.is_none() {
        return false;
    }

    let mut client = client.unwrap().join(Request::new(JoinRequest {  })).await.unwrap().into_inner();

    while let Some(cli) =  client.message().await.unwrap() {
      utils::set_uuid(&cli.uuid)
    };

    return true
  });

  println!("{:?}", future.value());

  match future.value() {
    Some(true) => cx.render(rsx!(
      div {
        style {"{style}"}
        TicBoard {}
        // ChessBoard {side: helpers::chesstactoe::Color::White}
      }
    )),
    Some(false) => cx.render(rsx!(
      div {
        "Something went to shit"
      }
    )),
    None => cx.render(rsx!(
      div {
        "Loading shit in main"
      }
    )),
}
  
}