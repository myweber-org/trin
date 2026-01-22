
use std::net::{TcpStream, ToSocketAddrs};
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

    pub fn ping_host(&self, host: &str) -> io::Result<bool> {
        let addr = format!("{}:80", host)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid host"))?;

        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn check_port(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr = format!("{}:{}", host, port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid host:port"))?;

        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn scan_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports.iter()
            .map(|&port| (port, self.check_port(host, port).unwrap_or(false)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connectivity() {
        let checker = NetworkCheck::new(2);
        let result = checker.ping_host("localhost");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_host() {
        let checker = NetworkCheck::new(1);
        let result = checker.ping_host("invalid.host.that.does.not.exist");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}