use dioxus::prelude::*;
use utils::{connect, get_client, get_uuid, set_uuid};
use pages::MainScreen::MainScreen;
use components::{};
use helpers::chesstactoe::{JoinRequest};
use tonic::{Request};

mod components;
mod pages;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    connect("http://[::1]:50051".to_owned()).await;
    dioxus::desktop::launch(app);
    
    Ok(())
    
}

fn app(cx: Scope) -> Element {

    let uuid = use_state(&cx, || "".to_owned());
    let set_state = uuid.setter();

    use_future(&cx, (),|_| async move {
        let client = get_client();

        if client.is_none() {
            return;
        }

        let mut client = client.unwrap();

        let mut res = client.join(Request::new(JoinRequest {  })).await.unwrap().into_inner();

        while let Some(i) = res.message().await.unwrap() {
            println!("{:?}", i);
            set_uuid(&i.uuid);
            set_state(i.uuid);
        }
    });
    
    let a = uuid.get();

    cx.render(
        rsx!{
            Router {
                Route { to: "/", MainScreen {}}
            }
        }
    )
}