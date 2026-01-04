
use std::process::Command;
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr};
use std::str;

const PACKET_COUNT: usize = 4;
const TIMEOUT_SECONDS: u64 = 2;

#[derive(Debug)]
pub struct PingResult {
    pub destination: String,
    pub packets_transmitted: usize,
    pub packets_received: usize,
    pub packet_loss: f32,
    pub avg_latency_ms: Option<f32>,
    pub reachable: bool,
}

pub fn check_host(host: &str) -> Result<PingResult, String> {
    let start_time = Instant::now();
    
    let output = Command::new("ping")
        .arg("-c")
        .arg(PACKET_COUNT.to_string())
        .arg("-W")
        .arg(TIMEOUT_SECONDS.to_string())
        .arg(host)
        .output()
        .map_err(|e| format!("Failed to execute ping: {}", e))?;

    let elapsed = start_time.elapsed();
    
    if !output.status.success() {
        return Ok(PingResult {
            destination: host.to_string(),
            packets_transmitted: PACKET_COUNT,
            packets_received: 0,
            packet_loss: 100.0,
            avg_latency_ms: None,
            reachable: false,
        });
    }

    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in ping output: {}", e))?;

    parse_ping_output(host, output_str, elapsed)
}

fn parse_ping_output(host: &str, output: &str, elapsed: Duration) -> Result<PingResult, String> {
    let lines: Vec<&str> = output.lines().collect();
    
    let mut packets_transmitted = 0;
    let mut packets_received = 0;
    let mut packet_loss = 100.0;
    let mut avg_latency = None;

    for line in lines {
        if line.contains("packets transmitted") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                if let Some(transmitted_part) = parts.get(0) {
                    if let Some(count_str) = transmitted_part.split_whitespace().next() {
                        packets_transmitted = count_str.parse().unwrap_or(PACKET_COUNT);
                    }
                }
                
                if let Some(received_part) = parts.get(1) {
                    if let Some(count_str) = received_part.split_whitespace().next() {
                        packets_received = count_str.parse().unwrap_or(0);
                    }
                }
                
                if let Some(loss_part) = parts.get(2) {
                    if let Some(loss_str) = loss_part.split_whitespace().next() {
                        if let Ok(loss_value) = loss_str.parse::<f32>() {
                            packet_loss = loss_value;
                        }
                    }
                }
            }
        }
        
        if line.contains("min/avg/max/mdev") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let latency_parts: Vec<&str> = parts[1].trim().split('/').collect();
                if latency_parts.len() >= 2 {
                    if let Ok(avg) = latency_parts[1].parse::<f32>() {
                        avg_latency = Some(avg);
                    }
                }
            }
        }
    }

    let reachable = packets_received > 0 && packet_loss < 100.0;
    
    Ok(PingResult {
        destination: host.to_string(),
        packets_transmitted,
        packets_received,
        packet_loss,
        avg_latency_ms: avg_latency,
        reachable,
    })
}

pub fn check_default_gateway() -> Result<PingResult, String> {
    let gateway = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    check_host(&gateway.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_check() {
        let result = check_host("127.0.0.1");
        assert!(result.is_ok());
        
        if let Ok(ping_result) = result {
            assert_eq!(ping_result.destination, "127.0.0.1");
            assert!(ping_result.reachable);
            assert!(ping_result.packet_loss < 100.0);
        }
    }
}