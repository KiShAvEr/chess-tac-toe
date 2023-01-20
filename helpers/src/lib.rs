#![allow(unused)]

use std::{fmt::{Display, write}, collections::HashMap};
use rand::seq::SliceRandom;

use chesstactoe::{*, chess::EndResult};

pub mod chesstactoe {
  tonic::include_proto!("chesstactoe");
}

macro_rules! regex {
  ($re:literal $(,)?) => {{
      static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
      RE.get_or_init(|| regex::Regex::new($re).unwrap())
  }};
}

impl Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        Color::White => write!(f, "White"),
        Color::Black => write!(f, "Black"),
    }
  }
}

use regex::Regex;

#[derive(Debug)]
pub enum FenError {
  InvalidFormat,
  InvalidSquare
}

impl std::error::Error for FenError {}

impl Display for FenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Debug)]
pub enum MoveError {
  WrongColor,
  InvalidMove,
  GameOver
}

impl Display for MoveError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self)
  }
}

impl std::error::Error for MoveError {}

#[derive(Debug)]
pub enum ChessError {
  FenError(FenError),
  MoveError(MoveError)
}

impl Display for ChessError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self)
  }
}

impl std::error::Error for ChessError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TicTacToe {
  pub chesses: [[ChessBoard; 3]; 3],
  pub next: Color
}

#[derive(Debug)]
pub enum TicError {
  InvalidCoords,
  WrongColor
}

impl Display for TicError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self)
  }
}

impl std::error::Error for TicError {}

impl TicTacToe {
  pub fn validate_move(&self, board: (usize, usize), alg: &str) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(self.get_board(board)?.validate_move(alg, self.next)?)
  }

  pub fn make_move(&mut self, board: (usize, usize), alg: &str) -> Result<(), Box<dyn std::error::Error>> {

    if !self.validate_move(board, alg)? {
      return Err(Box::new(MoveError::InvalidMove))
    }

    self.chesses[board.0][board.1].make_move(alg, self.next)?;

    self.next = if self.next == Color::White {Color::Black} else {Color::White};

    Ok(())
  }

  pub fn to_fen(&self) -> Result<String, Box<dyn std::error::Error>> {
    let mut res = "".to_owned();

    for boards in &self.chesses {
      for board in boards {
        res.extend(board.to_fen(self.next)?.chars());
        res.extend("\\".chars());
      }
    }

    res = res[0..res.len()-1].to_owned();

    res += &("+".to_owned() + if(self.next == Color::White) {"w"} else {"b"});

    return Ok(res);
  }

  pub fn from_fen(input: &str) -> Result<Self, FenError> {

    let mut parts = input.split("+");

    if(parts.clone().count() != 2) {
      return Err(FenError::InvalidFormat);
    }

    let boards_part = parts.next().unwrap();

    let mut boards = boards_part.split(r#"\"#);
    
    if(boards.clone().count() != 9) {
      return  Err(FenError::InvalidFormat);
    }

    let mut chesses = vec![];

    for i in 0..3 {
      chesses.push([ChessBoard::default(), ChessBoard::default(), ChessBoard::default()]);
      for j in 0..3 {
        chesses[i][j] = ChessBoard::parse_fen(boards.next().unwrap())?
      }
    }

    let chesses: [[ChessBoard; 3]; 3] = chesses.try_into().unwrap();

    let next = match parts.next().unwrap() {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err(FenError::InvalidFormat)
    };

    Ok(TicTacToe {chesses, next})
  }

  pub fn get_board(&self, board: (usize, usize)) -> Result<&ChessBoard, TicError> {
    if(board.0 > 2 || board.1 > 2) {
      return Err(TicError::InvalidCoords);
    }
  
    Ok(&self.chesses[board.0][board.1])
  }

}

impl Default for TicTacToe {
  fn default() -> Self {
    TicTacToe { chesses: [
      [ChessBoard::default(), ChessBoard::default(), ChessBoard::default()], 
      [ChessBoard::default(), ChessBoard::default(), ChessBoard::default()],
      [ChessBoard::default(), ChessBoard::default(), ChessBoard::default()]
    ], next: Color::White }
  }
}

