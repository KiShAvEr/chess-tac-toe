use std::{collections::HashMap, any::Any};

use dioxus::prelude::*;
use helpers::{ChessBoard, PieceName, chesstactoe::{Color, MovePieceRequest}, Piece};
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use utils::get_client;

static DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/assets");

async fn on_piece_click(ev: MouseEvent, square: (usize, usize), selected: &UseState<Option<(usize, usize)>>, chess: &ChessBoard, board_num: u32, side: Color) {
    if selected.is_none() {
        selected.modify(|_| Some(square))
    }
    on_click(ev, square, selected, chess, board_num, side).await;
}

async fn on_click(ev: MouseEvent, square: (usize, usize), selected: &UseState<Option<(usize, usize)>>, chess: &ChessBoard, board_num: u32, side: Color) {

    if selected.is_none() {
        return
    }

    let attacked_piece = chess.board[square.0][square.1];
    let selected_piece = chess.board[selected.unwrap().0][selected.unwrap().1];
    let x = match attacked_piece {
        Some(_) => "x",
        None => ""
    };

    let piece_name = match selected_piece.unwrap().name {
        PieceName::ROOK => "R",
        PieceName::KNIGHT => "N",
        PieceName::BISHOP => "B",
        PieceName::QUEEN => "Q",
        PieceName::KING => "K",
        PieceName::PAWN => "",
    };

    let end_tile = ChessBoard::get_tile(square).unwrap();
    let start_tile = ChessBoard::get_tile(selected.unwrap()).unwrap();

    println!("{piece_name}{start_tile}{x}{end_tile}");

    let mut alg = format!("{piece_name}{start_tile}{x}{end_tile}");

    if piece_name == "K" && ((side == Color::White && (start_tile == "e1" && (end_tile == "g1" || end_tile == "h1"))) || (side == Color::Black && (start_tile == "e8" && (end_tile == "g8" || end_tile == "h8")))) {
        alg = "O-O".to_owned()
    } else if piece_name == "K" && ((side == Color::White && (start_tile == "e1" && (end_tile == "c1" || end_tile == "a1"))) || (side == Color::Black && (start_tile == "e8" && (end_tile == "c8" || end_tile == "a8")))) {
        alg = "O-O-O".to_owned()
    }

    
    if piece_name == "" &&  ((side == Color::White && square.0 == 7) || (side == Color::Black && square.0 == 0)) {
        alg += "K"
    }
    
    println!("{:?}, {:?}, {alg}", side, square);

    let is_valid = chess.validate_move(&alg, side);

    if is_valid.is_ok() && is_valid.unwrap() {
        let res = get_client().unwrap().move_piece(MovePieceRequest { board: board_num, alg, uuid: utils::get_uuid().unwrap() }).await;
    }


    selected.modify(|_| None)

}


static VALID_MOVES: Lazy<HashMap<String, Vec<(isize, isize)>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    let rook_moves = (-8..=8).map(|val| vec![(0, val), (val, 0)]).filter(|el| !el.contains(&(0, 0))).reduce(|rest, next| [rest, next].concat()).unwrap();
    
    let bishop_moves = (1..=8).map(|val| vec![(val, -val), (-val, val), (val, val), (-val, -val)]).reduce(|rest, next| [rest, next].concat()).unwrap();
    
    let pawn_moves = vec![(1, 0), (-1, 0), (2, 0), (-2, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    
    let knight_moves = vec![(2, 1), (-2, 1), (-2, -1), (2, -1), (1, 2), (-1, 2), (-1, -2), (1, -2)];
    
    let king_moves = vec![(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)];
    
    map.insert("R".into(), rook_moves.clone());
    map.insert("N".into(), knight_moves);
    map.insert("B".into(), bishop_moves.clone());
    map.insert("Q".into(), [rook_moves, bishop_moves].concat());
    map.insert("K".into(), king_moves);
    map.insert("P".into(), pawn_moves);
    
    map
});

#[derive(Props)]
pub struct ChessProps<'a> {
    side: Color,
    chess: &'a ChessBoard,
    board_num: u32,
    onclick: Option<EventHandler<'a, MouseEvent>>,
    last_move: String,
    last: Color
}

