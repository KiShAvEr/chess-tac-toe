#![allow(unused)]

use chesstactoe::{game_client::GameClient, SubscribeBoardRequest};
use tonic::Request;
use helpers::{parse_fen, TicTacToe, Color};

pub mod chesstactoe {
  tonic::include_proto!("chesstactoe");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // let mut client = GameClient::connect("http://[::1]:50051").await?;

  // let mut res = client.subscribe_board(Request::new(SubscribeBoardRequest {board_id: 0})).await?.into_inner();

  // while let Some(thing) = res.message().await? {
  //   println!("{:?}", thing)
  // }

  let tic = TicTacToe::default();

  let chess = parse_fen("rp2k2r/p7/8/8/8/8/P7/R3K1pR b KQkq - 0 1")?;

  println!("{:?}", chess.validate_move("O-O-O", Color::BLACK)?);

  Ok(())
}