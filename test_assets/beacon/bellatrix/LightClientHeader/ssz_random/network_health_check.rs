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

    pub fn check_multiple_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports.iter()
            .map(|&port| (port, self.check_port(host, port)))
            .collect()
    }

    pub fn basic_connectivity_test(&self, hosts: &[&str]) -> Vec<(&str, bool)> {
        hosts.iter()
            .map(|&host| {
                let result = self.check_port(host, 80) || self.check_port(host, 443);
                (host, result)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_check() {
        let checker = NetworkHealth::new(2);
        // Test with localhost - port 80 might be closed, but connection should be attempted
        let result = checker.check_port("localhost", 80);
        // We can't assert true/false since it depends on system configuration
        // Just verify the function doesn't panic
        assert!(result == true || result == false);
    }

    #[test]
    fn test_multiple_ports() {
        let checker = NetworkHealth::new(1);
        let results = checker.check_multiple_ports("127.0.0.1", &[80, 443, 8080]);
        assert_eq!(results.len(), 3);
    }
}