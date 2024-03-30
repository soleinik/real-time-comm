use std::{error::Error, net::SocketAddr};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    response::IntoResponse,
    routing::get,
    Router,
};

use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    //let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // build our application with some routes
    let app = Router::new()
        //.fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    //let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match &msg {
                axum::extract::ws::Message::Text(t) => {
                    println!("Text:{t}");
                    if let Err(e) = socket.send(Message::Text(t.to_owned())).await {
                        println!("Text: reply to {who} error:{e}");
                        return;
                    }
                }
                axum::extract::ws::Message::Binary(b) => {
                    println!("Binary:{b:?}");

                    if let Err(e) = socket.send(Message::Binary(b.to_owned())).await {
                        println!("Binary: reply to {who} error:{e}");
                        return;
                    }
                }
                axum::extract::ws::Message::Ping(p) => {
                    println!("Ping:{p:?}");
                    if let Err(e) = socket.send(Message::Pong(p.to_owned())).await {
                        println!("Ping: reply to {who} error:{e}");
                        return;
                    }
                }
                axum::extract::ws::Message::Pong(p) => {
                    println!("Pong:{p:?}");
                    // if let Err(e) = socket.send(Message::Ping(p.to_owned())).await {
                    //     println!("Pong: reply to {who} error:{e}");
                    //     return;
                    // }
                }
                axum::extract::ws::Message::Close(c) => {
                    if let Some(cf) = c {
                        println!(
                            ">>> {} sent close with code {} and reason `{}`",
                            who, cf.code, cf.reason
                        );
                    } else {
                        println!(">>> {who} somehow sent close message without CloseFrame");
                    }
                }
            }
        } else {
            println!("client {who} abruptly disconnected");
            return;
        }
    }
}
