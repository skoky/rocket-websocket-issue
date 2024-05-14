use std::process::exit;
use rocket::fs::{FileServer, relative};
use rocket::futures::{SinkExt, StreamExt};
use rocket::response::stream::{Event, EventStream, TextStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::serde_json;
use rocket::Shutdown;
use rocket::State;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use tokio::sync::broadcast;
use ws::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
pub(crate) struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,   // TODO use enums
    pub event: String,
}

impl WsMessage {
    pub fn command(value: &str) -> Self {
        WsMessage{ msg_type: "COMMAND".to_string(), event: value.to_string() }
    }

}

#[get("/feedback")]
pub(crate) fn feedback_route(ws: ws::WebSocket, queue: &State<Sender<WsMessage>>, mut end: Shutdown) -> ws::Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};
    let mut rx = queue.subscribe();

    ws.channel(move |mut stream| Box::pin(async move {
        let (mut ws_sink, mut ws_stream) = stream.split();

        loop {
            select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => {
                            match serde_json::to_string(&msg) {
                                Ok(txt) => {
                                    let _ = ws_sink.send(Message::Text(txt)).await;
                                },
                                Err(e) => eprintln!("WS Error {}",e)
                            }
                        },
                    Err(e) => {
                            eprintln!("Can get msg from queue {}", e)
                        }
                },
                msg = ws_stream.next() => match msg {
                    Some(msg) => match msg {
                            Ok(msg) if msg.to_string() == "ping".to_string() => {
                                let _ = ws_sink.send(Message::Text("pong".to_string())).await;
                            },
                            _ => {}
                        },
                    _ => {}
                },
                _ = &mut end => {
                        break
                    }
            }
        }

        Ok(())
    }))
}

