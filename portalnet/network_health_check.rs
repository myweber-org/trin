use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

const MAX_RETRIES: u32 = 3;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const PORTS_TO_CHECK: [u16; 3] = [80, 443, 22];

struct NetworkTestResult {
    host: String,
    port: u16,
    success: bool,
    latency: Option<Duration>,
    error_message: Option<String>,
}

fn test_connection(host: &str, port: u16) -> NetworkTestResult {
    let addr_str = format!("{}:{}", host, port);
    let mut result = NetworkTestResult {
        host: host.to_string(),
        port,
        success: false,
        latency: None,
        error_message: None,
    };

    for attempt in 1..=MAX_RETRIES {
        let start_time = Instant::now();
        
        match SocketAddr::from_str(&addr_str) {
            Ok(socket_addr) => {
                match TcpStream::connect_timeout(&socket_addr, CONNECTION_TIMEOUT) {
                    Ok(_) => {
                        result.success = true;
                        result.latency = Some(start_time.elapsed());
                        break;
                    }
                    Err(e) => {
                        if attempt == MAX_RETRIES {
                            result.error_message = Some(format!("Failed after {} attempts: {}", MAX_RETRIES, e));
                        }
                    }
                }
            }
            Err(e) => {
                result.error_message = Some(format!("Invalid address: {}", e));
                break;
            }
        }
        
        if attempt < MAX_RETRIES {
            std::thread::sleep(Duration::from_millis(500));
        }
    }
    
    result
}

fn run_network_health_check(host: &str) -> Vec<NetworkTestResult> {
    let mut results = Vec::new();
    
    for &port in &PORTS_TO_CHECK {
        let result = test_connection(host, port);
        results.push(result);
    }
    
    results
}

fn print_results(results: &[NetworkTestResult]) {
    println!("Network Health Check Results:");
    println!("{:<20} {:<8} {:<10} {:<15} {:<30}", 
             "Host", "Port", "Status", "Latency", "Error");
    println!("{}", "-".repeat(85));
    
    for result in results {
        let status = if result.success { "OK" } else { "FAIL" };
        let latency = result.latency
            .map(|d| format!("{:.2}ms", d.as_millis()))
            .unwrap_or_else(|| "N/A".to_string());
        let error = result.error_message.as_deref().unwrap_or("");
        
        println!("{:<20} {:<8} {:<10} {:<15} {:<30}", 
                 result.host, result.port, status, latency, error);
    }
}

fn main() -> io::Result<()> {
    let test_host = "example.com";
    println!("Testing connectivity to {}...", test_host);
    
    let results = run_network_health_check(test_host);
    print_results(&results);
    
    let successful_tests = results.iter().filter(|r| r.success).count();
    println!("\nSummary: {}/{} ports accessible", successful_tests, results.len());
    
    if successful_tests == results.len() {
        println!("Network connectivity: EXCELLENT");
    } else if successful_tests > 0 {
        println!("Network connectivity: PARTIAL");
    } else {
        println!("Network connectivity: FAILED");
        return Err(io::Error::new(io::ErrorKind::ConnectionRefused, "No network connectivity"));
    }
    
    Ok(())
}