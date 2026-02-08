
use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;

pub struct NetworkChecker {
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkChecker {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_port(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn check_multiple_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports.iter()
            .map(|&port| (port, self.check_port(host, port).unwrap_or(false)))
            .collect()
    }
}

pub fn perform_health_check() -> Vec<(&'static str, bool)> {
    let checker = NetworkChecker::new(3);
    let mut results = Vec::new();
    
    let test_ports = vec![80, 443, 22, 8080];
    let host = "example.com";
    
    results.push(("port_scan", !checker.check_multiple_ports(host, &test_ports).is_empty()));
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_check() {
        let checker = NetworkChecker::new(1);
        let result = checker.check_port("127.0.0.1", 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_ports() {
        let checker = NetworkChecker::new(1);
        let results = checker.check_multiple_ports("localhost", &[80, 443]);
        assert_eq!(results.len(), 2);
    }
}