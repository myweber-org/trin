
use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::process::Command;

pub struct NetworkChecker;

impl NetworkChecker {
    pub fn ping_host(host: &str) -> bool {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", "ping", "-n", "1", "-w", "1000", host])
                .output()
        } else {
            Command::new("ping")
                .args(&["-c", "1", "-W", "1", host])
                .output()
        };

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    pub fn check_port(addr: &str, port: u16, timeout_ms: u64) -> bool {
        let socket_addr: SocketAddr = match format!("{}:{}", addr, port).parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(timeout_ms)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn perform_health_check(host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        let mut results = Vec::new();
        
        for &port in ports {
            let status = Self::check_port(host, port, 2000);
            results.push((port, status));
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_check_localhost() {
        let result = NetworkChecker::check_port("127.0.0.1", 80, 100);
        assert!(!result);
    }

    #[test]
    fn test_health_check_empty() {
        let results = NetworkChecker::perform_health_check("127.0.0.1", &[]);
        assert_eq!(results.len(), 0);
    }
}