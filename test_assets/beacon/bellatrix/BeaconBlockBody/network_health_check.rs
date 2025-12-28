use std::net::TcpStream;
use std::time::Duration;
use std::thread;

const HOSTS: [&str; 3] = ["8.8.8.8:53", "1.1.1.1:53", "208.67.222.222:53"];
const TIMEOUT_MS: u64 = 2000;
const MAX_RETRIES: u8 = 2;

fn test_connection(host: &str) -> bool {
    for attempt in 1..=MAX_RETRIES {
        match TcpStream::connect_timeout(
            &host.parse().unwrap(),
            Duration::from_millis(TIMEOUT_MS)
        ) {
            Ok(_) => {
                println!("✓ {} connected (attempt {})", host, attempt);
                return true;
            }
            Err(e) => {
                println!("✗ {} failed attempt {}: {}", host, attempt, e);
                if attempt < MAX_RETRIES {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }
    false
}

fn main() {
    println!("Testing network connectivity to DNS servers...");
    
    let mut healthy_connections = 0;
    for host in HOSTS.iter() {
        if test_connection(host) {
            healthy_connections += 1;
        }
    }
    
    println!("\nResults: {}/{} connections successful", 
             healthy_connections, 
             HOSTS.len());
    
    if healthy_connections >= 2 {
        println!("Network status: HEALTHY");
    } else if healthy_connections == 1 {
        println!("Network status: DEGRADED");
    } else {
        println!("Network status: OFFLINE");
    }
}