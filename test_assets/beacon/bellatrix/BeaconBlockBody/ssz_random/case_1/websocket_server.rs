use futures_util::{SinkExt, StreamExt};
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
                println!("Client initiated close");
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
}use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: std::net::TcpStream) {
    let mut buffer = [0; 1024];
    let mut handshake_done = false;

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                if !handshake_done {
                    let request = String::from_utf8_lossy(&buffer[..size]);
                    if request.contains("Upgrade: websocket") {
                        let response = "HTTP/1.1 101 Switching Protocols\r\n\
                                       Upgrade: websocket\r\n\
                                       Connection: Upgrade\r\n\
                                       Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                        handshake_done = true;
                    }
                } else {
                    let response_frame = create_echo_frame(&buffer[..size]);
                    stream.write_all(&response_frame).unwrap();
                }
            }
            _ => break,
        }
    }
}

fn create_echo_frame(data: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(0x81);
    frame.push(data.len() as u8);
    frame.extend_from_slice(data);
    frame
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}