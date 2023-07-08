use std::fmt::Display;

use crate::{FenError, chesstactoe};
use crate::{chesstactoe::Color, Coordinates, MoveError};

use crate::chess::ChessBoard;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TicTacToe {
  pub chesses: [[ChessBoard; 3]; 3],
  pub next: Color,
}

#[derive(Debug)]
pub enum TicError {
  InvalidCoords,
  WrongColor,
}

impl Display for TicError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl std::error::Error for TicError {}

impl TicTacToe {
  pub fn validate_move(
    &self,
    board: Coordinates,
    alg: &str,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    self.get_board(board)?.validate_move(alg, self.next)
  }

  pub fn make_move(
    &mut self,
    board: Coordinates,
    alg: &str,
  ) -> Result<(), Box<dyn std::error::Error>> {
    if !self.validate_move(board, alg)? {
      return Err(Box::new(MoveError::InvalidMove));
    }

    self.chesses[board.col][board.row].make_move(alg, self.next)?;

    self.next = if self.next == Color::White {
      Color::Black
    } else {
      Color::White
    };

    Ok(())
  }

  pub fn to_fen(&self) -> Result<String, Box<dyn std::error::Error>> {
    let mut res = "".to_owned();

    for boards in &self.chesses {
      for board in boards {
        res.push_str(&board.to_fen(self.next)?);
        res.push('\\');
      }
    }

    res = res[0..res.len() - 1].to_owned();

    res += &("+".to_owned()
      + if (self.next == Color::White) {
        "w"
      } else {
        "b"
      });

    Ok(res)
  }

  pub fn from_fen(input: &str) -> Result<Self, FenError> {
    let mut parts = input.split('+');

    if (parts.clone().count() != 2) {
      return Err(FenError::InvalidFormat);
    }

    let boards_part = parts.next().unwrap();

    let mut boards = boards_part.split('\\');

    if (boards.clone().count() != 9) {
      return Err(FenError::InvalidFormat);
    }

    let mut chesses = vec![];

    for i in 0..3 {
      chesses.push([
        ChessBoard::default(),
        ChessBoard::default(),
        ChessBoard::default(),
      ]);
      for j in 0..3 {
        chesses[i][j] = ChessBoard::parse_fen(boards.next().unwrap())?
      }
    }

    let chesses: [[ChessBoard; 3]; 3] = chesses.try_into().unwrap();

    let next = match parts.next().unwrap() {
      "w" => Color::White,
      "b" => Color::Black,
      _ => return Err(FenError::InvalidFormat),
    };

    Ok(TicTacToe { chesses, next })
  }

  pub fn get_board(&self, board: Coordinates) -> Result<&ChessBoard, TicError> {
    if (board.col > 2 || board.row > 2) {
      return Err(TicError::InvalidCoords);
    }

    Ok(&self.chesses[board.col][board.row])
  }
}

impl Default for TicTacToe {
  fn default() -> Self {
    TicTacToe {
      chesses: [
        [
          ChessBoard::default(),
          ChessBoard::default(),
          ChessBoard::default(),
        ],
        [
          ChessBoard::default(),
          ChessBoard::default(),
          ChessBoard::default(),
        ],
        [
          ChessBoard::default(),
          ChessBoard::default(),
          ChessBoard::default(),
        ],
      ],
      next: Color::White,
    }
  }
}

impl From<chesstactoe::TicTacToe> for TicTacToe {
  fn from(value: chesstactoe::TicTacToe) -> TicTacToe {
    let mut chesses = TicTacToe::default().chesses;

    for i in 0..3 {}

    for (i, row) in chesses.iter_mut().enumerate() {
      for (j, chess) in row.iter_mut().enumerate().take(3) {
        *chess = ChessBoard::parse_fen(&value.chesses[i * 3 + j].fen).unwrap()
      }
    }

    TicTacToe {
      chesses,
      next: value.next(),
    }
  }
}