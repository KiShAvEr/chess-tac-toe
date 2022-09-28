#![allow(unused)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{self, Sender};
use tonic::codegen::http::request;
use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use helpers::{TicTacToe as HelperToe, TicError, ChessBoard, chesstactoe::{game_server::{Game, GameServer}, join_response::GameStatus}};
use helpers::chesstactoe::{JoinRequest, JoinResponse, TicTacToe, MovePieceRequest, MovePieceResponse, Color, Chess, SubscribeBoardRequest, MoveResult, SubscribeBoardResponse, TakeBackRequest, TakeBackResponse, MidGameRequest};
use uuid::{Uuid, uuid};

#[derive(Debug, Default)]
pub struct ChessServer {}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Ongoing {
    white: Uuid,
    black: Uuid,
    game: HelperToe
}

#[derive(Debug, Default)]
struct GameService {
    q: Arc<RwLock<Vec<(Uuid, Sender<Result<JoinResponse, Status>>)>>>, 
    receivers: Arc<RwLock<HashMap<Uuid, Sender<Result<SubscribeBoardResponse, Status>>>>>,
    game_ids: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    games: Arc<RwLock<HashMap<Uuid, Ongoing>>>
}

#[tonic::async_trait]
impl Game for GameService {
    async fn move_piece(
        &self,
        request: Request<MovePieceRequest>
    ) -> Result<Response<MovePieceResponse>, Status> {

        let request = request.into_inner();

        let uuid = Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        if(self.game_ids.read().await.get(&uuid)).is_none() {
            return Err(Status::permission_denied("User needs to join a game first"))
        }

        let uuids = self.game_ids.read().await;

        let game_uuid = uuids.get(&uuid).unwrap();

        let mut games = self.games.write().await;

        let mut game = games.get(game_uuid).unwrap().clone();

        let white = game.white;
        let black = game.black;

        let color = if(uuid == white) {Color::White} else if(uuid == black) {Color::Black} else {return Err(Status::internal("Something went wrong"))};

        let tic = make_move(game.game.to_fen().unwrap(), 
            ((request.board/3).try_into().unwrap(), (request.board%3).try_into().unwrap()), 
            &request.alg, 
            color).map_err(|e| Status::internal(e.to_string()))?;
        
        game.game = tic;

        games.insert(*game_uuid, game.clone());

        let recs = self.receivers.read().await;

        let chesses = game.game.chesses;
        let next = game.game.next;

        let res = SubscribeBoardResponse {
            game: Some(TicTacToe { chesses: chesses.into_iter().flatten().map(|chess| Chess { winner: chess.winner as i32, fen: chess.to_fen(next).unwrap() }).collect(), next: next as i32, last_move: request.alg }),
            color: if (game.black == uuid) {Color::Black as i32} else {Color::White as i32},
            request: Some(MidGameRequest {
                draw: Color::None as i32,
                takeback: Color::None as i32
            })
        };

        recs.get(&game.white).unwrap().send(Ok(res.clone())).await.unwrap();
        recs.get(&game.black).unwrap().send(Ok(res.clone())).await.unwrap();
        
        Ok(Response::new(MovePieceResponse { successful: MoveResult::ResultSuccessful as i32 }))
    }

    type JoinStream = ReceiverStream<Result<JoinResponse, Status>>;

    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<Self::JoinStream>, Status> {

        let uuid = Uuid::new_v4();

        if(!self.q.read().await.is_empty()) {
            let white = self.q.read().await.first().unwrap().clone();
            self.q.write().await.remove(0);
            let game: Ongoing = Ongoing {
                white: white.0,
                black: uuid,
                game: HelperToe::default()
            };

            let game_uuid = Uuid::new_v4();

            self.games.write().await.insert(game_uuid, game);

            self.game_ids.write().await.insert(uuid, game_uuid);

            self.game_ids.write().await.insert(white.0, game_uuid);

            white.1.send(Ok(JoinResponse { status: GameStatus::Ready as i32, uuid: white.0.to_string() })).await.unwrap();

            let (mut tx, rx) = mpsc::channel(4);

            tx.send(Ok(JoinResponse { status: GameStatus::Ready as i32, uuid: uuid.to_string() })).await.unwrap();

            tx.closed();

            white.1.closed();
            
            return Ok(Response::new(ReceiverStream::new(rx)))
        } 

        let (mut tx, rx) = mpsc::channel(4);

        self.q.write().await.push((uuid, tx.clone()));

        tx.send(Ok(JoinResponse { status: GameStatus::NotReady as i32, uuid: uuid.to_string()})).await.unwrap();

        println!("picsa");
    
        return Ok(Response::new(ReceiverStream::new(rx)))

    }

    type SubscribeBoardStream = ReceiverStream<Result<SubscribeBoardResponse, Status>>;

    async fn subscribe_board(&self, request: Request<SubscribeBoardRequest>) -> Result<Response<Self::SubscribeBoardStream>, Status> {

        let request = request.into_inner();

        let asker = Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        if(self.game_ids.read().await.get(&asker).is_none()) {
            return Err(Status::permission_denied("User needs to join a game first"))
        }

        let (mut tx, rx) = mpsc::channel(4);

        self.receivers.write().await.insert(asker, tx.clone());

        let game = self.games.read().await.get(self.game_ids.read().await.get(&asker).unwrap()).unwrap().clone();

        let res: SubscribeBoardResponse = SubscribeBoardResponse {
            game: Some(TicTacToe {chesses:game.game.chesses.into_iter().flatten().map(|chess|Chess{winner:Color::None as i32,fen:chess.to_fen(game.game.next).unwrap()}).collect(),next:game.game.next as i32, last_move: "".to_owned()}),
            color: if(asker == game.black) {Color::Black as i32} else {Color::White as i32},
            request: None
        };

        tx.send(Ok(res)).await.unwrap();

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn take_back(&self, request: Request<TakeBackRequest>) -> Result<Response<TakeBackResponse>, Status> {

        let request = request.into_inner();

        let uuid = Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        if let Some(game_id) = self.game_ids.read().await.get(&uuid) {
            if let Some(game) = self.games.read().await.get(game_id) {
                
            }
        }

        return Err(Status::permission_denied("User is not in a game"));

        Ok(Response::new(TakeBackResponse{}))
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let fasz = GameService::default();

    Server::builder()
        .add_service(GameServer::new(fasz))
        .serve("[::1]:50051".parse()?)
        .await?;

    println!("server running on port {}", 50051);
        
    Ok(())

}



pub fn make_move(tic: String, board: (usize, usize), alg: &str, mover: Color) -> Result<helpers::TicTacToe, Box<dyn std::error::Error>> {

    let mut tic = HelperToe::from_fen(&tic)?;

    if(board.0 > 2 || board.1 > 2) {
      return Err(Box::new(TicError::InvalidCoords));
    }

    if(tic.next != mover) {
      return Err(Box::new(TicError::WrongColor))
    }

    let chessboard = &mut tic.chesses[board.0][board.1];

    chessboard.make_move(mover, alg, tic.next)?;

    tic.next = if tic.next == Color::White {Color::Black} else {Color::White};

    return Ok(tic);
  }