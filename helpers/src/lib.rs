// #![allow(unused)]

pub mod tictactoe;
pub mod chess;

use std::fmt::Display;

use chesstactoe::*;

pub mod chesstactoe {
  tonic::include_proto!("chesstactoe");
}

impl Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Color::White => write!(f, "White"),
      Color::Black => write!(f, "Black"),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FenError {
  InvalidFormat,
  InvalidSquare,
}

impl std::error::Error for FenError {}

impl Display for FenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

#[derive(Debug)]
pub enum MoveError {
  WrongColor,
  InvalidMove,
  GameOver,
}

impl Display for MoveError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Coordinates {
  pub row: usize,
  pub col: usize,
}

impl Coordinates {
  pub const fn new(coords: (usize, usize)) -> Self {
    Coordinates {
      row: coords.0,
      col: coords.1,
    }
  }
}

impl std::error::Error for MoveError {}
