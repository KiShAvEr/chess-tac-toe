use std::{collections::HashMap, sync::Arc};

use base64::Engine;
use dioxus::prelude::*;
use futures::stream::StreamExt;
use helpers::{
  chess::{ChessBoard, PieceName},
  chesstactoe::{game_client::GameClient, Color, MovePieceRequest},
  Coordinates,
};
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tonic::transport::Channel;

fn on_piece_click(
  ev: MouseEvent,
  square: (usize, usize),
  selected: &UseState<Option<(usize, usize)>>,
  chess: &ChessBoard,
  side: Color,
  ct: &Coroutine<String>,
  promotion_data: PromotionData,
) {
  if selected.is_none() {
    selected.modify(|_| Some(square))
  }
  on_click(ev, square, selected, chess, side, ct, promotion_data);
}

struct PromotionData<'a> {
  promotin: &'a UseState<bool>,
  promotion_square: &'a UseState<Option<(usize, usize)>>,
}

fn on_click(
  _ev: MouseEvent,
  square: (usize, usize),
  selected: &UseState<Option<(usize, usize)>>,
  chess: &ChessBoard,
  side: Color,
  ct: &Coroutine<String>,
  PromotionData {
    promotin,
    promotion_square,
  }: PromotionData,
) {
  if selected.is_none() {
    return;
  }
  promotin.set(false);

  let square = Coordinates::new(square);

  let attacked_piece = chess.board[square.row][square.col];
  let selected_piece = chess.board[selected.unwrap().0][selected.unwrap().1];
  let x = match attacked_piece {
    Some(_) => "x",
    None if selected_piece.unwrap().name == PieceName::PAWN && chess.en_passant == Some(square) => {
      "x"
    }
    None => "",
  };

  let piece_name = match selected_piece.unwrap().name {
    PieceName::ROOK => "R",
    PieceName::KNIGHT => "N",
    PieceName::BISHOP => "B",
    PieceName::QUEEN => "Q",
    PieceName::KING => "K",
    PieceName::PAWN => "",
  };

  let selected_coords = Coordinates::new(selected.unwrap());

  let end_tile = ChessBoard::get_tile(square).unwrap();
  let start_tile = ChessBoard::get_tile(selected_coords).unwrap();

  eprintln!("{piece_name}{start_tile}{x}{end_tile}");
  eprintln!("{:?}", chess.en_passant);

  let mut alg = format!("{piece_name}{start_tile}{x}{end_tile}");

  if piece_name == "K"
    && ((side == Color::White && (start_tile == "e1" && (end_tile == "g1" || end_tile == "h1")))
      || (side == Color::Black && (start_tile == "e8" && (end_tile == "g8" || end_tile == "h8"))))
  {
    alg = "O-O".to_owned()
  } else if piece_name == "K"
    && ((side == Color::White && (start_tile == "e1" && (end_tile == "c1" || end_tile == "a1")))
      || (side == Color::Black && (start_tile == "e8" && (end_tile == "c8" || end_tile == "a8"))))
  {
    alg = "O-O-O".to_owned()
  }

  if piece_name.is_empty()
    && ((side == Color::White && square.row == 7) || (side == Color::Black && square.row == 0))
  {
    promotin.set(true);
    promotion_square.set(Some((square.row, square.col)));
    return;
  }

  // eprintln!("{:?}, {:?}, {alg}", side, square);

  let is_valid = chess.validate_move(&alg, side);

  if is_valid.is_ok() && is_valid.unwrap() {
    unsafe {
      static mut COUNT: usize = 0;
      COUNT += 1;
      println!("{COUNT}");
    }

    ct.send(alg);
  }

  selected.modify(|_| None)
}

