use std::{collections::HashMap, fmt::format};

use dioxus::prelude::*;
use helpers::{ChessBoard, Piece};
use include_dir::{include_dir, Dir};

static DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/assets");

pub fn ChessBoard(cx: Scope) -> Element {

    let mut board = ChessBoard::random().board;

    let mut images: HashMap<String, String> = HashMap::new();

    DIR.files().for_each(|file| {
        images.insert(file.path().file_name().unwrap().to_str().unwrap()[..file.path().file_name().unwrap().len()-4].to_string(),
        format!("data:image/svg+xml;base64, {}", base64::encode(file.contents())));
    });

    board.reverse();

    let style = include_str!("./styles/chess.css"); 

    cx.render(rsx!{
        style { "{style}" }
        div { class: "chess-container",
            board.map(|col| {
                let map = images.clone();
                rsx!(
                    div {
                        class: "chess-col",
                        col.map(|cell| {
                            let piece = match cell {
                                Some(piece) => format!("{}{}", piece.color, piece.name),
                                None => "".to_owned()
                            };
                            let map = map.clone();
                            if(piece != "") {
                                let src = map.get(&piece).unwrap().to_owned();
                                return rsx!(
                                    div {
                                        class: "chess-cell",
                                        img {
                                            src: "{src}",
                                            height: "100%",
                                            width: "100%",
                                        }
                                    }
                                )
                            }
                            else {
                                return rsx!(div {class: "chess-cell", ""})
                            }
                        })
                    }
                )
            })
        }
    })
}