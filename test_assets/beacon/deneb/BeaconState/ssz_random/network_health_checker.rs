use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

pub struct NetworkChecker {
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(timeout_seconds: u64) -> Self {
        NetworkChecker {
            timeout: Duration::from_secs(timeout_seconds),
        }
    }

    pub fn ping_host(&self, host: IpAddr) -> bool {
        match host {
            IpAddr::V4(ipv4) => self.ping_ipv4(ipv4),
            IpAddr::V6(_) => false, // IPv6 ping not implemented
        }
    }

    fn ping_ipv4(&self, ip: Ipv4Addr) -> bool {
        // Simple ICMP simulation using TCP connection attempt
        let socket_addr = SocketAddr::new(IpAddr::V4(ip), 80);
        TcpStream::connect_timeout(&socket_addr, self.timeout).is_ok()
    }

    pub fn check_port(&self, host: IpAddr, port: u16) -> bool {
        let socket_addr = SocketAddr::new(host, port);
        TcpStream::connect_timeout(&socket_addr, self.timeout).is_ok()
    }

    pub fn scan_common_ports(&self, host: IpAddr) -> Vec<u16> {
        let common_ports = [21, 22, 23, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995];
        
        common_ports
            .iter()
            .filter(|&&port| self.check_port(host, port))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_network_checker_creation() {
        let checker = NetworkChecker::new(5);
        assert_eq!(checker.timeout.as_secs(), 5);
    }

    #[test]
    fn test_port_check() {
        let checker = NetworkChecker::new(2);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Port 80 might be closed on localhost
        let result = checker.check_port(localhost, 80);
        println!("Port 80 open on localhost: {}", result);
    }
}