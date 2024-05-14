extern crate core;
#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket::futures::task::Spawn;
use rocket::tokio::sync::broadcast::channel;

use crate::websocket::{feedback_route, WsMessage};

mod websocket;

#[launch]
async fn rocket() -> _ {

    let sender = channel::<WsMessage>(1024).0;

    rocket::build().mount("/api", routes![ feedback_route])
        .manage(sender)
        .mount("/", FileServer::from("."))
}
