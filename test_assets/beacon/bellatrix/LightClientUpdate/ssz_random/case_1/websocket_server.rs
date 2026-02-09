use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection from: {}", addr);
    let ws_stream = match accept_async(raw_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                println!("Received message from {}: {:?}", addr, msg);
                if msg.is_text() || msg.is_binary() {
                    if let Err(e) = write.send(msg).await {
                        eprintln!("Error sending message to {}: {}", addr, e);
                        break;
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message from {}: {}", addr, e);
                break;
            }
        }
    }

    println!("WebSocket connection closed: {}", addr);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }

    Ok(())
}