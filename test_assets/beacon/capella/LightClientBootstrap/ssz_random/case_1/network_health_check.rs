use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::io::{self, Write};

pub struct NetworkCheckResult {
    pub host: String,
    pub ping_ms: Option<u128>,
    pub ports_open: Vec<u16>,
    pub check_time: Duration,
}

pub fn check_host_connectivity(host: &str, ports: &[u16], timeout_secs: u64) -> io::Result<NetworkCheckResult> {
    let timeout = Duration::from_secs(timeout_secs);
    let start_time = Instant::now();
    
    let mut result = NetworkCheckResult {
        host: host.to_string(),
        ping_ms: None,
        ports_open: Vec::new(),
        check_time: Duration::default(),
    };
    
    let ping_start = Instant::now();
    if let Ok(_) = TcpStream::connect_timeout(&format!("{}:80", host).parse().unwrap(), timeout) {
        result.ping_ms = Some(ping_start.elapsed().as_millis());
    }
    
    for &port in ports {
        let addr_string = format!("{}:{}", host, port);
        if let Ok(addr) = addr_string.to_socket_addrs() {
            if let Some(addr) = addr.next() {
                if TcpStream::connect_timeout(&addr, timeout).is_ok() {
                    result.ports_open.push(port);
                }
            }
        }
    }
    
    result.check_time = start_time.elapsed();
    Ok(result)
}

pub fn print_network_report(results: &[NetworkCheckResult]) {
    println!("Network Health Report");
    println!("=====================");
    
    for result in results {
        println!("\nHost: {}", result.host);
        match result.ping_ms {
            Some(ms) => println!("  Ping: {} ms", ms),
            None => println!("  Ping: Unreachable"),
        }
        
        if result.ports_open.is_empty() {
            println!("  Open ports: None");
        } else {
            println!("  Open ports: {:?}", result.ports_open);
        }
        
        println!("  Check duration: {:?}", result.check_time);
    }
    
    let total_hosts = results.len();
    let reachable_hosts = results.iter().filter(|r| r.ping_ms.is_some()).count();
    println!("\nSummary: {}/{} hosts reachable", reachable_hosts, total_hosts);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_localhost_connectivity() {
        let result = check_host_connectivity("127.0.0.1", &[80, 443, 8080], 1);
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.host, "127.0.0.1");
    }
}