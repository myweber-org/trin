use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;

type Connections = Arc<Mutex<HashMap<Uuid, tokio_tungstenite::WebSocketStream<TcpStream>>>>;

async fn handle_connection(stream: TcpStream, connections: Connections) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to accept WebSocket connection: {}", e);
            return;
        }
    };

    let client_id = Uuid::new_v4();
    println!("New client connected: {}", client_id);

    {
        let mut conns = connections.lock().unwrap();
        conns.insert(client_id, ws_stream);
    }

    let (mut sender, mut receiver) = {
        let mut conns = connections.lock().unwrap();
        conns.remove(&client_id).unwrap().split()
    };

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                println!("Received from {}: {}", client_id, text);
                let response = format!("Echo: {}", text);
                if let Err(e) = sender.send(Message::Text(response)).await {
                    eprintln!("Failed to send message to {}: {}", client_id, e);
                    break;
                }
            }
            Message::Close(_) => {
                println!("Client {} requested close", client_id);
                break;
            }
            _ => {}
        }
    }

    {
        let mut conns = connections.lock().unwrap();
        conns.remove(&client_id);
    }
    println!("Client {} disconnected", client_id);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    let connections: Connections = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        let conns = connections.clone();
        tokio::spawn(async move {
            handle_connection(stream, conns).await;
        });
    }
}