impl From<chesstactoe::TicTacToe> for TicTacToe {
  fn from(value: chesstactoe::TicTacToe) -> TicTacToe {
    let mut chesses = TicTacToe::default().chesses;

    for i in 0..3 {
      for j in 0..3 {
        chesses[i][j] = ChessBoard::parse_fen(&value.chesses[i*3+j].fen).unwrap()
      }
    }

    TicTacToe { chesses: chesses, next: value.next() }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessBoard {
  pub board: [[Option<Piece>; 8]; 8],
  castling: HashMap<(Color, Castling), bool>,
  en_passant: Option<(usize, usize)>,
  halfmove: usize,
  fullmove: usize, 
  pub end: EndResult,
  past: Vec<String>
}

impl Eq for EndResult {}

impl Default for ChessBoard {
  fn default() -> Self {
    return Self::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
  }
}

impl ChessBoard {

  pub fn random() -> Self {
    let mut board = "".to_owned();
    let samples = vec!["r", "n", "b", "q", "k", "1", "p", "P", "R", "N", "B", "Q", "K"];
    for i in (0..8) {
      let rand = samples.choose_multiple(&mut rand::thread_rng(), 8).map(|el| el.to_owned().to_string()).reduce(|this, next| {
        this + &next
      }).unwrap();
      board += &(rand + "/")
    }
    board = board[..board.len()-1].to_string();
    return Self::parse_fen(&format!("{} w KQkq - 0 1", board)).unwrap()
  }

  pub fn parse_fen(fen: &str) -> Result<ChessBoard, FenError> {
    let mut parts = fen.split(" ");
    if parts.clone().count() != 6 {
      return Err(FenError::InvalidFormat)
    }
  
    let board_part = parts.next().unwrap();
  
    let board_parts = board_part.split("/");
  
    if board_parts.clone().count() != 8 {
      return Err(FenError::InvalidFormat)
    }
  
    let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];
  
    for (index, part) in board_parts.enumerate() {
      let mut row = vec![];
      for j in part.chars() {
  
        if j.is_digit(10) {
          row.extend(vec![None; (j.to_digit(10).unwrap() as usize)]);
        }
        
        else {
          match &*j.clone().to_lowercase().to_string() {
            "r" => row.push(Some(Piece{ name: PieceName::ROOK, color: if j.is_uppercase() {Color::White} else {Color::Black}})),
            "n" => row.push(Some(Piece{ name: PieceName::KNIGHT, color: if j.is_uppercase() {Color::White} else {Color::Black}})),
            "b" => row.push(Some(Piece{ name: PieceName::BISHOP, color: if j.is_uppercase() {Color::White} else {Color::Black}})),
            "k" => row.push(Some(Piece{ name: PieceName::KING, color: if j.is_uppercase() {Color::White} else {Color::Black}})),
            "q" => row.push(Some(Piece{ name: PieceName::QUEEN, color: if j.is_uppercase() {Color::White} else {Color::Black}})),
            "p" => row.push(Some(Piece{ name: PieceName::PAWN, color: if j.is_uppercase() {Color::White} else {Color::Black}})),
            _ => return Err(FenError::InvalidFormat)
          }
        }
  
  
      }
  
      if(row.len() != 8) {
        return Err(FenError::InvalidFormat)
      }
  
      board[index] = row.try_into().unwrap();
    }
  
    let next = match parts.next().unwrap() {
      "w" => Color::White,
      "b" => Color::Black,
      _ => return Err(FenError::InvalidFormat)
    };
  
    let castle_part = parts.next().unwrap();
  
    let mut castling:HashMap<(Color, Castling), bool> = HashMap::from([
        ((Color::Black, Castling::KINGSIDE), false),
        ((Color::Black, Castling::QUEENSIDE), false),
        ((Color::White, Castling::KINGSIDE), false),
        ((Color::White, Castling::QUEENSIDE), false),
    ]);
  
    if(castle_part.contains("K")) {
      castling.insert((Color::White, Castling::KINGSIDE), true);
    }
    if(castle_part.contains("Q")) {
      castling.insert((Color::White, Castling::QUEENSIDE), true);
    }
    if(castle_part.contains("k")) {
      castling.insert((Color::Black, Castling::KINGSIDE), true);
    }
    if(castle_part.contains("q")) {
      castling.insert((Color::Black, Castling::QUEENSIDE), true);
    }
  
    let en_passant = parts.next().unwrap();
  
    let halfmove = parts.next().unwrap();
  
    let fullmove = parts.next().unwrap();
  
    board.reverse();
    
    let end = if(!board.into_iter().any(|a| a.contains(&Some(Piece {name: PieceName::KING, color: Color::Black})))) {
      EndResult::Color(Color::White as i32)
    } else if (!board.into_iter().any(|a| a.contains(&Some(Piece {name: PieceName::KING, color: Color::White})))) {
      EndResult::Color(Color::Black as i32)
    }
    else {
      EndResult::None(true)
    };
  
    Ok(ChessBoard {board, end, castling, 
      en_passant: if(en_passant == "-") {None} else {Some(ChessBoard::get_square(en_passant)?)}, 
      halfmove: halfmove.parse::<usize>().map_err(|_| FenError::InvalidFormat)?, 
      fullmove: fullmove.parse::<usize>().map_err(|_| FenError::InvalidFormat)?, past: vec![fen.to_owned()]}
    )
  
  }

  pub fn get_square(str: &str) -> Result<(usize, usize), FenError> {
    let regex = regex!("[a-h]{1}[1-8]{1}");
    if(!regex.is_match(str)) {
      return Err(FenError::InvalidFormat)
    }

    let column = match &str[0..1] {
      "a" => 0,
      "b" => 1,
      "c" => 2,
      "d" => 3,
      "e" => 4,
      "f" => 5,
      "g" => 6,
      "h" => 7,
      _ => return Err(FenError::InvalidSquare)
    };

    let row = str[1..2].parse::<u8>().unwrap() - 1; 

    Ok((row as usize, column as usize))
  }

  pub fn get_tile(square: (usize, usize)) -> Result<String, FenError> {
    let chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

    if(square.0 > 7 || square.1 > 7) {
      return Err(FenError::InvalidSquare)
    }

    return Ok(chars[square.1].to_string() + &(square.0 + 1).to_string())

  }

  fn get_data_from_move(move_string: &str) -> Result<((usize, usize), (usize, usize), PieceName), FenError> {
    let name = match &move_string[0..1] {
      "N" => PieceName::KNIGHT,
      "R" => PieceName::ROOK,
      "B" => PieceName::BISHOP,
      "Q" => PieceName::QUEEN,
      "K" => PieceName::KING,
      _ => PieceName::PAWN
    }; 
  
    let starting_sqare = &move_string[if name == PieceName::PAWN {0} else {1}..if name == PieceName::PAWN {2} else {3}];
    let starting_coords = ChessBoard::get_square(starting_sqare)?;

    let regex = regex!("[a-h]{1}[1-8]{1}");
    
    let end_square = &regex.captures_iter(move_string).last().unwrap()[0];
    let end_coords = ChessBoard::get_square(end_square)?;
  
    return Ok((starting_coords, end_coords, name))
  
  }

  //TODO test
  pub fn validate_move(&self, move_string: &str, next: Color) -> Result<bool, Box<dyn std::error::Error>> {

    let regex = regex!(r"^((O-O-O)|(O-O)|([RNBKQ]{0,1}([a-h]{1}[1-8]{1})x{0,1}([a-h]{1}[1-8]{1}))|(([a-h]{1}[1-8]{1})x{0,1}([a-h]{1}[1-8]{1})[RNBKQ]{0,1}))$");

    if(!regex.is_match(move_string)) {
      return Err(Box::new(FenError::InvalidFormat))
    }

    if(self.end != EndResult::None(true)) {
      return Err(Box::new(MoveError::GameOver))
    }

    if(move_string == "O-O-O" || move_string == "O-O") {
      let (rook_square, king_square, between_squares) = match (move_string, next) {
        ("O-O-O", Color::Black) => {
          (ChessBoard::get_square("a8")?, ChessBoard::get_square("e8")?, vec![ChessBoard::get_square("b8")?, ChessBoard::get_square("c8")?, ChessBoard::get_square("d8")?])
        },
        ("O-O-O", Color::White) => {
          (ChessBoard::get_square("a1")?, ChessBoard::get_square("e1")?, vec![ChessBoard::get_square("b1")?, ChessBoard::get_square("c1")?, ChessBoard::get_square("d1")?])
        },
        ("O-O", Color::Black) => {
          (ChessBoard::get_square("h8")?, ChessBoard::get_square("e8")?, vec![ChessBoard::get_square("f8")?, ChessBoard::get_square("g8")?])
        },
        ("O-O", Color::White) => {
          (ChessBoard::get_square("h1")?, ChessBoard::get_square("e1")?, vec![ChessBoard::get_square("f1")?, ChessBoard::get_square("g1")?])
        }
        _ => {
          return Err(Box::new(MoveError::InvalidMove))
        }
      };

      if(self.board[rook_square.0][rook_square.1] == Some(Piece{ color: next, name: PieceName::ROOK}) &&
          self.board[king_square.0][king_square.1] == Some(Piece{ color: next, name: PieceName::KING})
        ) {
          if(
            between_squares.iter()
              .all(|square| self.board[square.0][square.1] == None)
          ) {
            if(!self.is_checked(&next) && between_squares.iter().all(|square| {
              let mut fake_board = self.clone();
              let coords = king_square;
              fake_board.board[coords.0][coords.1] = None;
              
              fake_board.board[square.0][square.1] = Some(Piece { color: next, name: PieceName::KING });

              !fake_board.is_checked(&next)
            })) {
              return Ok(true)
            }
          }
        }
        return Ok(false)
    }

    let (starting_coords, end_coords, piece_name) = ChessBoard::get_data_from_move(move_string)?;

    let piece = Piece { color: next, name: piece_name };

    if(!move_string.contains("x")) {
      if(self.board[end_coords.0][end_coords.1] != None) {
        return Ok(false)
      }
    }
    else {
      match self.board[end_coords.0][end_coords.1] {
        Some(piece) if piece.color == next => return Ok(false),
        None if piece.name != PieceName::PAWN => return Ok(false),
        _ => {}
      }
    }

    if(self.board[starting_coords.0][starting_coords.1] == None || self.board[starting_coords.0][starting_coords.1].unwrap() != piece) {
      return Ok(false)
    }

    match piece.name {
        PieceName::ROOK => {
          return Ok(ChessBoard::validate_rook(&starting_coords, &end_coords, &self.board));
        },
        PieceName::KNIGHT => {
          return Ok(
            starting_coords.0.abs_diff(end_coords.0) == 1 && starting_coords.1.abs_diff(end_coords.1) == 2 ||
            (starting_coords.0.abs_diff(end_coords.0) == 2 && starting_coords.1.abs_diff(end_coords.1) == 1) )
        },
        PieceName::BISHOP => {
          return Ok(ChessBoard::validate_bishop(&starting_coords, &end_coords, &self.board))
        },
        PieceName::QUEEN => {
          return Ok(ChessBoard::validate_rook(&starting_coords, &end_coords, &self.board) || ChessBoard::validate_bishop(&starting_coords, &end_coords, &self.board))
        },
        PieceName::KING => {
          let diff = (starting_coords.0.abs_diff(end_coords.0) == 1 || starting_coords.1.abs_diff(end_coords.1) == 1);
          if diff {
            return Ok(ChessBoard::validate_bishop(&starting_coords, &end_coords, &self.board) || ChessBoard::validate_rook(&starting_coords, &end_coords, &self.board))
          }
          return Ok(false)
        },
        PieceName::PAWN => {
          let double = match next {
            Color::Black => starting_coords.0 == 6 && end_coords.0 == 4,
            Color::White => starting_coords.0 == 1 && end_coords.0 == 3,
          };
          let single = match next {
              Color::Black => starting_coords.0.saturating_sub(end_coords.0) == 1,
              Color::White => end_coords.0.saturating_sub(starting_coords.0) == 1,
          };
          let en_passant = match next {
              Color::Black => self.en_passant == Some(end_coords),
              Color::White => self.en_passant == Some(end_coords),
          };

          let mut valid = false;

          if move_string.contains("x") {
            let diagonal_move = match next {
                Color::Black => starting_coords.0.saturating_sub(end_coords.0) == 1 && starting_coords.1.abs_diff(end_coords.1) == 1,
                Color::White => end_coords.0.saturating_sub(starting_coords.0) == 1 && starting_coords.1.abs_diff(end_coords.1) == 1,
            };
        
            valid = diagonal_move && (self.board[end_coords.0][end_coords.1] != None || en_passant)
          } else if (double || single) && starting_coords.1 == end_coords.1 {
            valid = self.board[end_coords.0][end_coords.1] == None || en_passant
          }

          if end_coords.0 == 0 || end_coords.0 == 7 {
            let endings = ["K", "R", "B", "N", "Q"];
            if endings.contains(&move_string.chars().last().unwrap().to_string().as_str()) {
              return Ok(valid)
            }
            return Ok(false)
          }

          return Ok(valid);
        },
    }

    Ok(false)

  }

  fn validate_check(&self, starting_coords: &(usize, usize), end_coords: &(usize, usize), board: &[[Option<Piece>; 8]; 8], alg: &str, next: Color) -> bool {
    let mut fake_board = self.clone();
    fake_board.make_move(alg, next);
    return !fake_board.is_checked(&next);
  }

  fn is_checked(&self, color: &Color) -> bool {
    let king_position = self.board.into_iter().position(|col| col.into_iter().any(|p| p.is_some() && p.unwrap().color == *color && p.unwrap().name == PieceName::KING)).unwrap();

    let king_position = (king_position, self.board[king_position].into_iter().position(|p| p.is_some() && p.unwrap().color == *color && p.unwrap().name == PieceName::KING).unwrap());

    fn check_the_diagonals(board: &[[Option<Piece>; 8]; 8], color: &Color, king_position: (usize, usize)) -> bool {
      let directions = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
      for &(dx, dy) in &directions {
        let mut x = king_position.0 as i8 + dx;
        let mut y = king_position.1 as i8 + dy;
        while x >= 0 && x < 8 && y >= 0 && y < 8 {
          let piece = board[x as usize][y as usize];
          if let Some(other_piece) = piece {
            if other_piece.color != *color {
              if other_piece.name == PieceName::BISHOP || other_piece.name == PieceName::QUEEN {
                return false;
              } else if dx == 1 && dy == 1 && other_piece.name == PieceName::PAWN && *color == Color::White {
                return false;
              } else if dx == -1 && dy == 1 && other_piece.name == PieceName::PAWN && *color == Color::Black {
                return false;
              }
            }
            break;
          }
          x += dx;
          y += dy;
        }
      }
      true
    }
    
    
    fn check_the_lateral(board: &[[Option<Piece>; 8]; 8], color: &Color, king_position: (usize, usize)) -> bool {
      let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
      for &(x, y) in directions.iter() {
        let mut piece: Option<Piece> = None;
        let mut pos = king_position;
        loop {
          match piece {
            Some(other_piece) => {
              if other_piece.color != *color {
                if other_piece.name == PieceName::ROOK || other_piece.name == PieceName::QUEEN {
                  return false;
                }
              }
              break;
            }
            None => {
              if pos.0 as i8 + x < 0 || pos.0 as i8 + x > 7 || pos.1 as i8 + y < 0 || pos.1 as i8 + y > 7 {
                  break;
              }
              pos.0 = (pos.0 as i8 + x) as usize;
              pos.1 = (pos.1 as i8 + y) as usize;
              piece = board[pos.0][pos.1]
            }
          }
        }
      }
      true
    }
  

    fn check_the_knights(board: &[[Option<Piece>; 8]; 8], color: &Color, king_position: (usize, usize)) -> bool {
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
        let row = (king_position.0 as isize) + row_offset;
        let col = (king_position.1 as isize) + col_offset;

        // Make sure the position is within the bounds of the board
        if row < 0 || row > 7 || col < 0 || col > 7 {
            continue;
        }

        let piece = board[row as usize][col as usize];
        if piece.is_some() && piece.unwrap().name == PieceName::KNIGHT && piece.unwrap().color != *color {
            return false;
        }
      }
  
      true
    }

    let diag = check_the_diagonals(&self.board, color, king_position);

    let lat = check_the_lateral(&self.board, color, king_position);

    let paci = check_the_knights(&self.board, color, king_position);
 
    return !diag || !lat || !paci;
  }