fn promote(
  selected: &UseState<Option<(usize, usize)>>,
  chess: &ChessBoard,
  ct: &Coroutine<String>,
  PromotionData {
    promotin,
    promotion_square,
  }: PromotionData,
  target_piece: &str,
) {
  if promotion_square.is_none() {
    panic!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
  }
  let promotion_square_uw = promotion_square.get().unwrap();
  let attacked_piece = chess.board[promotion_square_uw.0][promotion_square_uw.1];
  let selected_piece = chess.board[selected.unwrap().0][selected.unwrap().1];
  let x = match attacked_piece {
    Some(_) => "x",
    None => "",
  };

  let piece_name = match selected_piece.unwrap().name {
    PieceName::PAWN => "",
    _ => panic!("AAAAAAAAAouch"),
  };

  let target_name = match target_piece {
    "WhiteKnight" | "BlackKnight" => "N",
    "WhiteBishop" | "BlackBishop" => "B",
    "WhiteRook" | "BlackRook" => "R",
    "WhiteQueen" | "BlackQueen" => "Q",
    "WhiteKing" | "BlackKing" => "K",
    _ => panic!("mifaszstas {}", target_piece),
  };

  let selected_coords = Coordinates::new(selected.unwrap());

  let end_tile = ChessBoard::get_tile(Coordinates {
    row: promotion_square_uw.0,
    col: promotion_square_uw.1,
  })
  .unwrap();
  let start_tile = ChessBoard::get_tile(selected_coords).unwrap();

  eprintln!("{piece_name}{start_tile}{x}{end_tile}{target_name}");

  let alg = format!("{piece_name}{start_tile}{x}{end_tile}{target_name}");

  ct.send(alg);

  promotin.set(false);

  promotion_square.set(None);
}

