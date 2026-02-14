use std::net::TcpStream;
use std::time::Duration;
use std::thread;

const HOSTS: &[&str] = &["google.com:80", "github.com:443", "cloudflare.com:80"];
const MAX_RETRIES: u32 = 3;
const TIMEOUT_SECS: u64 = 5;

fn test_connection(host: &str) -> bool {
    for attempt in 1..=MAX_RETRIES {
        match TcpStream::connect_timeout(
            &host.parse().unwrap(),
            Duration::from_secs(TIMEOUT_SECS)
        ) {
            Ok(_) => {
                println!("✓ {} connected (attempt {})", host, attempt);
                return true;
            }
            Err(e) => {
                println!("✗ {} failed attempt {}: {}", host, attempt, e);
                if attempt < MAX_RETRIES {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }
    false
}

fn main() {
    println!("Network Health Check");
    println!("Testing {} endpoints...\n", HOSTS.len());

    let mut successful = 0;
    for host in HOSTS {
        if test_connection(host) {
            successful += 1;
        }
    }

    println!("\nResults: {}/{} endpoints reachable", successful, HOSTS.len());
    if successful == HOSTS.len() {
        println!("Network connectivity: OK");
    } else if successful > 0 {
        println!("Network connectivity: PARTIAL");
    } else {
        println!("Network connectivity: FAILED");
    }
}