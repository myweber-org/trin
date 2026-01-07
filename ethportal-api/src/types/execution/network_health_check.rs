use std::net::{TcpStream, IpAddr};
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
        let output = std::process::Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg(self.timeout.as_secs().to_string())
            .arg(host)
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    pub fn check_port(&self, host: &str, port: u16) -> bool {
        let addr = format!("{}:{}", host, port);
        match TcpStream::connect_timeout(&addr.parse().unwrap(), self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn scan_ports(&self, host: &str, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = Vec::new();

        for port in start_port..=end_port {
            let host = host.to_string();
            let checker = self.timeout;
            let handle = thread::spawn(move || {
                let addr = format!("{}:{}", host, port);
                match TcpStream::connect_timeout(&addr.parse().unwrap(), checker) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_check_creation() {
        let checker = NetworkCheck::new(5);
        assert_eq!(checker.timeout.as_secs(), 5);
    }
}