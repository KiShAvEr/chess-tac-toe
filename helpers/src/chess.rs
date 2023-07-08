use std::{collections::HashMap, fmt::Display};

use rand::seq::SliceRandom;

use crate::{chesstactoe::chess::EndResult, Color, Coordinates, FenError, MoveError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
  pub color: Color,
  pub name: PieceName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceName {
  ROOK,
  KNIGHT,
  BISHOP,
  QUEEN,
  KING,
  PAWN,
}

impl Display for PieceName {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PieceName::ROOK => write!(f, "Rook"),
      PieceName::KNIGHT => write!(f, "Knight"),
      PieceName::BISHOP => write!(f, "Bishop"),
      PieceName::QUEEN => write!(f, "Queen"),
      PieceName::KING => write!(f, "King"),
      PieceName::PAWN => write!(f, "Pawn"),
    }
  }
}

#[derive(Debug)]
pub enum ChessError {
  FenError(FenError),
  MoveError(MoveError),
}

impl Display for ChessError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl std::error::Error for ChessError {}

macro_rules! regex {
  ($re:literal $(,)?) => {{
    static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| regex::Regex::new($re).unwrap())
  }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessBoard {
  pub board: [[Option<Piece>; 8]; 8],
  castling: HashMap<(Color, Castling), bool>,
  pub en_passant: Option<Coordinates>,
  halfmove: usize,
  fullmove: usize,
  pub end: EndResult,
  past: Vec<String>,
}

impl Eq for EndResult {}

impl Default for ChessBoard {
  fn default() -> Self {
    Self::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
  }
}

impl TryFrom<&str> for ChessBoard {
  type Error = FenError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    ChessBoard::parse_fen(value)
  }
}

impl ChessBoard {
  pub fn random() -> Self {
    let mut board = "".to_owned();
    let samples = vec![
      "r", "n", "b", "q", "k", "1", "p", "P", "R", "N", "B", "Q", "K",
    ];
    for _i in 0..8 {
      let rand = samples
        .choose_multiple(&mut rand::thread_rng(), 8)
        .map(|el| el.to_owned().to_string())
        .reduce(|this, next| this + &next)
        .unwrap();
      board += &(rand + "/")
    }
    board = board[..board.len() - 1].to_string();

    Self::parse_fen(&format!("{board} w KQkq - 0 1")).unwrap()
  }