  fn validate_bishop(starting_coords: &(usize, usize), end_coords: &(usize, usize), board: &[[Option<Piece>; 8]; 8]) -> bool {
    let row_diff = starting_coords.0 as i32 - end_coords.0 as i32;
    let col_diff = starting_coords.1 as i32 - end_coords.1 as i32;

    if row_diff.abs() != col_diff.abs() || starting_coords.0 == end_coords.0 || starting_coords.1 == end_coords.1 {
      return false;
    }

    let (row_dir, col_dir) = if row_diff > 0 && col_diff > 0 { (-1, -1) }
    else if row_diff > 0 && col_diff < 0 { (-1, 1) }
    else if row_diff < 0 && col_diff > 0 { (1, -1) }
    else { (1, 1) };

    let mut row = starting_coords.0 as i32 + row_dir;
    let mut col = starting_coords.1 as i32 + col_dir;
    while row != end_coords.0 as i32 && col != end_coords.1 as i32 {
      if let Some(_) = board[row as usize][col as usize] {
        return false;
      }
      row += row_dir;
      col += col_dir;
    }
    true
  }
  

  fn validate_rook(starting_coords: &(usize, usize), end_coords: &(usize, usize), board: &[[Option<Piece>; 8]; 8]) -> bool {
    if starting_coords.0 == end_coords.0 || starting_coords.1 == end_coords.1 {
      let (min, max) = if starting_coords.0 == end_coords.0 {
         (std::cmp::min(starting_coords.1, end_coords.1), std::cmp::max(starting_coords.1, end_coords.1)) 
        } else {
           (std::cmp::min(starting_coords.0, end_coords.0), std::cmp::max(starting_coords.0, end_coords.0)) 
        };
      for i in min + 1..max {
        if starting_coords.0 == end_coords.0 {
          if board[starting_coords.0][i] != None { return false; }
        } else {
          if board[i][starting_coords.1] != None { return false; }
        }
      }
      return true;
    }
    return false;
  }


