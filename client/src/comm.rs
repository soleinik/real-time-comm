mod payload;

use std::sync::Arc;

use egui::Pos2;
use futures::{SinkExt, StreamExt};

pub use payload::Payload;
use tokio::sync::{mpsc::Sender, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};

const SERVER: &str = "ws://127.0.0.1:8080/ws";

//pub type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect(lines: Arc<RwLock<Vec<Vec<Pos2>>>>) -> Sender<Pos2> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Pos2>(5);

    let ws_stream = match connect_async(SERVER).await {
        Ok((stream, response)) => {
            println!("Server response was {response:?}");
            stream
        }
        Err(e) => {
            println!("WebSocket handshake failed with {e}!");
            std::process::exit(1);
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    //receive from local and send to ws
    tokio::spawn(async move {
        while let Some(canvas_pos) = rx.recv().await {
            let payload = Payload::from(&canvas_pos);

            sender
                .send(Message::binary::<Vec<u8>>(payload.into()))
                .await
                .unwrap();
        }
    });

    tokio::spawn(async move {
        while let Some(remote) = receiver.next().await {
            let Ok(msg) = remote else {
                println!("WS: receive error:{}", remote.err().unwrap());
                continue;
            };
            match msg {
                Message::Text(t) => println!("WS::text - {t}"),
                Message::Binary(b) => {
                    //lines
                    let canvas_pos: Pos2 = Payload::from(&b).into();

                    //poisoned lock
                    let mut l = lines.write().await;

                    if canvas_pos == super::NULL_POS {
                        l.push(vec![]);
                        continue;
                    }

                    let current_line = l.last_mut().unwrap();

                    if current_line.last() != Some(&canvas_pos) {
                        current_line.push(canvas_pos);
                    }
                }
                Message::Ping(b) => println!("WS::ping - {b:?}"),
                Message::Pong(b) => println!("WS::pong - {b:?}"),
                Message::Close(b) => println!("WS::close - {b:?}"),
                Message::Frame(f) => println!("WS::frame - {f:?}"),
            }
        }
    });
    tx
}
