use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(1);

pub struct NetworkChecker {
    target: SocketAddr,
    timeout: Duration,
    max_retries: u32,
}

impl NetworkChecker {
    pub fn new(target: SocketAddr) -> Self {
        Self {
            target,
            timeout: DEFAULT_TIMEOUT,
            max_retries: MAX_RETRIES,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn check_connection(&self) -> Result<Duration, String> {
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            if attempt > 0 {
                std::thread::sleep(RETRY_DELAY);
            }

            match self.try_connect() {
                Ok(duration) => return Ok(duration),
                Err(e) => {
                    last_error = Some(e);
                    eprintln!("Connection attempt {} failed: {}", attempt + 1, e);
                }
            }
        }

        Err(format!(
            "Failed to connect after {} attempts. Last error: {}",
            self.max_retries,
            last_error.unwrap_or_else(|| "unknown".to_string())
        ))
    }

    fn try_connect(&self) -> Result<Duration, String> {
        let start = Instant::now();

        match TcpStream::connect_timeout(&self.target, self.timeout) {
            Ok(mut stream) => {
                let connect_time = start.elapsed();
                
                // Send a simple probe
                if let Err(e) = stream.write(b"PING") {
                    return Err(format!("Write failed after connection: {}", e));
                }
                
                stream.shutdown(std::net::Shutdown::Both)
                    .map_err(|e| format!("Shutdown failed: {}", e))?;
                
                Ok(connect_time)
            }
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    }
}

pub fn check_multiple_targets(targets: &[SocketAddr]) -> Vec<(SocketAddr, Result<Duration, String>)> {
    targets
        .iter()
        .map(|&target| {
            let checker = NetworkChecker::new(target);
            let result = checker.check_connection();
            (target, result)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_network_checker_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let checker = NetworkChecker::new(addr);
        
        assert_eq!(checker.target, addr);
        assert_eq!(checker.timeout, DEFAULT_TIMEOUT);
        assert_eq!(checker.max_retries, MAX_RETRIES);
    }

    #[test]
    fn test_with_custom_config() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        let custom_timeout = Duration::from_secs(2);
        let custom_retries = 5;
        
        let checker = NetworkChecker::new(addr)
            .with_timeout(custom_timeout)
            .with_max_retries(custom_retries);
        
        assert_eq!(checker.timeout, custom_timeout);
        assert_eq!(checker.max_retries, custom_retries);
    }
}