pub fn ChessBoard<'a>(cx: Scope<'a, ChessProps>) -> Element<'a> {

    let mut board = cx.props.chess.board;

    let mut images: HashMap<String, String> = HashMap::new();

    let mut split_move = cx.props.last_move.split(" ");

    let (last_board, last_move) = (split_move.next(), split_move.next());

    DIR.files().for_each(|file| {
        images.insert(file.path().file_name().unwrap().to_str().unwrap()[..file.path().file_name().unwrap().len()-4].to_string(),
        format!("data:image/svg+xml;base64, {}", base64::encode(file.contents())));
    });

    if cx.props.side == Color::White {
        board.reverse();
    }
    else {
        for i in 0..board.len() {
            board[i].reverse();
        }
    }

    let selected = use_state(&cx, || None::<(usize, usize)>);

    let piece_name = if selected.is_some() {
        let selected = selected.get().unwrap();

        let (col, row) = match cx.props.side {
            Color::White => (7-selected.0, selected.1),
            Color::Black => (selected.0, 7-selected.1)
        };

        match board[col][row] {
            Some(piece) => match piece.name {
                PieceName::ROOK => "R",
                PieceName::KNIGHT => "N",
                PieceName::BISHOP => "B",
                PieceName::QUEEN => "Q",
                PieceName::KING => "K",
                PieceName::PAWN => "",
            },
            None => "",
        }
    } else {
        ""
    };

    cx.render(rsx!{
        div { 
            class: "chess-container",
            board.iter().enumerate().map(|(col_idx, col)| {
                let map = images.clone();
                rsx!(
                    div {
                        class: "chess-col",
                        col.iter().enumerate().map(|(cell_idx, cell)| {
                            let piece = match cell {
                                Some(piece) => format!("{}{}", piece.color, piece.name),
                                None => "".to_owned()
                            };
                            let real_col = if cx.props.side == Color::White {7-col_idx} else {col_idx};
                            let real_row = if cx.props.side == Color::White {cell_idx} else {7-cell_idx};
                            let valid_move = if selected.is_some() && VALID_MOVES.get(if piece_name == "" {"P"} else {piece_name}).unwrap().iter().any(|la_move| {
                                let square = selected.unwrap();
                                square.0.saturating_add_signed(la_move.0) == real_col && square.1.saturating_add_signed(la_move.1) == real_row
                            }) {
                                let end_tile = ChessBoard::get_tile((real_col, real_row)).unwrap();

                                let starting_tile = ChessBoard::get_tile(selected.unwrap()).unwrap();

                                let x = if cell.is_some() {
                                    "x"
                                } else {
                                    ""
                                };
                            
                                let move_str = format!("{piece_name}{starting_tile}{x}{end_tile}");

                                let valid = cx.props.chess.validate_move(&move_str, cx.props.side);

                                match valid {
                                    Ok(true) => "valid",
                                    _ => ""
                                }

                            } else {
                                ""
                            };

                            let is_last_board = match last_board {
                                Some(board) => match board.parse::<u32>() {
                                    Ok(num) => num == cx.props.board_num,
                                    Err(_) => false
                                },
                                None => false
                            };

                            let is_last_move = match last_move {
                                Some(move_str) => {
                        
                                    let (start, end, _piece) = if move_str == "O-O" || move_str == "O-O-O" {
                                        match (move_str, cx.props.last ) {
                                            ("O-O", Color::White) => (ChessBoard::get_square("e1").unwrap(), ChessBoard::get_square("g1").unwrap(), PieceName::KING),
                                            ("O-O", Color::Black) => (ChessBoard::get_square("e8").unwrap(), ChessBoard::get_square("g8").unwrap(), PieceName::KING),
                                            ("O-O-O", Color::White) => (ChessBoard::get_square("e1").unwrap(), ChessBoard::get_square("c1").unwrap(), PieceName::KING),
                                            ("O-O-O", Color::Black) => (ChessBoard::get_square("e8").unwrap(), ChessBoard::get_square("c8").unwrap(), PieceName::KING),
                                            _ => unreachable!()
                                        }
                                    } else {
                                        ChessBoard::get_data_from_move(move_str).unwrap()
                                    };
                        
                                    (real_col == start.0 && real_row == start.1) || (real_col == end.0 && real_row == end.1)
                                },
                                None => false
                            };

                            if is_last_board {
                                println!("{:?}", last_move);
                            }

                            let class = format!("chess-cell {} {} {} {}", 
                                if (col_idx+cell_idx)%2==0 {"light"} else {"dark"}, 
                                if selected.is_some() && selected.unwrap().0 == real_col && selected.unwrap().1 == real_row {"selected"} else {""},
                                valid_move,
                                if is_last_board && is_last_move {"last"} else {""}
                            );
                            if piece != "" {
                                let src = map.get(&piece).unwrap().to_owned();
                                return rsx!(
                                    div {
                                        class: "{class}",
                                        onclick: move |ev| { if let Some(onclick) = cx.props.onclick.as_ref() {
                                            onclick.call(ev.clone());
                                            }; let selected: UseState<Option<(usize, usize)>> = selected.clone();let chess = cx.props.chess.clone();let board_num = cx.props.board_num.clone();let side = cx.props.side.clone(); cx.spawn(async move {on_piece_click(ev, (real_col, real_row), &selected, &chess, board_num, side).await})},
                                        img {
                                            src: "{src}",
                                            height: "100%",
                                            width: "100%",
                                        },
                                    }
                                )
                            }
                            else {
                                return rsx!(
                                    div {
                                        class: "{class}", 
                                        onclick: move |ev| {let selected: UseState<Option<(usize, usize)>> = selected.clone();let chess = cx.props.chess.clone();let board_num = cx.props.board_num.clone();let side = cx.props.side.clone(); if let Some(onclick) = cx.props.onclick.as_ref() {
                                            onclick.call(ev.clone());
                                        }; cx.spawn(async move {on_click(ev, (real_col, real_row), &selected, &chess, board_num, side).await})},
                                    }
                                )
                            }
                        })
                    }
                )
            })
        }
    })
}