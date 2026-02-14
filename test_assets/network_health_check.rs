use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct NetworkHealth {
    timeout: Duration,
}

impl NetworkHealth {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkHealth {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_port(&self, host: &str, port: u16) -> Result<bool, String> {
        let addr_string = format!("{}:{}", host, port);
        let addrs: Vec<_> = addr_string.to_socket_addrs()
            .map_err(|e| format!("DNS resolution failed: {}", e))?
            .collect();

        if addrs.is_empty() {
            return Err("No addresses resolved".to_string());
        }

        for addr in addrs {
            match TcpStream::connect_timeout(&addr, self.timeout) {
                Ok(_) => return Ok(true),
                Err(_) => continue,
            }
        }
        Ok(false)
    }

    pub fn check_multiple_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports.iter()
            .map(|&port| (port, self.check_port(host, port).unwrap_or(false)))
            .collect()
    }

    pub fn validate_host(&self, host: &str) -> bool {
        let test_port = 80;
        self.check_port(host, test_port).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connection() {
        let checker = NetworkHealth::new(2);
        let result = checker.check_port("localhost", 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_host() {
        let checker = NetworkHealth::new(1);
        let result = checker.check_port("nonexistent.invalid", 80);
        assert!(result.is_err());
    }
}