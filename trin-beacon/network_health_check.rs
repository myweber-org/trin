use std::process::Command;
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr};
use std::str;

struct NetworkTarget {
    host: String,
    ip: IpAddr,
}

impl NetworkTarget {
    fn new(host: &str) -> Result<Self, String> {
        let ip = resolve_host(host)?;
        Ok(NetworkTarget {
            host: host.to_string(),
            ip,
        })
    }
}

fn resolve_host(host: &str) -> Result<IpAddr, String> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip);
    }

    let output = Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg(host)
        .output()
        .map_err(|e| format!("Failed to execute ping: {}", e))?;

    if output.status.success() {
        let stdout = str::from_utf8(&output.stdout).unwrap_or("");
        if let Some(line) = stdout.lines().find(|l| l.contains("PING")) {
            let ip_str = line
                .split('(')
                .nth(1)
                .and_then(|s| s.split(')').next())
                .ok_or_else(|| "Failed to parse IP from ping output".to_string())?;
            ip_str.parse::<IpAddr>().map_err(|e| format!("Invalid IP address: {}", e))
        } else {
            Err("No PING line found in output".to_string())
        }
    } else {
        Err(format!("Host resolution failed for: {}", host))
    }
}

fn perform_ping(target: &NetworkTarget, count: u8) -> Result<Vec<Duration>, String> {
    let mut latencies = Vec::new();

    for _ in 0..count {
        let start = Instant::now();
        
        let output = Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg(target.ip.to_string())
            .output()
            .map_err(|e| format!("Ping execution failed: {}", e))?;

        let elapsed = start.elapsed();

        if output.status.success() {
            latencies.push(elapsed);
        } else {
            return Err(format!("Ping failed to {}", target.host));
        }
    }

    Ok(latencies)
}

fn calculate_statistics(latencies: &[Duration]) -> (Duration, Duration, Duration) {
    if latencies.is_empty() {
        return (Duration::from_millis(0), Duration::from_millis(0), Duration::from_millis(0));
    }

    let min = *latencies.iter().min().unwrap();
    let max = *latencies.iter().max().unwrap();
    let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;

    (min, max, avg)
}

fn check_network_health(host: &str) -> Result<(), String> {
    println!("Checking network health for: {}", host);
    
    let target = NetworkTarget::new(host)?;
    println!("Resolved {} to {}", target.host, target.ip);

    let latencies = perform_ping(&target, 4)?;
    let (min, max, avg) = calculate_statistics(&latencies);

    println!("Ping statistics:");
    println!("  Packets sent: {}", latencies.len());
    println!("  Min latency: {:.2}ms", min.as_secs_f64() * 1000.0);
    println!("  Max latency: {:.2}ms", max.as_secs_f64() * 1000.0);
    println!("  Avg latency: {:.2}ms", avg.as_secs_f64() * 1000.0);

    if avg > Duration::from_millis(100) {
        println!("Warning: High average latency detected");
    }

    if latencies.len() < 4 {
        println!("Warning: Packet loss detected");
    }

    Ok(())
}

fn main() {
    let test_hosts = ["8.8.8.8", "google.com", "1.1.1.1"];
    
    for host in test_hosts.iter() {
        match check_network_health(host) {
            Ok(_) => println!("✓ {} passed health check\n", host),
            Err(e) => println!("✗ {} failed: {}\n", host, e),
        }
    }
}