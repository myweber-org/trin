use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use log::{info, error};

pub async fn run_websocket_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on: {}", addr);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, peer_addr));
    }
    Ok(())
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    info!("New WebSocket connection from: {}", addr);
    let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to establish WebSocket connection with {}: {}", addr, e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                info!("Received text from {}: {}", addr, text);
                if let Err(e) = write.send(Message::Text(text)).await {
                    error!("Error sending echo to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} requested connection close", addr);
                break;
            }
            Ok(_) => {
                info!("Received non-text message from {}", addr);
            }
            Err(e) => {
                error!("Error reading message from {}: {}", addr, e);
                break;
            }
        }
    }
    info!("Connection with {} closed", addr);
}