  pub fn to_fen(&self, next: Color) -> Result<String, FenError> {

    let mut output = "".to_owned();

    for row in self.board.into_iter().rev() {
      let mut counter: usize = 0;
      for piece in row.iter() {
        match piece {
            Some(piece) => {
              if(counter != 0) {
                output += &counter.to_string();
              }
              counter = 0;
              match piece.name {
                PieceName::ROOK => output += if(piece.color == Color::White) {"R"} else {"r"},
                PieceName::KNIGHT => output += if(piece.color == Color::White) {"N"} else {"n"},
                PieceName::BISHOP => output += if(piece.color == Color::White) {"B"} else {"b"},
                PieceName::QUEEN => output += if(piece.color == Color::White) {"Q"} else {"q"},
                PieceName::KING => output += if(piece.color == Color::White) {"K"} else {"k"},
                PieceName::PAWN => output += if(piece.color == Color::White) {"P"} else {"p"},
              }
            },
            None => {
              counter += 1;
            },
        }
      }

      if counter != 0 {
        output += &counter.to_string();
      }
      output += "/";

    }

    output = output[0..output.len()-1].to_owned();

    output += " ";
    output += if(next == Color::White) {"w "} else {"b "};

    let mut has_castling = false;
    for &(color, side) in &[(Color::White, Castling::KINGSIDE), (Color::White, Castling::QUEENSIDE), (Color::Black, Castling::KINGSIDE), (Color::Black, Castling::QUEENSIDE)] {
        if *self.castling.get(&(color, side)).unwrap() {
            has_castling = true;
            let letter = match side {
                Castling::KINGSIDE => 'K',
                Castling::QUEENSIDE => 'Q',
            };
            output += &if color == Color::White { letter } else { letter.to_ascii_lowercase() }.to_string();
        }
    }
    if !has_castling {
        output += "-";
    }

    output += " ";

    match self.en_passant {
        Some(tile) => {
          output += &Self::get_tile(tile)?;
        },
        None => output += "-",
    }

    output += " ";

    output += &self.halfmove.to_string();

    output += " ";

    output += &self.fullmove.to_string();

    return Ok(output)

  }



