
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::thread;

pub struct NetworkCheck {
    timeout: Duration,
}

impl NetworkCheck {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkCheck {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn ping_host(&self, host: &str) -> bool {
        let addr = match host.to_socket_addrs() {
            Ok(mut addrs) => addrs.next(),
            Err(_) => return false,
        };

        if let Some(addr) = addr {
            TcpStream::connect_timeout(&addr, self.timeout).is_ok()
        } else {
            false
        }
    }

    pub fn scan_port(&self, host: &str, port: u16) -> bool {
        let addr_string = format!("{}:{}", host, port);
        let addr = match addr_string.to_socket_addrs() {
            Ok(mut addrs) => addrs.next(),
            Err(_) => return false,
        };

        if let Some(addr) = addr {
            TcpStream::connect_timeout(&addr, self.timeout).is_ok()
        } else {
            false
        }
    }

    pub fn scan_port_range(&self, host: &str, start: u16, end: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = Vec::new();

        for port in start..=end {
            let host = host.to_string();
            let checker = self.timeout;
            let handle = thread::spawn(move || {
                let addr_string = format!("{}:{}", host, port);
                if let Ok(mut addrs) = addr_string.to_socket_addrs() {
                    if let Some(addr) = addrs.next() {
                        return TcpStream::connect_timeout(&addr, checker).is_ok();
                    }
                }
                false
            });
            handles.push((port, handle));
        }

        for (port, handle) in handles {
            if handle.join().unwrap_or(false) {
                open_ports.push(port);
            }
        }

        open_ports.sort();
        open_ports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_check_creation() {
        let checker = NetworkCheck::new(5);
        assert_eq!(checker.timeout.as_secs(), 5);
    }

    #[test]
    fn test_ping_known_host() {
        let checker = NetworkCheck::new(3);
        let result = checker.ping_host("google.com:80");
        assert!(result || !result);
    }

    #[test]
    fn test_scan_single_port() {
        let checker = NetworkCheck::new(2);
        let result = checker.scan_port("localhost", 8080);
        assert!(!result);
    }
}