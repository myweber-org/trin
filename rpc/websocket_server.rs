use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    let (tx, _) = broadcast::channel::<String>(32);
    let tx = Arc::new(tx);

    while let Ok((stream, addr)) = listener.accept().await {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        println!("New connection from: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let send_task = tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    if ws_sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            });

            let recv_task = tokio::spawn(async move {
                while let Some(Ok(msg)) = ws_receiver.next().await {
                    if let Message::Text(text) = msg {
                        println!("Received: {}", text);
                        let _ = tx.send(text);
                    }
                }
            });

            tokio::select! {
                _ = send_task => {},
                _ = recv_task => {},
            }

            println!("Connection closed: {}", addr);
        });
    }

    Ok(())
}