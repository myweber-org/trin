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
}use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug)]
struct HostStatus {
    host: String,
    port: u16,
    reachable: bool,
    latency: Option<Duration>,
}

fn check_host(host: &str, port: u16, timeout: Duration) -> HostStatus {
    let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
        Ok(addr) => addr,
        Err(_) => {
            return HostStatus {
                host: host.to_string(),
                port,
                reachable: false,
                latency: None,
            }
        }
    };

    let start = std::time::Instant::now();
    let reachable = TcpStream::connect_timeout(&addr, timeout).is_ok();
    let elapsed = start.elapsed();

    HostStatus {
        host: host.to_string(),
        port,
        reachable,
        latency: if reachable { Some(elapsed) } else { None },
    }
}

fn main() {
    let hosts_to_check = vec![
        ("google.com", 80),
        ("github.com", 443),
        ("example.invalid", 80),
        ("localhost", 8080),
    ];

    let timeout = Duration::from_secs(3);
    let mut results = Vec::new();

    for (host, port) in hosts_to_check {
        let status = check_host(host, port, timeout);
        results.push(status);
    }

    println!("Network Health Check Results:");
    println!("{:<25} {:<8} {:<12} {:<10}", "Host", "Port", "Reachable", "Latency (ms)");
    println!("{}", "-".repeat(60));

    for status in results {
        let latency_str = match status.latency {
            Some(dur) => format!("{:.2}", dur.as_millis()),
            None => "N/A".to_string(),
        };
        println!(
            "{:<25} {:<8} {:<12} {:<10}",
            status.host,
            status.port,
            status.reachable,
            latency_str
        );
    }
}