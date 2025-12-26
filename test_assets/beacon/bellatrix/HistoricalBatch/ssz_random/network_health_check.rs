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

    pub fn ping_host(&self, host: IpAddr) -> bool {
        let output = std::process::Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg(self.timeout.as_secs().to_string())
            .arg(host.to_string())
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    pub fn check_port(&self, host: IpAddr, port: u16) -> bool {
        match TcpStream::connect_timeout(&(host, port).into(), self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn scan_ports(&self, host: IpAddr, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = vec![];

        for port in start_port..=end_port {
            let timeout = self.timeout;
            let host_clone = host;
            let handle = thread::spawn(move || {
                match TcpStream::connect_timeout(&(host_clone, port).into(), timeout) {
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
    use std::net::Ipv4Addr;

    #[test]
    fn test_port_check() {
        let checker = NetworkCheck::new(2);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Port 80 should be closed on localhost (unless running web server)
        assert!(!checker.check_port(localhost, 80));
    }

    #[test]
    fn test_scan_ports() {
        let checker = NetworkCheck::new(1);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let open_ports = checker.scan_ports(localhost, 80, 85);
        
        // Should return empty vector or specific ports depending on system
        println!("Open ports on localhost: {:?}", open_ports);
    }
}