  pub fn parse_fen(fen: &str) -> Result<ChessBoard, FenError> {
    let mut parts = fen.split(' ');
    if parts.clone().count() != 6 {
      return Err(FenError::InvalidFormat);
    }

    let board_part = parts.next().unwrap();

    let board_parts = board_part.split('/');

    if board_parts.clone().count() != 8 {
      return Err(FenError::InvalidFormat);
    }

    let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

    for (index, part) in board_parts.enumerate() {
      let mut row = vec![];
      for j in part.chars() {
        if j.is_ascii_digit() {
          row.extend(vec![None; j.to_digit(10).unwrap() as usize]);
        } else {
          match &*j.to_lowercase().to_string() {
            "r" => row.push(Some(Piece {
              name: PieceName::ROOK,
              color: if j.is_uppercase() {
                Color::White
              } else {
                Color::Black
              },
            })),
            "n" => row.push(Some(Piece {
              name: PieceName::KNIGHT,
              color: if j.is_uppercase() {
                Color::White
              } else {
                Color::Black
              },
            })),
            "b" => row.push(Some(Piece {
              name: PieceName::BISHOP,
              color: if j.is_uppercase() {
                Color::White
              } else {
                Color::Black
              },
            })),
            "k" => row.push(Some(Piece {
              name: PieceName::KING,
              color: if j.is_uppercase() {
                Color::White
              } else {
                Color::Black
              },
            })),
            "q" => row.push(Some(Piece {
              name: PieceName::QUEEN,
              color: if j.is_uppercase() {
                Color::White
              } else {
                Color::Black
              },
            })),
            "p" => row.push(Some(Piece {
              name: PieceName::PAWN,
              color: if j.is_uppercase() {
                Color::White
              } else {
                Color::Black
              },
            })),
            _ => return Err(FenError::InvalidFormat),
          }
        }
      }

      if row.len() != 8 {
        return Err(FenError::InvalidFormat);
      }

      board[index] = row.try_into().unwrap();
    }

    let _next = match parts.next().unwrap() {
      "w" => Color::White,
      "b" => Color::Black,
      _ => return Err(FenError::InvalidFormat),
    };

    let castle_part = parts.next().unwrap();

    let mut castling: HashMap<(Color, Castling), bool> = HashMap::from([
      ((Color::Black, Castling::Kingside), false),
      ((Color::Black, Castling::Queenside), false),
      ((Color::White, Castling::Kingside), false),
      ((Color::White, Castling::Queenside), false),
    ]);

    if castle_part.contains('K') {
      castling.insert((Color::White, Castling::Kingside), true);
    }
    if castle_part.contains('Q') {
      castling.insert((Color::White, Castling::Queenside), true);
    }
    if castle_part.contains('k') {
      castling.insert((Color::Black, Castling::Kingside), true);
    }
    if castle_part.contains('q') {
      castling.insert((Color::Black, Castling::Queenside), true);
    }

    let en_passant = parts.next().unwrap();

    let halfmove = parts.next().unwrap();

    let fullmove = parts.next().unwrap();

    board.reverse();

    let end = if !board.into_iter().any(|a| {
      a.contains(&Some(Piece {
        name: PieceName::KING,
        color: Color::Black,
      }))
    }) {
      EndResult::Color(Color::White as i32)
    } else if !board.into_iter().any(|a| {
      a.contains(&Some(Piece {
        name: PieceName::KING,
        color: Color::White,
      }))
    }) {
      EndResult::Color(Color::Black as i32)
    } else {
      EndResult::None(true)
    };

    Ok(ChessBoard {
      board,
      end,
      castling,
      en_passant: if en_passant == "-" {
        None
      } else {
        Some(ChessBoard::get_square(en_passant)?)
      },
      halfmove: halfmove
        .parse::<usize>()
        .map_err(|_| FenError::InvalidFormat)?,
      fullmove: fullmove
        .parse::<usize>()
        .map_err(|_| FenError::InvalidFormat)?,
      past: vec![fen.to_owned()],
    })
  }

  pub fn get_square(str: &str) -> Result<Coordinates, FenError> {
    let regex = regex!("[a-h]{1}[1-8]{1}");
    if !regex.is_match(str) {
      return Err(FenError::InvalidFormat);
    }

    let col = match &str[0..1] {
      "a" => 0,
      "b" => 1,
      "c" => 2,
      "d" => 3,
      "e" => 4,
      "f" => 5,
      "g" => 6,
      "h" => 7,
      _ => return Err(FenError::InvalidSquare),
    };

    let row = str[1..2].parse::<usize>().unwrap() - 1;

    Ok(Coordinates { row, col })
  }

  pub fn get_tile(square: Coordinates) -> Result<String, FenError> {
    let chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

    if square.row > 7 || square.col > 7 {
      return Err(FenError::InvalidSquare);
    }

    Ok(chars[square.col].to_string() + &(square.row + 1).to_string())
  }

  pub fn get_data_from_move(
    move_string: &str,
  ) -> Result<(Coordinates, Coordinates, PieceName), FenError> {
    let name = match &move_string[0..1] {
      "N" => PieceName::KNIGHT,
      "R" => PieceName::ROOK,
      "B" => PieceName::BISHOP,
      "Q" => PieceName::QUEEN,
      "K" => PieceName::KING,
      _ => PieceName::PAWN,
    };

    let starting_sqare = &move_string
      [if name == PieceName::PAWN { 0 } else { 1 }..if name == PieceName::PAWN { 2 } else { 3 }];
    let starting_coords = ChessBoard::get_square(starting_sqare)?;

    let regex = regex!("[a-h]{1}[1-8]{1}");

    let end_square = &regex.captures_iter(move_string).last().unwrap()[0];
    let end_coords = ChessBoard::get_square(end_square)?;

    Ok((starting_coords, end_coords, name))
  }

