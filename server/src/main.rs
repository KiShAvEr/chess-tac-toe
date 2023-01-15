#![allow(unused)]

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use dashmap::DashMap;
use tokio::sync::{mpsc::{self, Sender}, Mutex};
use tonic::{codegen::http::{request, self}, transport::Server, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use helpers::{TicTacToe as HelperToe, TicError, ChessBoard, chesstactoe::{game_server::{Game, GameServer}, join_response::GameStatus, chess::EndResult}};
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
    q: Arc<Mutex<Vec<(Uuid, Sender<Result<JoinResponse, Status>>)>>>, 
    receivers: Arc<DashMap<Uuid, Sender<Result<SubscribeBoardResponse, Status>>>>,
    game_ids: Arc<DashMap<Uuid, Uuid>>,
    games: Arc<DashMap<Uuid, Ongoing>>
}

#[tonic::async_trait]
impl Game for GameService {
    async fn move_piece(
        &self,
        request: Request<MovePieceRequest>
    ) -> Result<Response<MovePieceResponse>, Status> {

        let request = request.into_inner();

        let uuid = Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        if(!self.game_ids.contains_key(&uuid)) {
            return Err(Status::permission_denied("User needs to join a game first"))
        }

        let game = self.game_ids.get(&uuid).unwrap();

        let game_uuid = game.value();

        let mut game = self.games.get_mut(game_uuid).unwrap();

        let game = game.value_mut();

        let white = game.white;
        let black = game.black;

        let color = if(uuid == white) {Color::White} else if(uuid == black) {Color::Black} else {return Err(Status::internal("Something went wrong"))};

        let requested_board: (usize, usize) = ((request.board/3).try_into().unwrap(), (request.board%3).try_into().unwrap());

        if color != game.game.next {
            return Err(Status::permission_denied("You're not next"))
        }

        game.game.make_move(requested_board, &request.alg).map_err(|e| Status::internal(e.to_string()))?;
        
        let chesses = &game.game.chesses;
        let next = game.game.next;

        let mut res = SubscribeBoardResponse {
            game: Some(TicTacToe { chesses: chesses.iter().flatten().map(|chess| Chess { end_result: Some(chess.end.clone()), fen: chess.to_fen(next).unwrap() }).collect(), next: next as i32, last_move: request.alg }),
            color: Color::White.into(),
            request: Some(MidGameRequest {
                draw: None,
                takeback: None
            })
        };

        self.receivers.get(&game.white).unwrap().send(Ok(res.clone())).await.unwrap();
        res.color = Color::Black.into();
        self.receivers.get(&game.black).unwrap().send(Ok(res)).await.unwrap();
        
        Ok(Response::new(MovePieceResponse { successful: MoveResult::ResultSuccessful as i32 }))
    }

    type JoinStream = ReceiverStream<Result<JoinResponse, Status>>;

    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<Self::JoinStream>, Status> {

        let uuid = Uuid::new_v4();

        if(!self.q.lock().await.is_empty()) {
            let mut q = self.q.lock().await;
            let white = q.remove(0);;
            let game: Ongoing = Ongoing {
                white: white.0,
                black: uuid,
                game: HelperToe::default()
            };

            let game_uuid = Uuid::new_v4();

            self.games.insert(game_uuid, game);

            self.game_ids.insert(uuid, game_uuid);

            self.game_ids.insert(white.0, game_uuid);

            println!("{:?}", white.1);

            white.1.send(Ok(JoinResponse { status: GameStatus::Ready as i32, uuid: white.0.to_string() })).await.unwrap_or_else(|a| {
                println!("{}", a);
                ()
            });

            let (mut tx, rx) = mpsc::channel(4);

            tx.send(Ok(JoinResponse { status: GameStatus::Ready as i32, uuid: uuid.to_string() })).await.unwrap();

            tx.closed();

            white.1.closed();
            
            return Ok(Response::new(ReceiverStream::new(rx)))
        }

        let (mut tx, rx) = mpsc::channel(4);

        self.q.lock().await.push((uuid, tx.clone()));

        tx.send(Ok(JoinResponse { status: GameStatus::NotReady as i32, uuid: uuid.to_string()})).await.unwrap();

        println!("picsa");
    
        return Ok(Response::new(ReceiverStream::new(rx)))

    }

    type SubscribeBoardStream = ReceiverStream<Result<SubscribeBoardResponse, Status>>;

    async fn subscribe_board(&self, request: Request<SubscribeBoardRequest>) -> Result<Response<Self::SubscribeBoardStream>, Status> {

        let request = request.into_inner();

        let asker = Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        if(!self.game_ids.contains_key(&asker)) {
            return Err(Status::permission_denied("User needs to join a game first"))
        }

        let (mut tx, rx) = mpsc::channel(4);

        self.receivers.insert(asker, tx.clone());

        let game = self.games.get(&self.game_ids.get(&asker).unwrap().value()).unwrap();

        let game = game.value();

        let res: SubscribeBoardResponse = SubscribeBoardResponse {
            game: Some(TicTacToe {chesses:game.game.chesses.iter().flatten().map(|chess|Chess{end_result: Some(EndResult::None(true)), fen:chess.to_fen(game.game.next).unwrap()}).collect(), next:game.game.next as i32, last_move: "".to_owned()}),
            color: if(asker == game.black) {Color::Black as i32} else {Color::White as i32},
            request: None
        };

        tx.send(Ok(res)).await.unwrap();

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn take_back(&self, request: Request<TakeBackRequest>) -> Result<Response<TakeBackResponse>, Status> {

        let request = request.into_inner();

        let uuid = Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        if let Some(game_id) = self.game_ids.get(&uuid) {
            if let Some(game) = self.games.get(&game_id.value()) {
                
            }
        }

        return Err(Status::permission_denied("User is not in a game"));

        Ok(Response::new(TakeBackResponse{}))
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let fasz = GameService::default();

    println!("Server listening on 0.0.0.0:50051");

    Server::builder()
        .add_service(GameServer::new(fasz))
        .serve("0.0.0.0:50051".parse()?)
        .await?;
        
    Ok(())

}