#![allow(unused)]

use chesstactoe::{game_client::GameClient, SubscribeBoardRequest};
use tokio::sync::broadcast::Receiver;
use tonic::{Request, Status, codegen::futures_core::stream};
use helpers::{ChessBoard, TicTacToe, chesstactoe::Color};

use crate::chesstactoe::JoinRequest;

pub mod chesstactoe {
  tonic::include_proto!("chesstactoe");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // let mut client = GameClient::connect("http://[::1]:50051").await?;

  // let mut res = client.join(Request::new(JoinRequest {})).await?.into_inner();

  // let mut res = client.join(Request::new(JoinRequest {})).await?.into_inner();

  // while let Some(a) = res.message().await? {
  //   println!("{:?}", a);
  // }


  // let tic = TicTacToe::default();

  let chess = ChessBoard::parse_fen("rnbqkbnr/1p1ppppp/8/8/8/8/PpPPPPPP/R1BQ1BNR w KQkq - 0 1")?;

  println!("{:?}", chess.winner);

  Ok(())
}