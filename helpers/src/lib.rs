#![allow(unused)]

use std::{fmt::{Display, write}, collections::HashMap};

use chesstactoe::*;
pub mod chesstactoe {
  tonic::include_proto!("chesstactoe");
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
  pub fn validate_move(&self, board: (usize, usize), alg: &str) -> Result<bool, Box<dyn std::error::Error>>{

    if(board.0 > 2 || board.1 > 2) {
      return Err(Box::new(TicError::InvalidCoords));
    }

    Ok(self.chesses[board.0][board.1].validate_move(alg, self.next)?)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessBoard {
  board: [[Option<Piece>; 8]; 8],
  castling: HashMap<(Color, Castling), bool>,
  en_passant: Option<(usize, usize)>,
  halfmove: usize,
  fullmove: usize, 
  pub winner: Color
}

impl Default for ChessBoard {
  fn default() -> Self {
      return Self::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
  }
}

impl ChessBoard {

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
    
    let winner = if(!board.into_iter().any(|a| a.contains(&Some(Piece {name: PieceName::KING, color: Color::Black})))) {
      Color::White
    } else if (!board.into_iter().any(|a| a.contains(&Some(Piece {name: PieceName::KING, color: Color::White})))) {
      Color::Black
    }
    else {
      Color::None
    };
  
    Ok(ChessBoard {board, winner, castling, 
      en_passant: if(en_passant == "-") {None} else {Some(ChessBoard::get_square(en_passant)?)}, 
      halfmove: halfmove.parse::<usize>().map_err(|_| FenError::InvalidFormat)?, 
      fullmove: fullmove.parse::<usize>().map_err(|_| FenError::InvalidFormat)?}
    )
  
  }

  fn get_square(str: &str) -> Result<(usize, usize), FenError> {
    let regex = Regex::new("[a-h]{1}[1-8]{1}").unwrap();
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

  fn get_tile(square: (usize, usize)) -> Result<String, FenError> {
    let chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

    if(square.0 > 7 || square.1 > 7) {
      return Err(FenError::InvalidSquare)
    }

    return Ok(chars[square.1].to_string() + &(square.0 + 1).to_string())

  }


  //TODO castling
  pub fn validate_move(&self, move_string: &str, next: Color) -> Result<bool, Box<dyn std::error::Error>> {

    let regex = Regex::new(r"(O-O-O)|(O-O)|([R,N,B,K,Q]{0,1}([a-h]{1}[1-8]{1})x{0,1}([a-h]{1}[1-8]{1})\+{0,1})")
                        .unwrap();

    if(!regex.is_match(move_string)) {
      return Err(Box::new(FenError::InvalidFormat))
    }

    if(self.winner != Color::None) {
      return Err(Box::new(MoveError::GameOver))
    } 

    if(move_string == "O-O-O") {
      if(!*self.castling.get(&(next, Castling::QUEENSIDE)).unwrap()) {
        return Ok(false)
      }

      match next {
          Color::Black => {
            let rook_square = ChessBoard::get_square("a8")?;
            let king_square = ChessBoard::get_square("e8")?;
            if(self.board[rook_square.0][rook_square.1] == Some(Piece{ color: Color::Black, name: PieceName::ROOK}) &&
                self.board[king_square.0][king_square.1] == Some(Piece{ color: Color::Black, name: PieceName::KING})
              ) {
                if(
                  [ChessBoard::get_square("b8")?, ChessBoard::get_square("c8")?, ChessBoard::get_square("d8")?].into_iter()
                    .all(|square| self.board[square.0][square.1] == None)
                ) {
                  return Ok(true)
                }
                return Ok(false)
            }
          },
          Color::White => {
            let rook_square = ChessBoard::get_square("a1")?;
            let king_square = ChessBoard::get_square("e1")?;
            if(self.board[rook_square.0][rook_square.1] == Some(Piece{ color: Color::White, name: PieceName::ROOK}) &&
                self.board[king_square.0][king_square.1] == Some(Piece{ color: Color::White, name: PieceName::KING})
              ) {
                if(
                  [ChessBoard::get_square("b1")?, ChessBoard::get_square("c1")?, ChessBoard::get_square("d1")?].into_iter()
                    .all(|square| self.board[square.0][square.1] == None)
                ) {
                  return Ok(true)
                }
                return Ok(false)
            }
          }
        Color::None => {},
      }

      return Ok(true);
    }
    else if move_string == "O-O" {
      if(!*self.castling.get(&(next, Castling::KINGSIDE)).unwrap()) {
        return Ok(false)
      }

      match next {
          Color::Black => {
            let rook_square = ChessBoard::get_square("h8")?;
            let king_square = ChessBoard::get_square("e8")?;
            if(self.board[rook_square.0][rook_square.1] == Some(Piece{ color: Color::Black, name: PieceName::ROOK}) &&
                self.board[king_square.0][king_square.1] == Some(Piece{ color: Color::Black, name: PieceName::KING})
              ) {
                if(
                  [ChessBoard::get_square("f8")?, ChessBoard::get_square("g8")?].into_iter()
                    .all(|square| self.board[square.0][square.1] == None)
                ) {
                  return Ok(true)
                }
                return Ok(false)
            }
          },
          Color::White => {
            let rook_square = ChessBoard::get_square("h1")?;
            let king_square = ChessBoard::get_square("e1")?;
            if(self.board[rook_square.0][rook_square.1] == Some(Piece{ color: Color::White, name: PieceName::ROOK}) &&
                self.board[king_square.0][king_square.1] == Some(Piece{ color: Color::White, name: PieceName::KING})
              ) {
                if(
                  [ChessBoard::get_square("f1")?, ChessBoard::get_square("g1")?].into_iter()
                    .all(|square| self.board[square.0][square.1] == None)
                ) {
                  return Ok(true)
                }
                return Ok(false)
            }
          }
        Color::None => todo!(),
      }

      return Ok(true);
    }

    let piece = match &move_string[0..1] {
      "N" => Piece {color: next, name: PieceName::KNIGHT},
      "R" => Piece {color: next, name: PieceName::ROOK},
      "B" => Piece {color: next, name: PieceName::BISHOP},
      "Q" => Piece {color: next, name: PieceName::QUEEN},
      "K" => Piece {color: next, name: PieceName::KING},
      _ => Piece {color: next, name: PieceName::PAWN}
    };

    let starting_sqare = &move_string[if piece.name == PieceName::PAWN {0} else {1}..if piece.name == PieceName::PAWN {2} else {3}];
    let starting_coords = Self::get_square(starting_sqare)?;
    
    let end_square = &Regex::new("[a-h]{1}[1-8]{1}").unwrap().captures_iter(move_string).last().unwrap()[0];
    let end_coords = Self::get_square(end_square)?;

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
          match move_string.contains("x") {
              true => {
                match next {
                  Color::Black => {
                    if(starting_coords.0 > end_coords.0 && starting_coords.0 - end_coords.0 == 1 && starting_coords.1.abs_diff(end_coords.1) == 1) {
                      return Ok(self.board[end_coords.0][end_coords.1] != None || self.en_passant == Some(end_coords))
                    }
                    return Ok(false)
                  }
                  Color::White => {
                    if(end_coords.0 > starting_coords.0 && end_coords.0 - starting_coords.0 == 1 && starting_coords.1.abs_diff(end_coords.1) == 1) {
                      return Ok(self.board[end_coords.0][end_coords.1] != None || self.en_passant == Some(end_coords))
                    }
                    return Ok(false)
                  }
                    Color::None => todo!(),
                }
              },
              false => {
                match next {
                  Color::Black => {
                    let double = starting_coords.0 == 6 && end_coords.0 == 4;
                    let single = starting_coords.0 - end_coords.0 == 1 && starting_coords.1 == end_coords.1;
                    if(double || single) {
                      return Ok(self.board[end_coords.0][end_coords.1] == None)
                    }
                    return Ok(false)
                  }
                  Color::White => {
                    let double = starting_coords.0 == 1 && end_coords.0 == 3;
                    let single = end_coords.0 - starting_coords.0 == 1 && starting_coords.1 == end_coords.1;
                    if(double || single) {
                      return Ok(self.board[end_coords.0][end_coords.1] == None)
                    }
                    return Ok(false)
                  }
                    Color::None => todo!(),
                }
              }
          }
        },
    }

    Ok(false)

  }