  //TODO test
  pub fn validate_move(
    &self,
    move_string: &str,
    next: Color,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    let regex = regex!(
      r"^((O-O-O)|(O-O)|([RNBKQ]{0,1}([a-h]{1}[1-8]{1})x{0,1}([a-h]{1}[1-8]{1}))|(([a-h]{1}[1-8]{1})x{0,1}([a-h]{1}[1-8]{1})[RNBKQ]{0,1}))$"
    );

    if !regex.is_match(move_string) {
      return Err(Box::new(FenError::InvalidFormat));
    }

    if self.end != EndResult::None(true) {
      return Err(Box::new(MoveError::GameOver));
    }

    if move_string == "O-O-O" || move_string == "O-O" {
      let (rook_square, king_square, between_squares) = match (move_string, next) {
        ("O-O-O", Color::Black) => (
          ChessBoard::get_square("a8")?,
          ChessBoard::get_square("e8")?,
          vec![
            ChessBoard::get_square("b8")?,
            ChessBoard::get_square("c8")?,
            ChessBoard::get_square("d8")?,
          ],
        ),
        ("O-O-O", Color::White) => (
          ChessBoard::get_square("a1")?,
          ChessBoard::get_square("e1")?,
          vec![
            ChessBoard::get_square("b1")?,
            ChessBoard::get_square("c1")?,
            ChessBoard::get_square("d1")?,
          ],
        ),
        ("O-O", Color::Black) => (
          ChessBoard::get_square("h8")?,
          ChessBoard::get_square("e8")?,
          vec![ChessBoard::get_square("f8")?, ChessBoard::get_square("g8")?],
        ),
        ("O-O", Color::White) => (
          ChessBoard::get_square("h1")?,
          ChessBoard::get_square("e1")?,
          vec![ChessBoard::get_square("f1")?, ChessBoard::get_square("g1")?],
        ),
        _ => return Err(Box::new(MoveError::InvalidMove)),
      };

      if (self.board[rook_square.row][rook_square.col]
        == Some(Piece {
          color: next,
          name: PieceName::ROOK,
        })
        && self.board[king_square.row][king_square.col]
          == Some(Piece {
            color: next,
            name: PieceName::KING,
          }))
        && (between_squares
          .iter()
          .all(|square| self.board[square.row][square.col].is_none()))
        && (!self.is_checked(&next)
          && between_squares.iter().all(|square| {
            let mut fake_board = self.clone();
            let coords = king_square;
            fake_board.board[coords.row][coords.col] = None;

            fake_board.board[square.row][square.col] = Some(Piece {
              color: next,
              name: PieceName::KING,
            });

            !fake_board.is_checked(&next)
          }))
      {
        return Ok(true);
      }
      return Ok(false);
    }

    let (starting_coords, end_coords, piece_name) = ChessBoard::get_data_from_move(move_string)?;

    let piece = Piece {
      color: next,
      name: piece_name,
    };

    if !move_string.contains('x') {
      if self.board[end_coords.row][end_coords.col].is_some() {
        return Ok(false);
      }
    } else {
      match self.board[end_coords.row][end_coords.col] {
        Some(piece) if piece.color == next => return Ok(false),
        None if piece.name != PieceName::PAWN => return Ok(false),
        _ => {}
      }
    }

    if self.board[starting_coords.row][starting_coords.col].is_none()
      || self.board[starting_coords.row][starting_coords.col].unwrap() != piece
    {
      return Ok(false);
    }

    match piece.name {
      PieceName::ROOK => {
        return Ok(ChessBoard::validate_rook(
          &starting_coords,
          &end_coords,
          &self.board,
        ));
      }
      PieceName::KNIGHT => {
        return Ok(
          starting_coords.row.abs_diff(end_coords.row) == 1
            && starting_coords.col.abs_diff(end_coords.col) == 2
            || (starting_coords.row.abs_diff(end_coords.row) == 2
              && starting_coords.col.abs_diff(end_coords.col) == 1),
        )
      }
      PieceName::BISHOP => {
        return Ok(ChessBoard::validate_bishop(
          &starting_coords,
          &end_coords,
          &self.board,
        ))
      }
      PieceName::QUEEN => {
        return Ok(
          ChessBoard::validate_rook(&starting_coords, &end_coords, &self.board)
            || ChessBoard::validate_bishop(&starting_coords, &end_coords, &self.board),
        )
      }
      PieceName::KING => {
        let diff = starting_coords.row.abs_diff(end_coords.row) == 1
          || starting_coords.col.abs_diff(end_coords.col) == 1;
        if diff {
          return Ok(
            ChessBoard::validate_bishop(&starting_coords, &end_coords, &self.board)
              || ChessBoard::validate_rook(&starting_coords, &end_coords, &self.board),
          );
        }
        return Ok(false);
      }
      PieceName::PAWN => {
        let double = match next {
          Color::Black => starting_coords.row == 6 && end_coords.row == 4,
          Color::White => starting_coords.row == 1 && end_coords.row == 3,
        };
        let single = match next {
          Color::Black => starting_coords.row.saturating_sub(end_coords.row) == 1,
          Color::White => end_coords.row.saturating_sub(starting_coords.row) == 1,
        };
        let en_passant = match next {
          Color::Black => self.en_passant == Some(end_coords),
          Color::White => self.en_passant == Some(end_coords),
        };

        let mut valid = false;

        if move_string.contains('x') {
          let diagonal_move = match next {
            Color::Black => {
              starting_coords.row.saturating_sub(end_coords.row) == 1
                && starting_coords.col.abs_diff(end_coords.col) == 1
            }
            Color::White => {
              end_coords.row.saturating_sub(starting_coords.row) == 1
                && starting_coords.col.abs_diff(end_coords.col) == 1
            }
          };

          valid =
            diagonal_move && (self.board[end_coords.row][end_coords.col].is_some() || en_passant);
        } else if single && starting_coords.col == end_coords.col {
          valid = self.board[end_coords.row][end_coords.col].is_none();
        } else if double && starting_coords.col == end_coords.col {
          valid = self.board[end_coords.row][end_coords.col].is_none()
            && ((next == Color::Black && self.board[5][starting_coords.col].is_none())
              || (next == Color::White && self.board[2][starting_coords.col].is_none()));
        }

        if end_coords.row == 0 || end_coords.row == 7 {
          let endings = ["K", "R", "B", "N", "Q"];
          if endings.contains(&move_string.chars().last().unwrap().to_string().as_str()) {
            return Ok(valid);
          }
          return Ok(false);
        }

        return Ok(valid);
      }
    }
  }

