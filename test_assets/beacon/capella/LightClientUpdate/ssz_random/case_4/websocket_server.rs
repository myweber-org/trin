use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Ok(ws_stream) = accept_async(stream).await {
                let (mut write, mut read) = ws_stream.split();
                while let Some(Ok(msg)) = read.next().await {
                    if let Message::Text(text) = msg {
                        if let Err(e) = write.send(Message::Text(text)).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }
    Ok(())
}