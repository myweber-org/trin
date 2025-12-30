use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

const MAX_RETRIES: u32 = 3;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const TARGET_HOST: &str = "example.com";
const TARGET_PORT: u16 = 80;

fn test_connection(addr: SocketAddr) -> io::Result<()> {
    match TcpStream::connect_timeout(&addr, CONNECTION_TIMEOUT) {
        Ok(mut stream) => {
            let request = format!("GET / HTTP/1.1\r\nHost: {}\r\n\r\n", TARGET_HOST);
            stream.write_all(request.as_bytes())?;
            println!("Successfully connected to {}", addr);
            Ok(())
        }
        Err(e) => {
            eprintln!("Connection failed: {}", e);
            Err(e)
        }
    }
}

fn main() -> io::Result<()> {
    let addr = SocketAddr::from(([93, 184, 216, 34], TARGET_PORT));
    let mut retries = 0;
    let start_time = Instant::now();

    while retries < MAX_RETRIES {
        println!("Attempt {} to connect to {}...", retries + 1, TARGET_HOST);
        
        match test_connection(addr) {
            Ok(_) => {
                let elapsed = start_time.elapsed();
                println!("Network test completed successfully in {:?}", elapsed);
                return Ok(());
            }
            Err(_) => {
                retries += 1;
                if retries < MAX_RETRIES {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    eprintln!("Failed to establish connection after {} attempts", MAX_RETRIES);
    Err(io::Error::new(io::ErrorKind::TimedOut, "Connection attempts exhausted"))
}