  fn _validate_check(
    &self,
    _starting_coords: &Coordinates,
    _end_coords: &Coordinates,
    _board: &[[Option<Piece>; 8]; 8],
    alg: &str,
    next: Color,
  ) -> bool {
    let mut fake_board = self.clone();
    fake_board.make_move(alg, next).unwrap();
    !fake_board.is_checked(&next)
  }

  fn is_checked(&self, color: &Color) -> bool {
    let king_position = self
      .board
      .into_iter()
      .position(|col| {
        col
          .into_iter()
          .any(|p| p.is_some() && p.unwrap().color == *color && p.unwrap().name == PieceName::KING)
      })
      .unwrap();

    let king_position = (
      king_position,
      self.board[king_position]
        .into_iter()
        .position(|p| {
          p.is_some() && p.unwrap().color == *color && p.unwrap().name == PieceName::KING
        })
        .unwrap(),
    );

    let king_position = Coordinates {
      row: king_position.0,
      col: king_position.1,
    };

    fn check_the_diagonals(
      board: &[[Option<Piece>; 8]; 8],
      color: &Color,
      king_position: Coordinates,
    ) -> bool {
      let directions = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
      for &(dx, dy) in &directions {
        let mut x = king_position.row as i8 + dx;
        let mut y = king_position.col as i8 + dy;
        while (0..8).contains(&x) && (0..8).contains(&y) {
          let piece = board[x as usize][y as usize];
          if let Some(other_piece) = piece {
            if other_piece.color != *color
              && (other_piece.name == PieceName::BISHOP
                || other_piece.name == PieceName::QUEEN
                || dx == 1
                  && dy == 1
                  && other_piece.name == PieceName::PAWN
                  && *color == Color::White
                || dx == -1
                  && dy == 1
                  && other_piece.name == PieceName::PAWN
                  && *color == Color::Black)
            {
              return false;
            }
            break;
          }
          x += dx;
          y += dy;
        }
      }
      true
    }

    fn check_the_lateral(
      board: &[[Option<Piece>; 8]; 8],
      color: &Color,
      king_position: Coordinates,
    ) -> bool {
      let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
      for &(x, y) in directions.iter() {
        let mut piece: Option<Piece> = None;
        let mut pos = king_position;
        loop {
          match piece {
            Some(other_piece) => {
              if other_piece.color != *color
                && (other_piece.name == PieceName::ROOK || other_piece.name == PieceName::QUEEN)
              {
                return false;
              }
              break;
            }
            None => {
              if pos.row as i8 + x < 0
                || pos.row as i8 + x > 7
                || pos.col as i8 + y < 0
                || pos.col as i8 + y > 7
              {
                break;
              }
              pos.row = (pos.row as i8 + x) as usize;
              pos.col = (pos.col as i8 + y) as usize;
              piece = board[pos.row][pos.col]
            }
          }
        }
      }
      true
    }

    fn check_the_knights(
      board: &[[Option<Piece>; 8]; 8],
      color: &Color,
      king_position: Coordinates,
    ) -> bool {
      // A list of tuples representing the positions that a knight
      // could attack from its current position
      let moves = [
        (2, 1),
        (2, -1),
        (-2, 1),
        (-2, -1),
        (1, 2),
        (1, -2),
        (-1, 2),
        (-1, -2),
      ];

      // Iterate through the possible positions and check if any of them
      // contain a knight of the opposite color
      for (row_offset, col_offset) in moves.iter() {
        let row = (king_position.row as isize) + row_offset;
        let col = (king_position.col as isize) + col_offset;

        // Make sure the position is within the bounds of the board
        if !(0..=7).contains(&row) || !(0..=7).contains(&col) {
          continue;
        }

        let piece = board[row as usize][col as usize];
        if piece.is_some()
          && piece.unwrap().name == PieceName::KNIGHT
          && piece.unwrap().color != *color
        {
          return false;
        }
      }

      true
    }

    let diag = check_the_diagonals(&self.board, color, king_position);

    let lat = check_the_lateral(&self.board, color, king_position);

    let paci = check_the_knights(&self.board, color, king_position);

    !diag || !lat || !paci
  }

