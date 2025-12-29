use std::net::TcpStream;
use std::time::Duration;
use std::thread;

const HOST: &str = "example.com";
const PORT: u16 = 80;
const MAX_RETRIES: u8 = 3;
const TIMEOUT_SECS: u64 = 5;

fn test_connection(host: &str, port: u16) -> bool {
    match TcpStream::connect((host, port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    let mut attempts = 0;
    
    while attempts < MAX_RETRIES {
        println!("Attempt {} of {} to connect to {}:{}", 
                 attempts + 1, MAX_RETRIES, HOST, PORT);
        
        if test_connection(HOST, PORT) {
            println!("Connection successful!");
            return;
        }
        
        attempts += 1;
        
        if attempts < MAX_RETRIES {
            println!("Connection failed. Retrying in {} seconds...", TIMEOUT_SECS);
            thread::sleep(Duration::from_secs(TIMEOUT_SECS));
        }
    }
    
    println!("Failed to establish connection after {} attempts", MAX_RETRIES);
}