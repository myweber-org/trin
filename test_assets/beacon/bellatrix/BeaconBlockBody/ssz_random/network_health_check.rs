use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;

pub struct NetworkCheck {
    timeout: Duration,
}

impl NetworkCheck {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkCheck {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn ping_host(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn check_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports.iter()
            .map(|&port| (port, self.ping_host(host, port).unwrap_or(false)))
            .collect()
    }
}

pub fn scan_common_ports(host: &str) -> Vec<(u16, bool)> {
    let checker = NetworkCheck::new(3);
    let common_ports = [80, 443, 22, 21, 25, 53, 3306, 5432, 8080];
    checker.check_ports(host, &common_ports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_scan() {
        let results = scan_common_ports("127.0.0.1");
        assert!(!results.is_empty());
        
        for (port, status) in results {
            println!("Port {}: {}", port, if status { "open" } else { "closed" });
        }
    }
}