  fn validate_bishop(
    starting_coords: &Coordinates,
    end_coords: &Coordinates,
    board: &[[Option<Piece>; 8]; 8],
  ) -> bool {
    let row_diff = starting_coords.row as i32 - end_coords.row as i32;
    let col_diff = starting_coords.col as i32 - end_coords.col as i32;

    if row_diff.abs() != col_diff.abs()
      || starting_coords.row == end_coords.row
      || starting_coords.col == end_coords.col
    {
      return false;
    }

    let (row_dir, col_dir) = if row_diff > 0 && col_diff > 0 {
      (-1, -1)
    } else if row_diff > 0 && col_diff < 0 {
      (-1, 1)
    } else if row_diff < 0 && col_diff > 0 {
      (1, -1)
    } else {
      (1, 1)
    };

    let mut row = starting_coords.row as i32 + row_dir;
    let mut col = starting_coords.col as i32 + col_dir;
    while row != end_coords.row as i32 && col != end_coords.col as i32 {
      if board[row as usize][col as usize].is_some() {
        return false;
      }
      row += row_dir;
      col += col_dir;
    }
    true
  }

  fn validate_rook(
    starting_coords: &Coordinates,
    end_coords: &Coordinates,
    board: &[[Option<Piece>; 8]; 8],
  ) -> bool {
    if starting_coords.row == end_coords.row || starting_coords.col == end_coords.col {
      let (min, max) = if starting_coords.row == end_coords.row {
        (
          std::cmp::min(starting_coords.col, end_coords.col),
          std::cmp::max(starting_coords.col, end_coords.col),
        )
      } else {
        (
          std::cmp::min(starting_coords.row, end_coords.row),
          std::cmp::max(starting_coords.row, end_coords.row),
        )
      };
      for i in min + 1..max {
        if starting_coords.row == end_coords.row {
          if board[starting_coords.row][i].is_some() {
            return false;
          }
        } else if board[i][starting_coords.col].is_some() {
          return false;
        }
      }
      return true;
    }
    false
  }

