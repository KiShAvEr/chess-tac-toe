use std::sync::RwLock;
use tonic::transport::Channel;
use helpers::chesstactoe::game_client::GameClient;

static CLIENT: RwLock<Option<GameClient<Channel>>> = RwLock::new(None);

static UUID: RwLock<Option<String>> = RwLock::new(None);

pub fn get_client() -> Option<GameClient<Channel>> {
  return CLIENT.read().unwrap().clone();
}

pub async fn connect(url: String) -> Result<(), Box<dyn std::error::Error>> {
  GameClient::connect(url).await.and_then(|client| {
    Ok(match CLIENT.write() {
        Ok(mut e) => {e.replace(client);},
        Err(a) => println!("{:?}", a),
    })
  })?;
  Ok(())
}

pub fn set_uuid(new: &str) {
  UUID.write().unwrap().replace(new.to_owned());
}

pub fn get_uuid() -> Option<String> {
  return UUID.read().unwrap().clone();
}