use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("New connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during WebSocket handshake");

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message from {}: {}", addr, text);
                let echo_msg = Message::Text(format!("Echo: {}", text));
                if let Err(e) = sender.send(echo_msg).await {
                    eprintln!("Error sending echo message to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client {} disconnected", addr);
                break;
            }
            Err(e) => {
                eprintln!("Error receiving message from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind address");
    println!("WebSocket echo server listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) = write.send(msg).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}