  pub fn to_fen(&self, next: Color) -> Result<String, FenError> {
    let mut output = "".to_owned();

    for row in self.board.into_iter().rev() {
      let mut counter: usize = 0;
      for piece in row.iter() {
        match piece {
          Some(piece) => {
            if counter != 0 {
              output += &counter.to_string();
            }
            counter = 0;
            match piece.name {
              PieceName::ROOK => {
                output += if piece.color == Color::White {
                  "R"
                } else {
                  "r"
                }
              }
              PieceName::KNIGHT => {
                output += if piece.color == Color::White {
                  "N"
                } else {
                  "n"
                }
              }
              PieceName::BISHOP => {
                output += if piece.color == Color::White {
                  "B"
                } else {
                  "b"
                }
              }
              PieceName::QUEEN => {
                output += if piece.color == Color::White {
                  "Q"
                } else {
                  "q"
                }
              }
              PieceName::KING => {
                output += if piece.color == Color::White {
                  "K"
                } else {
                  "k"
                }
              }
              PieceName::PAWN => {
                output += if piece.color == Color::White {
                  "P"
                } else {
                  "p"
                }
              }
            }
          }
          None => {
            counter += 1;
          }
        }
      }

      if counter != 0 {
        output += &counter.to_string();
      }
      output += "/";
    }

    output = output[0..output.len() - 1].to_owned();

    output += " ";
    output += if next == Color::White { "w " } else { "b " };

    let mut has_castling = false;
    for &(color, side) in &[
      (Color::White, Castling::Kingside),
      (Color::White, Castling::Queenside),
      (Color::Black, Castling::Kingside),
      (Color::Black, Castling::Queenside),
    ] {
      if *self.castling.get(&(color, side)).unwrap() {
        has_castling = true;
        let letter = match side {
          Castling::Kingside => 'K',
          Castling::Queenside => 'Q',
        };
        output += &if color == Color::White {
          letter
        } else {
          letter.to_ascii_lowercase()
        }
        .to_string();
      }
    }
    if !has_castling {
      output += "-";
    }

    output += " ";

    match self.en_passant {
      Some(tile) => {
        output += &Self::get_tile(tile)?;
      }
      None => output += "-",
    }

    output += " ";

    output += &self.halfmove.to_string();

    output += " ";

    output += &self.fullmove.to_string();

    Ok(output)
  }

