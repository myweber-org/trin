use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::io;

pub struct NetworkProbe;

impl NetworkProbe {
    pub fn ping_host(host: &str, timeout_secs: u64) -> io::Result<bool> {
        let addr = format!("{}:0", host);
        let socket_addrs = addr.to_socket_addrs()?;
        
        for socket_addr in socket_addrs {
            match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(timeout_secs)) {
                Ok(_) => return Ok(true),
                Err(_) => continue,
            }
        }
        Ok(false)
    }

    pub fn check_port(host: &str, port: u16, timeout_secs: u64) -> io::Result<bool> {
        let addr = format!("{}:{}", host, port);
        let socket_addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid host or port")
        })?;

        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(timeout_secs)) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.kind() == io::ErrorKind::TimedOut {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn scan_common_ports(host: &str) -> Vec<u16> {
        let common_ports = [80, 443, 22, 21, 25, 53, 3306, 5432, 8080];
        let mut open_ports = Vec::new();

        for &port in &common_ports {
            if Self::check_port(host, port, 2).unwrap_or(false) {
                open_ports.push(port);
            }
        }
        open_ports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_localhost() {
        let result = NetworkProbe::ping_host("localhost", 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_common_ports() {
        let open_ports = NetworkProbe::scan_common_ports("127.0.0.1");
        println!("Open ports on localhost: {:?}", open_ports);
    }
}