  fn validate_bishop(starting_coords: &(usize, usize), end_coords: &(usize, usize), board: &[[Option<Piece>; 8]; 8]) -> bool {
    if(starting_coords.0.abs_diff(end_coords.0) != starting_coords.1.abs_diff(end_coords.1)) {
      return false
    }

    if(starting_coords.0 == end_coords.0 || starting_coords.1 == end_coords.1) {
      return false
    }

    match starting_coords.0 > end_coords.0 {
        true => {
          match starting_coords.1 > end_coords.1 {
              true => {
                for i in 1..starting_coords.1-end_coords.1 {
                  if(board[end_coords.0-i][end_coords.1+i] != None) {
                    return false
                  }
                }
                return true
              },
              false => {
                for i in 1..end_coords.1-starting_coords.1 {
                  if(board[end_coords.0+i][end_coords.1-i] != None) {
                    return false
                  }
                }
                return true
              }
          }
        },
        false => {
          match starting_coords.1 > end_coords.1 {
              true => {
                for i in 1..starting_coords.1-end_coords.1 {
                  if(board[end_coords.0-i][end_coords.1+i] != None) {
                    return false
                  }
                }
                return true
              },
              false => {
                for i in 1..end_coords.1-starting_coords.1 {
                  if(board[end_coords.0+i][end_coords.1-i] != None) {
                    return false
                  }
                }
                return true
              }
          }
        }
    }

  }

