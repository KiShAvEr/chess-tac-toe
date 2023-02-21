#![allow(unused)]

use base64::Engine;
use dashmap::DashMap;
use helpers::chesstactoe::{
  Chess, Color, JoinLobbyRequest, JoinRequest, JoinResponse, MakeLobbyRequest, MakeLobbyResponse,
  MidGameRequest, MovePieceRequest, MovePieceResponse, MoveResult, SubscribeBoardRequest,
  SubscribeBoardResponse, TakeBackRequest, TakeBackResponse, TicTacToe,
};
use helpers::{
  chesstactoe::{
    chess::EndResult,
    game_server::{Game, GameServer},
    join_response::GameStatus,
  },
  ChessBoard, TicError, TicTacToe as HelperToe,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{
  mpsc::{self, Sender},
  Mutex,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
  codegen::http::{self, request},
  transport::Server,
  Request, Response, Status,
};
use uuid::{uuid, Uuid};

#[derive(Debug, Default)]
pub struct ChessServer {}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Ongoing {
  white: Uuid,
  black: Uuid,
  game: HelperToe,
}

type PlayerJoinData = (Uuid, Sender<Result<JoinResponse, Status>>);

type LobbyData = (Uuid, Sender<Result<MakeLobbyResponse, Status>>);

#[derive(Debug, Default)]
struct GameService {
  q: Arc<Mutex<Vec<PlayerJoinData>>>,
  receivers: Arc<DashMap<Uuid, Sender<Result<SubscribeBoardResponse, Status>>>>,
  game_ids: Arc<DashMap<Uuid, Uuid>>,
  games: Arc<DashMap<Uuid, Ongoing>>,
  lobbies: Arc<DashMap<String, LobbyData>>,
}

#[tonic::async_trait]
impl Game for GameService {
  async fn move_piece(
    &self,
    request: Request<MovePieceRequest>,
  ) -> Result<Response<MovePieceResponse>, Status> {
    let request = request.into_inner();

    let uuid =
      Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

    if (!self.game_ids.contains_key(&uuid)) {
      return Err(Status::permission_denied("User needs to join a game first"));
    }

    let game = self.game_ids.get(&uuid).unwrap();

    let game_uuid = game.value();

    let mut game = self.games.get_mut(game_uuid).unwrap();

    let game = game.value_mut();

    let white = game.white;
    let black = game.black;

    let color = if (uuid == white) {
      Color::White
    } else if (uuid == black) {
      Color::Black
    } else {
      return Err(Status::internal("Something went wrong"));
    };

    let requested_board: (usize, usize) = (
      (request.board / 3).try_into().unwrap(),
      (request.board % 3).try_into().unwrap(),
    );

    if color != game.game.next {
      return Err(Status::permission_denied("You're not next"));
    }

    game
      .game
      .make_move(requested_board, &request.alg)
      .map_err(|e| Status::internal(e.to_string()))?;

    let chesses = &game.game.chesses;
    let next = game.game.next;

    let mut res = SubscribeBoardResponse {
      game: Some(TicTacToe {
        chesses: chesses
          .iter()
          .flatten()
          .map(|chess| Chess {
            end_result: Some(chess.end.clone()),
            fen: chess.to_fen(next).unwrap(),
          })
          .collect(),
        next: next as i32,
        last_move: format!("{} {}", request.board, request.alg),
      }),
      color: Color::White.into(),
      request: Some(MidGameRequest {
        draw: None,
        takeback: None,
      }),
    };

    self
      .receivers
      .get(&game.white)
      .unwrap()
      .send(Ok(res.clone()))
      .await
      .unwrap();
    res.color = Color::Black.into();
    self
      .receivers
      .get(&game.black)
      .unwrap()
      .send(Ok(res))
      .await
      .unwrap();

    Ok(Response::new(MovePieceResponse {
      successful: MoveResult::ResultSuccessful as i32,
    }))
  }

  type JoinStream = ReceiverStream<Result<JoinResponse, Status>>;

  async fn join(
    &self,
    request: Request<JoinRequest>,
  ) -> Result<Response<Self::JoinStream>, Status> {
    let uuid = Uuid::new_v4();
    let mut q = self.q.lock().await;

    if (!q.is_empty()) {
      let white = q.remove(0);

      if !white.1.is_closed() {
        let game: Ongoing = Ongoing {
          white: white.0,
          black: uuid,
          game: HelperToe::default(),
        };
  
        let game_uuid = Uuid::new_v4();
  
        self.games.insert(game_uuid, game);
  
        self.game_ids.insert(uuid, game_uuid);
  
        self.game_ids.insert(white.0, game_uuid);
  
        white
          .1
          .send(Ok(JoinResponse {
            status: GameStatus::Ready as i32,
            uuid: white.0.to_string(),
          }))
          .await
          .unwrap_or(());
  
        let (mut tx, rx) = mpsc::channel(4);
  
        tx.send(Ok(JoinResponse {
          status: GameStatus::Ready as i32,
          uuid: uuid.to_string(),
        }))
        .await
        .unwrap();
  
        tx.closed();
  
        white.1.closed();
  
        return Ok(Response::new(ReceiverStream::new(rx)));
      }
      eprintln!("Closed")
    }

    let (mut tx, rx) = mpsc::channel(4);

    q.push((uuid, tx.clone()));

    drop(q);

    tx.send(Ok(JoinResponse {
      status: GameStatus::NotReady as i32,
      uuid: uuid.to_string(),
    }))
    .await
    .unwrap();

    return Ok(Response::new(ReceiverStream::new(rx)));
  }

  type SubscribeBoardStream = ReceiverStream<Result<SubscribeBoardResponse, Status>>;

  async fn subscribe_board(
    &self,
    request: Request<SubscribeBoardRequest>,
  ) -> Result<Response<Self::SubscribeBoardStream>, Status> {
    let request = request.into_inner();

    let asker =
      Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

    if (!self.game_ids.contains_key(&asker)) {
      return Err(Status::permission_denied("User needs to join a game first"));
    }

    let (mut tx, rx) = mpsc::channel(4);

    self.receivers.insert(asker, tx.clone());

    let game = self
      .games
      .get(self.game_ids.get(&asker).unwrap().value())
      .unwrap();

    let game = game.value();

    let res: SubscribeBoardResponse = SubscribeBoardResponse {
      game: Some(TicTacToe {
        chesses: game
          .game
          .chesses
          .iter()
          .flatten()
          .map(|chess| Chess {
            end_result: Some(EndResult::None(true)),
            fen: chess.to_fen(game.game.next).unwrap(),
          })
          .collect(),
        next: game.game.next as i32,
        last_move: "".to_owned(),
      }),
      color: if (asker == game.black) {
        Color::Black as i32
      } else {
        Color::White as i32
      },
      request: None,
    };

    tx.send(Ok(res)).await.unwrap();

    Ok(Response::new(ReceiverStream::new(rx)))
  }

  async fn take_back(
    &self,
    request: Request<TakeBackRequest>,
  ) -> Result<Response<TakeBackResponse>, Status> {
    let request = request.into_inner();

    let uuid =
      Uuid::parse_str(&request.uuid).map_err(|_| Status::invalid_argument("Invalid UUID"))?;

    if let Some(game_id) = self.game_ids.get(&uuid) {
      if let Some(game) = self.games.get(game_id.value()) {}
    }

    return Err(Status::permission_denied("User is not in a game"));

    Ok(Response::new(TakeBackResponse {}))
  }

  type MakeLobbyStream = ReceiverStream<Result<MakeLobbyResponse, Status>>;

  async fn make_lobby(
    &self,
    request: Request<MakeLobbyRequest>,
  ) -> Result<Response<Self::MakeLobbyStream>, Status> {
    let (tx, rx) = mpsc::channel(2);

    let room_id = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Uuid::new_v4());

    let user_id = Uuid::new_v4();

    tx.send(Ok(MakeLobbyResponse {
      join_response: Some(JoinResponse {
        status: GameStatus::NotReady as i32,
        uuid: user_id.to_string(),
      }),
      room_id: room_id.to_string(),
    }))
    .await;

    self.lobbies.insert(room_id, (user_id, tx));

    Ok(Response::new(ReceiverStream::new(rx)))
  }
  
  type JoinLobbyStream = Self::JoinStream;

  async fn join_lobby(
    &self,
    request: Request<JoinLobbyRequest>,
  ) -> Result<Response<Self::JoinLobbyStream>, Status> {
    let req = request.into_inner();

    let lobby_code = req.code;

    if let Some((_lobby, white)) = self.lobbies.remove(&lobby_code) {
      let join_uuid = Uuid::new_v4();

      if white.1.is_closed() {
        return Err(Status::not_found("Lobby does not exist"))
      }

      let game: Ongoing = Ongoing {
        white: white.0,
        black: join_uuid,
        game: HelperToe::default(),
      };

      let game_uuid = Uuid::new_v4();

      self.games.insert(game_uuid, game);

      self.game_ids.insert(join_uuid, game_uuid);

      self.game_ids.insert(white.0, game_uuid);

      white
        .1
        .send(Ok(MakeLobbyResponse {
          room_id: "".to_owned(),
          join_response: Some(JoinResponse {
            status: GameStatus::Ready as i32,
            uuid: white.0.to_string(),
          }),
        }))
        .await
        .unwrap_or(());

      white.1.closed();

      let (tx, rx) = mpsc::channel(1);

      tx.send(Ok(JoinResponse {
        status: GameStatus::Ready as i32,
        uuid: join_uuid.to_string(),
      })).await;

      tx.closed();

      return Ok(Response::new(ReceiverStream::new(rx)));
    }

    Err(Status::not_found("Lobby doesn't exist"))
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let fasz = GameService::default();

  println!("Server listening on 0.0.0.0:50051");

  Server::builder()
    .add_service(GameServer::new(fasz))
    .serve("0.0.0.0:50051".parse()?)
    .await?;

  Ok(())
}
