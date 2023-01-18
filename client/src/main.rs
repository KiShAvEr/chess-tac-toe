#![allow(non_snake_case)]

use std::future::Future;

use dioxus::prelude::*;
use dioxus_router::{Router, Route};
use utils::{connect, set_uuid, get_client};
use pages::MainScreen::MainScreen;
use components::{};
use helpers::chesstactoe::{JoinRequest};
use tonic::{Request, Status};

mod components;
mod pages;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    connect("http://31.21.202.217:50051".to_owned()).await.unwrap();
    dioxus_desktop::launch(app);
    
    Ok(())
    
}

fn app(cx: Scope) -> Element {
    cx.render(
        rsx!{
            Router {
                Route { to: "/", MainScreen {}},
            }
        }
    )
}