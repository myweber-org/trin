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

    pub fn check_port(&self, host: &str, port: u16) -> bool {
        let addr_string = format!("{}:{}", host, port);
        
        match addr_string.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    match TcpStream::connect_timeout(&addr, self.timeout) {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub fn check_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports
            .iter()
            .map(|&port| (port, self.check_port(host, port)))
            .collect()
    }

    pub fn check_multiple_hosts(&self, hosts: &[(&str, u16)]) -> Vec<(&str, u16, bool)> {
        hosts
            .iter()
            .map(|&(host, port)| (host, port, self.check_port(host, port)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_port() {
        let checker = NetworkHealth::new(2);
        // Port 80 likely closed on localhost
        assert!(!checker.check_port("localhost", 80));
    }

    #[test]
    fn test_multiple_ports() {
        let checker = NetworkHealth::new(2);
        let results = checker.check_ports("127.0.0.1", &[80, 443, 8080]);
        assert_eq!(results.len(), 3);
    }
}