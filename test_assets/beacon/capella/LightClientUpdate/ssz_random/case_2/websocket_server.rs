use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

pub async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        let msg = msg?;
        if msg.is_close() {
            break;
        }
        if msg.is_text() || msg.is_binary() {
            sender.send(msg).await?;
        }
    }
    Ok(())
}

pub async fn run_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run_server("127.0.0.1:8080").await
}