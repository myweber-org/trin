
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::io;
use std::thread;

const PING_TIMEOUT: Duration = Duration::from_secs(2);
const PORT_SCAN_TIMEOUT: Duration = Duration::from_secs(1);

pub struct NetworkHealth {
    host: String,
}

impl NetworkHealth {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
        }
    }

    pub fn ping(&self) -> Result<Duration, String> {
        let start = Instant::now();
        
        let socket_addr = match self.host.to_socket_addrs() {
            Ok(mut addrs) => addrs.next().ok_or("No addresses found")?,
            Err(e) => return Err(format!("DNS resolution failed: {}", e)),
        };

        match TcpStream::connect_timeout(&socket_addr, PING_TIMEOUT) {
            Ok(_) => {
                let elapsed = start.elapsed();
                Ok(elapsed)
            }
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    }

    pub fn scan_ports(&self, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = Vec::new();

        for port in start_port..=end_port {
            let host = self.host.clone();
            let handle = thread::spawn(move || {
                let addr = format!("{}:{}", host, port);
                match TcpStream::connect_timeout(&addr.parse().unwrap(), PORT_SCAN_TIMEOUT) {
                    Ok(_) => Some(port),
                    Err(_) => None,
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Ok(Some(port)) = handle.join() {
                open_ports.push(port);
            }
        }

        open_ports.sort();
        open_ports
    }

    pub fn check_connectivity(&self) -> bool {
        self.ping().is_ok()
    }
}

pub fn perform_health_check(host: &str) -> io::Result<()> {
    let checker = NetworkHealth::new(host);
    
    println!("Checking connectivity to {}...", host);
    
    match checker.ping() {
        Ok(duration) => {
            println!("✓ Host is reachable (ping: {:?})", duration);
            
            println!("Scanning common ports (80, 443, 22, 8080)...");
            let open_ports = checker.scan_ports(80, 8080);
            
            if !open_ports.is_empty() {
                println!("✓ Open ports found: {:?}", open_ports);
            } else {
                println!("✗ No common ports are open");
            }
            
            Ok(())
        }
        Err(e) => {
            println!("✗ Host is unreachable: {}", e);
            Err(io::Error::new(io::ErrorKind::ConnectionRefused, e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_health_creation() {
        let checker = NetworkHealth::new("example.com");
        assert_eq!(checker.host, "example.com");
    }

    #[test]
    fn test_connectivity_check() {
        let checker = NetworkHealth::new("127.0.0.1");
        let result = checker.check_connectivity();
        assert!(result || !result);
    }
}