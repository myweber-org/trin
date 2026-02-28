use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

pub struct NetworkChecker {
    targets: Vec<SocketAddr>,
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(targets: Vec<SocketAddr>, timeout_secs: u64) -> Self {
        NetworkChecker {
            targets,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_all(&self) -> Vec<CheckResult> {
        self.targets
            .iter()
            .map(|addr| self.check_single(addr))
            .collect()
    }

    fn check_single(&self, addr: &SocketAddr) -> CheckResult {
        let start = Instant::now();
        let connection_result = TcpStream::connect_timeout(addr, self.timeout);
        let elapsed = start.elapsed();

        match connection_result {
            Ok(_) => CheckResult::success(*addr, elapsed),
            Err(e) => CheckResult::failure(*addr, elapsed, e.to_string()),
        }
    }
}

pub struct CheckResult {
    pub address: SocketAddr,
    pub latency: Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

impl CheckResult {
    fn success(address: SocketAddr, latency: Duration) -> Self {
        CheckResult {
            address,
            latency,
            success: true,
            error_message: None,
        }
    }

    fn failure(address: SocketAddr, latency: Duration, error: String) -> Self {
        CheckResult {
            address,
            latency,
            success: false,
            error_message: Some(error),
        }
    }

    pub fn format(&self) -> String {
        if self.success {
            format!(
                "{}: Connected successfully ({} ms)",
                self.address,
                self.latency.as_millis()
            )
        } else {
            format!(
                "{}: Connection failed after {} ms - {}",
                self.address,
                self.latency.as_millis(),
                self.error_message.as_ref().unwrap()
            )
        }
    }
}

pub fn run_health_check() -> io::Result<()> {
    let targets = vec![
        "8.8.8.8:53".parse().unwrap(), // Google DNS
        "1.1.1.1:53".parse().unwrap(), // Cloudflare DNS
        "208.67.222.222:53".parse().unwrap(), // OpenDNS
    ];

    let checker = NetworkChecker::new(targets, 5);
    let results = checker.check_all();

    println!("Network Health Check Results:");
    println!("{}", "-".repeat(50));
    
    for result in results {
        println!("{}", result.format());
    }

    let successful = results.iter().filter(|r| r.success).count();
    println!("\nSummary: {}/{} targets reachable", successful, results.len());

    Ok(())
}