  pub fn make_move(&mut self, alg: &str, next: Color) -> Result<(), Box<dyn std::error::Error>> {

    if(!self.validate_move(alg, next)?) {
      return Err(Box::new(ChessError::MoveError(MoveError::InvalidMove)))
    }

    if(alg == "O-O" || alg == "O-O-O") {
      let (king_old, king_new, rook_old, rook_new) = match (alg, next) {
        ("O-O", Color::White) => (
          ChessBoard::get_square("e1").unwrap(), ChessBoard::get_square("g1").unwrap(),
          ChessBoard::get_square("h1").unwrap(), ChessBoard::get_square("f1").unwrap()
        ),
        ("O-O", Color::Black) => (
          ChessBoard::get_square("e8").unwrap(), ChessBoard::get_square("g8").unwrap(),
          ChessBoard::get_square("h8").unwrap(), ChessBoard::get_square("f8").unwrap()
        ),
        ("O-O-O", Color::White) => (
          ChessBoard::get_square("e1").unwrap(), ChessBoard::get_square("c1").unwrap(),
          ChessBoard::get_square("a1").unwrap(), ChessBoard::get_square("d1").unwrap()
        ),
        ("O-O-O", Color::Black) => (
          ChessBoard::get_square("e8").unwrap(), ChessBoard::get_square("c8").unwrap(),
          ChessBoard::get_square("a8").unwrap(), ChessBoard::get_square("d8").unwrap()
        ),
        _ => return Err(Box::new(MoveError::InvalidMove))
      };

      self.board[king_old.0][king_old.1] = None;
      self.board[king_new.0][king_new.1] = Some(Piece {color: next, name: PieceName::KING});
      self.board[rook_old.0][rook_old.1] = None;
      self.board[rook_new.0][rook_new.1] = Some(Piece {color: next, name: PieceName::ROOK});

      self.halfmove += 1;

      self.castling.insert((next, Castling::KINGSIDE), false);
      self.castling.insert((next, Castling::QUEENSIDE), false);

      if(next == Color::Black) {
        self.fullmove += 1;
      }

      self.en_passant = None;
  
      return Ok(())

    }

    let (starting_coords, end_coords, piece_name) = Self::get_data_from_move(alg).map_err(|e| ChessError::FenError(e))?;
    
    let piece = Piece { color: next, name: piece_name };

    let starting_square = Self::get_tile(starting_coords)?;
    
    let opposite = if next == Color::White {Color::Black} else {Color::White};

    if 
      self.board[end_coords.0][end_coords.1].is_some() 
      && !self.board.iter().any(|row| 
          row.iter().any(|cell| 
            cell.is_some() && cell.unwrap().name == PieceName::KING && cell.unwrap().color == opposite
          )
        ) {
      self.end = EndResult::Color(next as i32);
    }

    self.board[starting_coords.0][starting_coords.1] = None;

    self.board[end_coords.0][end_coords.1] = Some(piece);

    if(piece.name == PieceName::KING) {
      self.castling.insert((next, Castling::KINGSIDE), false);
      self.castling.insert((next, Castling::QUEENSIDE), false);
    }
    else if (piece.name == PieceName::ROOK) {
      match next {
        Color::Black => {
          if(starting_square == "a8") {
            self.castling.insert((Color::Black, Castling::QUEENSIDE), false);
          }
          else if (starting_square == "h8") {
            self.castling.insert((Color::Black, Castling::KINGSIDE), false);
          }
        }
        Color::White => {
          if(starting_square == "a1") {
            self.castling.insert((Color::White, Castling::QUEENSIDE), false);
          }
          else if (starting_square == "h1") {
            self.castling.insert((Color::White, Castling::KINGSIDE), false);
          }
        }
      }
    }
    if (piece.name == PieceName::PAWN && (starting_coords.0.abs_diff(end_coords.0) == 2)) {
        self.en_passant = Some(((starting_coords.0 + end_coords.0)/2, starting_coords.1));
    }
    else {
      self.en_passant = None;
    }

    if(piece.name == PieceName::PAWN || alg.contains("x")) {
      self.halfmove = 0;
    }
    else {
      self.halfmove += 1;
    }

    let endings = ["Q", "K", "R", "N", "B"];

    if piece.name == PieceName::PAWN && endings.contains(&alg.chars().last().unwrap().to_string().as_str()) {
      let promoted_name = match alg.chars().last().unwrap().to_string().as_str() {
        "Q" => PieceName::QUEEN,
        "K" => PieceName::KING,
        "R" => PieceName::ROOK,
        "N" => PieceName::KNIGHT,
        "B" => PieceName::BISHOP,
        _ => panic!("BRUHHHHH")
      };
      self.board[end_coords.0][end_coords.1] = Some(Piece { name: promoted_name, color: piece.color });
    }

    if(next == Color::Black) {
      self.fullmove += 1;
    }
    
    let binding = self.to_fen(next).unwrap();
    let curr_fen = binding.split(" ").next().unwrap();

    let count = self.past.iter().filter(|fen| {let checked = fen.split(" ").next().unwrap(); println!("{checked}, {curr_fen}");  checked == curr_fen}).count();
    
    if(count >= 2) {
      self.end = EndResult::Draw(true)
    }
    
    self.past.push(binding);

    Ok(())
  }

}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Castling {
  QUEENSIDE,
  KINGSIDE
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
  pub color: Color,
  pub name: PieceName
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceName {
  ROOK,
  KNIGHT,
  BISHOP,
  QUEEN,
  KING,
  PAWN
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