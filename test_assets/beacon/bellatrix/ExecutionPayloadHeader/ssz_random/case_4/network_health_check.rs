use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;

pub struct NetworkChecker {
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkChecker {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_host(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Connection failed to {}:{} - {}", host, port, e);
                Ok(false)
            }
        }
    }

    pub fn check_multiple_hosts(&self, hosts: &[(&str, u16)]) -> Vec<(&str, bool)> {
        hosts.iter()
            .map(|&(host, port)| (host, self.check_host(host, port).unwrap_or(false)))
            .collect()
    }
}

pub fn run_health_check() {
    let checker = NetworkChecker::new(5);
    let targets = [
        ("google.com", 80),
        ("github.com", 443),
        ("example.com", 8080),
    ];

    println!("Starting network health check...");
    let results = checker.check_multiple_hosts(&targets);

    for (host, status) in results {
        println!("{}: {}", host, if status { "✓ Reachable" } else { "✗ Unreachable" });
    }

    let all_healthy = results.iter().all(|(_, status)| *status);
    if all_healthy {
        println!("All hosts are reachable");
    } else {
        println!("Some hosts are unreachable");
    }
}