  fn validate_rook(starting_coords: &(usize, usize), end_coords: &(usize, usize), board: &[[Option<Piece>; 8]; 8]) -> bool {
    match starting_coords.0.abs_diff(end_coords.0) {
      0 => {
        if starting_coords.1.abs_diff(end_coords.1) == 0 {
          return false;
        }
        
        for i in if starting_coords.1 > end_coords.1 {end_coords.1+1..starting_coords.1} else {starting_coords.1+1..end_coords.1} {
          if(board[starting_coords.0][i] != None) {
            return false;
          }
        }

        return true

      },
      _ => {
        if starting_coords.0.abs_diff(end_coords.0) == 0 {
          return false;
        }
        
        for i in if starting_coords.0 > end_coords.0 {end_coords.0+1..starting_coords.0} else {starting_coords.0+1..end_coords.0} {
          if(board[i][starting_coords.1] != None) {
            return false;
          }
        }

        return true
      }
    }

  }

  pub fn to_fen(&self, next: Color) -> Result<String, FenError> {

    let mut output = "".to_owned();

    for i in self.board.into_iter().rev() {
      let mut counter: usize = 0;
      for (jindex, piece) in i.iter().enumerate() {
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

    if*(self.castling.get(&(Color::White, Castling::KINGSIDE)).unwrap()) {
      output += "K"
    }
    if*(self.castling.get(&(Color::White, Castling::QUEENSIDE)).unwrap()) {
      output += "Q"
    }
    if*(self.castling.get(&(Color::Black, Castling::KINGSIDE)).unwrap()) {
      output += "k"
    }
    if*(self.castling.get(&(Color::Black, Castling::QUEENSIDE)).unwrap()) {
      output += "q"
    }

    if !(*(self.castling.get(&(Color::White, Castling::QUEENSIDE)).unwrap()) || *(self.castling.get(&(Color::White, Castling::KINGSIDE)).unwrap())  || *(self.castling.get(&(Color::Black, Castling::QUEENSIDE)).unwrap()) || *(self.castling.get(&(Color::Black, Castling::KINGSIDE)).unwrap())) {
      output += "-"
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



  pub fn make_move(&mut self, mover: Color, alg: &str, next: Color) -> Result<(), Box<dyn std::error::Error>> {
    
    let piece = match &alg[0..1] {
      "N" => Piece {color: mover, name: PieceName::KNIGHT},
      "R" => Piece {color: mover, name: PieceName::ROOK},
      "B" => Piece {color: mover, name: PieceName::BISHOP},
      "Q" => Piece {color: mover, name: PieceName::QUEEN},
      "K" => Piece {color: mover, name: PieceName::KING},
      _ => Piece {color: mover, name: PieceName::PAWN}
    };

    let starting_sqare = &alg[if piece.name == PieceName::PAWN {0} else {1}..if piece.name == PieceName::PAWN {2} else {3}];
    let starting_coords = Self::get_square(starting_sqare).map_err(|e| ChessError::FenError(e))?;
    
    let end_square = &Regex::new("[a-h]{1}[1-8]{1}").unwrap().captures_iter(alg).last().unwrap()[0];
    let end_coords = Self::get_square(end_square).map_err(|e| ChessError::FenError(e))?;
    
    if(!self.validate_move(alg, next)?) {
      return Err(Box::new(ChessError::MoveError(MoveError::InvalidMove)))
    }

    if(self.board[end_coords.0][end_coords.1].is_some() && self.board[end_coords.0][end_coords.1].unwrap().name == PieceName::KING) {
      self.winner = mover;
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
          if(starting_sqare == "a8") {
            self.castling.insert((Color::Black, Castling::QUEENSIDE), false);
          }
          else if (starting_sqare == "h8") {
            self.castling.insert((Color::Black, Castling::KINGSIDE), false);
          }
        }
        Color::White => {
          if(starting_sqare == "a1") {
            self.castling.insert((Color::White, Castling::QUEENSIDE), false);
          }
          else if (starting_sqare == "h1") {
            self.castling.insert((Color::White, Castling::KINGSIDE), false);
          }
        }
        Color::None => todo!(),
      }
    }
    if (piece.name == PieceName::PAWN) {
      if(starting_coords.0.abs_diff(end_coords.0) == 2) {
        self.en_passant = Some((starting_coords.0, (starting_coords.1 + end_coords.1)/2));
      }
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

    if(mover == Color::Black) {
      self.fullmove += 1;
    }

    let ret = self;
    Ok(())
  }

}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Castling {
  QUEENSIDE,
  KINGSIDE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PieceName {
  ROOK,
  KNIGHT,
  BISHOP,
  QUEEN,
  KING,
  PAWN
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
  color: Color,
  name: PieceName
}