static VALID_MOVES: Lazy<HashMap<String, Vec<(isize, isize)>>> = Lazy::new(|| {
  let mut map = HashMap::new();

  let rook_moves = (-8..=8)
    .map(|val| vec![(0, val), (val, 0)])
    .filter(|el| !el.contains(&(0, 0)))
    .reduce(|rest, next| [rest, next].concat())
    .unwrap();

  let bishop_moves = (1..=8)
    .map(|val| vec![(val, -val), (-val, val), (val, val), (-val, -val)])
    .reduce(|rest, next| [rest, next].concat())
    .unwrap();

  let pawn_moves = vec![
    (1, 0),
    (-1, 0),
    (2, 0),
    (-2, 0),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
  ];

  let knight_moves = vec![
    (2, 1),
    (-2, 1),
    (-2, -1),
    (2, -1),
    (1, 2),
    (-1, 2),
    (-1, -2),
    (1, -2),
  ];

  let king_moves = vec![
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
  ];

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
  pub side: Color,
  pub chess: &'a ChessBoard,
  pub board_num: u32,
  pub onclick: Option<EventHandler<'a, MouseEvent>>,
  pub last_move: String,
  pub last: Color,
}

pub fn ChessBoard<'a>(cx: Scope<'a, ChessProps>) -> Element<'a> {
  let mut board = cx.props.chess.board;

  static IMAGES: Lazy<Arc<HashMap<String, String>>> = Lazy::new(|| {
    static DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/assets/pieces");

    let mut inmages = HashMap::new();

    DIR.files().for_each(|file| {
      inmages.insert(
        file.path().file_name().unwrap().to_str().unwrap()
          [..file.path().file_name().unwrap().len() - 4]
          .to_string(),
        format!(
          "data:image/svg+xml;base64, {}",
          base64::engine::general_purpose::STANDARD.encode(file.contents())
        ),
      );
    });

    Arc::new(inmages)
  });

  let mut split_move = cx.props.last_move.split(' ');

  let (last_board, last_move) = (split_move.next(), split_move.next());

  let client = cx
    .use_hook(|| cx.consume_context::<Arc<Mutex<GameClient<Channel>>>>())
    .clone()
    .unwrap();

  if cx.props.side == Color::White {
    board.reverse();
  } else {
    for row in &mut board {
      row.reverse();
    }
  }

  let selected = use_state(cx, || None::<(usize, usize)>);

  let promotin = use_state(cx, || false);

  let piece_name = if selected.is_some() {
    let selected = selected.get().unwrap();

    let (col, row) = match cx.props.side {
      Color::White => (7 - selected.0, selected.1),
      Color::Black => (selected.0, 7 - selected.1),
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

  let board_num = cx.props.board_num;

  let ct: &Coroutine<String> = use_coroutine(cx, |mut rx: UnboundedReceiver<String>| async move {
    tokio::spawn(async move {
      while let Some(alg) = rx.next().await {
        let res = client
          .lock()
          .await
          .move_piece(MovePieceRequest {
            board: board_num,
            alg,
            uuid: utils::get_uuid().unwrap(),
          })
          .await;

        println!("{res:?}");
      }
    })
    .await
    .unwrap();
  });

  let pre = match cx.props.side {
    Color::White => "White",
    Color::Black => "Black",
  };

  let valid_promotions =
    ["Knight", "Bishop", "Rook", "Queen", "King"].map(|piece| pre.to_owned() + piece);

  let promotion_square = use_state(cx, || None::<(usize, usize)>);

  cx.render(rsx!{
    div { class: "chess-container", onclick: |_| promotin.set(false),
        match promotin.get() {
          true => rsx!{dialog {class: "promotin-dialog" , open: *promotin.get(), rsx!{
            valid_promotions.iter().map(|name| {
              let src = IMAGES.get(name).unwrap();
              let name = name.clone();
              rsx!(
                img {
                  class: "piece-image",
                  src: "{src}",
                  onclick: move |_| promote(selected, cx.props.chess, ct, PromotionData{ promotion_square, promotin }, &name),
                }
              )
            })
          }}},
          false => rsx!(""),
        },
        board.iter().enumerate().map(|(row_idx, row)| {
            let map = IMAGES.clone();
            rsx!(
                div {
                    class: "chess-row",
                    row.iter().enumerate().map(|(cell_idx, cell)| {
                        let piece = match cell {
                            Some(piece) => format!("{}{}", piece.color, piece.name),
                            None => "".to_owned()
                        };
                        let real_row = if cx.props.side == Color::White {7-row_idx} else {row_idx};
                        let real_col = if cx.props.side == Color::White {cell_idx} else {7-cell_idx};
                        let valid_move = if selected.is_some() && VALID_MOVES.get(if piece_name.is_empty() {"P"} else {piece_name}).unwrap().iter().any(|la_move| {
                            let square = selected.unwrap();
                            square.0.saturating_add_signed(la_move.0) == real_row && square.1.saturating_add_signed(la_move.1) == real_col
                        }) {
                            let end_tile = ChessBoard::get_tile(Coordinates::new((real_row, real_col))).unwrap();

                            let starting_tile = ChessBoard::get_tile(Coordinates::new(selected.unwrap())).unwrap();

                            let x = if cell.is_some() || piece_name.is_empty() && cx.props.chess.en_passant == Some(Coordinates::new((real_row, real_col))) {
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

                                (real_row == start.row && real_col == start.col) || (real_row == end.row && real_col == end.col)
                            },
                            None => false
                        };

                        let class = format!("chess-cell {} {} {} {}", 
                            if (row_idx+cell_idx)%2==0 {"light"} else {"dark"}, 
                            if selected.is_some() && selected.unwrap().0 == real_row && selected.unwrap().1 == real_col {"selected"} else {""},
                            valid_move,
                            if is_last_board && is_last_move {"last"} else {""}
                        );
                        if !piece.is_empty() {
                            let src = map.get(&piece).unwrap().to_owned();
                            return rsx!(
                                div {
                                    class: "{class}",
                                    onclick: move |ev| { if let Some(onclick) = cx.props.onclick.as_ref() {
                                        onclick.call(ev.clone());
                                        }; on_piece_click(ev, (real_row, real_col), selected, cx.props.chess, cx.props.side, ct, PromotionData { promotin, promotion_square })},
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
                                    onclick: move |ev| {if let Some(onclick) = cx.props.onclick.as_ref() {
                                        onclick.call(ev.clone());
                                    }; on_click(ev, (real_row, real_col), selected, cx.props.chess, cx.props.side, ct, PromotionData { promotin, promotion_square })},
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
