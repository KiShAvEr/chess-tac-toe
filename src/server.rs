#![allow(unused)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{self, Sender};
use tonic::codegen::futures_core::Stream;
use tonic::{transport::Server, Request, Response, Status, codegen::http::request};
use chesstactoe::*;
use chesstactoe::game_server::{Game, GameServer};
use tokio_stream::wrappers::ReceiverStream;
use helpers::{TicTacToe, TicError, Color as HelperColor};

pub mod chesstactoe {
    tonic::include_proto!("chesstactoe");
}

#[derive(Debug, Default)]
pub struct ChessServer {}

#[derive(Debug)]
struct Ongoing {
    white: SocketAddr,
    black: SocketAddr,
    game: TicTacToe
}

#[derive(Debug, Default)]
struct GameService {
    q: Vec<SocketAddr>, 
    receivers: Arc<RwLock<HashMap<SocketAddr, Sender<Result<BoardState, Status>>>>>,
    games: Arc<RwLock<HashMap<SocketAddr, &'static Ongoing>>>
}

#[tonic::async_trait]
impl Game for GameService {
    async fn move_piece(
        &self,
        request: Request<MovePieceRequest>
    ) -> Result<Response<MovePieceResponse>, Status> {

        let asker = request.remote_addr();

        let request = request.into_inner();

        make_move(TicTacToe::default(), (request.board[0] as usize, request.board[1] as usize), &request.alg, helpers::Color::WHITE).map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(MovePieceResponse { successful: 1, board: None }))
    }

    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<JoinResponse>, Status> {

        println!("{:?}", request);
    
        Ok(Response::new(JoinResponse::default()))
    }

    // type SubscribeBoardStream = Pin<Box<dyn Stream<Item = Result<BoardState, Status>> + Send + 'static>>;
    type SubscribeBoardStream = ReceiverStream<Result<BoardState, Status>>;

    async fn subscribe_board(&self, request: Request<SubscribeBoardRequest>) -> Result<Response<Self::SubscribeBoardStream>, Status> {

        println!("{:?}", request);

        let (mut tx, rx) = mpsc::channel(4);

        self.receivers.write().await.insert(request.remote_addr().unwrap(), tx);

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let fasz = GameService::default();

    Server::builder()
        .add_service(GameServer::new(fasz))
        .serve("[::1]:50051".parse()?)
        .await?;
        
    Ok(())

}



pub fn make_move(mut tic: helpers::TicTacToe, board: (usize, usize), alg: &str, mover: helpers::Color) -> Result<helpers::TicTacToe, Box<dyn std::error::Error>> {
    if(board.0 > 2 || board.1 > 2) {
      return Err(Box::new(TicError::InvalidCoords));
    }

    if(tic.next != mover) {
      return Err(Box::new(TicError::WrongColor))
    }

    let chessboard = &mut tic.chesses[board.0][board.1];

    chessboard.make_move(mover, alg, tic.next)?;

    tic.next = if tic.next == helpers::Color::WHITE {HelperColor::BLACK} else {helpers::Color::WHITE};

    return Ok(tic);
  }