  pub fn make_move(&mut self, alg: &str, next: Color) -> Result<(), Box<dyn std::error::Error>> {
    if !self.validate_move(alg, next)? {
      return Err(Box::new(ChessError::MoveError(MoveError::InvalidMove)));
    }

    if alg == "O-O" || alg == "O-O-O" {
      let (king_old, king_new, rook_old, rook_new) = match (alg, next) {
        ("O-O", Color::White) => (
          ChessBoard::get_square("e1").unwrap(),
          ChessBoard::get_square("g1").unwrap(),
          ChessBoard::get_square("h1").unwrap(),
          ChessBoard::get_square("f1").unwrap(),
        ),
        ("O-O", Color::Black) => (
          ChessBoard::get_square("e8").unwrap(),
          ChessBoard::get_square("g8").unwrap(),
          ChessBoard::get_square("h8").unwrap(),
          ChessBoard::get_square("f8").unwrap(),
        ),
        ("O-O-O", Color::White) => (
          ChessBoard::get_square("e1").unwrap(),
          ChessBoard::get_square("c1").unwrap(),
          ChessBoard::get_square("a1").unwrap(),
          ChessBoard::get_square("d1").unwrap(),
        ),
        ("O-O-O", Color::Black) => (
          ChessBoard::get_square("e8").unwrap(),
          ChessBoard::get_square("c8").unwrap(),
          ChessBoard::get_square("a8").unwrap(),
          ChessBoard::get_square("d8").unwrap(),
        ),
        _ => return Err(Box::new(MoveError::InvalidMove)),
      };

      self.board[king_old.row][king_old.col] = None;
      self.board[king_new.row][king_new.col] = Some(Piece {
        color: next,
        name: PieceName::KING,
      });
      self.board[rook_old.row][rook_old.col] = None;
      self.board[rook_new.row][rook_new.col] = Some(Piece {
        color: next,
        name: PieceName::ROOK,
      });

      self.halfmove += 1;

      self.castling.insert((next, Castling::Kingside), false);
      self.castling.insert((next, Castling::Queenside), false);

      if next == Color::Black {
        self.fullmove += 1;
      }

      self.en_passant = None;

      return Ok(());
    }

    let (starting_coords, end_coords, piece_name) =
      Self::get_data_from_move(alg).map_err(ChessError::FenError)?;

    let piece = Piece {
      color: next,
      name: piece_name,
    };

    let starting_square = Self::get_tile(starting_coords)?;

    let opposite = if next == Color::White {
      Color::Black
    } else {
      Color::White
    };

    if self.board[end_coords.row][end_coords.col].is_some()
      && !self.board.iter().any(|row| {
        row.iter().any(|cell| {
          cell.is_some() && cell.unwrap().name == PieceName::KING && cell.unwrap().color == opposite
        })
      })
    {
      self.end = EndResult::Color(next as i32);
    }

    let en_passant = match next {
      Color::Black => self.en_passant == Some(end_coords),
      Color::White => self.en_passant == Some(end_coords),
    };

    if piece.name == PieceName::PAWN && en_passant {
      let dir: isize = match next {
        Color::Black => 1,
        Color::White => -1,
      };

      let _faszom = self.board[end_coords.row.checked_add_signed(dir).unwrap()][end_coords.col];

      self.board[end_coords.row.checked_add_signed(dir).unwrap()][end_coords.col] = None
    }
    self.board[starting_coords.row][starting_coords.col] = None;

    self.board[end_coords.row][end_coords.col] = Some(piece);

    if piece.name == PieceName::KING {
      self.castling.insert((next, Castling::Kingside), false);
      self.castling.insert((next, Castling::Queenside), false);
    } else if piece.name == PieceName::ROOK {
      match next {
        Color::Black => {
          if starting_square == "a8" {
            self
              .castling
              .insert((Color::Black, Castling::Queenside), false);
          } else if starting_square == "h8" {
            self
              .castling
              .insert((Color::Black, Castling::Kingside), false);
          }
        }
        Color::White => {
          if starting_square == "a1" {
            self
              .castling
              .insert((Color::White, Castling::Queenside), false);
          } else if starting_square == "h1" {
            self
              .castling
              .insert((Color::White, Castling::Kingside), false);
          }
        }
      }
    }
    if piece.name == PieceName::PAWN && (starting_coords.row.abs_diff(end_coords.row) == 2) {
      self.en_passant = Some(Coordinates {
        row: (starting_coords.row + end_coords.row) / 2,
        col: starting_coords.col,
      });
    } else {
      self.en_passant = None;
    }

    if piece.name == PieceName::PAWN || alg.contains('x') {
      self.halfmove = 0;
    } else {
      self.halfmove += 1;
    }

    let endings = ["Q", "K", "R", "N", "B"];

    if piece.name == PieceName::PAWN
      && endings.contains(&alg.chars().last().unwrap().to_string().as_str())
    {
      let promoted_name = match alg.chars().last().unwrap().to_string().as_str() {
        "Q" => PieceName::QUEEN,
        "K" => PieceName::KING,
        "R" => PieceName::ROOK,
        "N" => PieceName::KNIGHT,
        "B" => PieceName::BISHOP,
        _ => panic!("BRUHHHHH"),
      };
      self.board[end_coords.row][end_coords.col] = Some(Piece {
        name: promoted_name,
        color: piece.color,
      });
    }

    if next == Color::Black {
      self.fullmove += 1;
    }

    let binding = self.to_fen(next).unwrap();
    let curr_fen = binding.split(' ').next().unwrap();

    let count = self
      .past
      .iter()
      .filter(|fen| {
        let checked = fen.split(' ').next().unwrap();
        checked == curr_fen
      })
      .count();

    if count >= 2 {
      self.end = EndResult::Draw(true)
    }

    self.past.push(binding);

    Ok(())
  }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Castling {
  Queenside,
  Kingside,
}
