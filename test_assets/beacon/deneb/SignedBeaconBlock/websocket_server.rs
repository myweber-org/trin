use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use log::{info, error};

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    info!("New WebSocket connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Failed to accept WebSocket connection");

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(message) = receiver.next().await {
        match message {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        info!("Received text message from {}: {}", addr, text);
                        let echo_msg = Message::Text(format!("Echo: {}", text));
                        if let Err(e) = sender.send(echo_msg).await {
                            error!("Failed to send echo message to {}: {}", addr, e);
                            break;
                        }
                    }
                    Message::Close(_) => {
                        info!("Client {} requested close", addr);
                        break;
                    }
                    _ => {
                        info!("Received non-text message from {}, ignoring", addr);
                    }
                }
            }
            Err(e) => {
                error!("Error receiving message from {}: {}", addr, e);
                break;
            }
        }
    }
    info!("Connection closed for client: {}", addr);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind address");
    info!("WebSocket echo server listening on: {}", addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, client_addr));
    }
}use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

async fn handle_connection(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                if let Err(e) = write.send(Message::Text(text)).await {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client closed connection");
                break;
            }
            Err(e) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("WebSocket echo server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}