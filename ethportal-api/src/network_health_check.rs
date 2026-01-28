
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;

const PACKET_COUNT: usize = 10;
const TIMEOUT_SECONDS: u64 = 2;

pub struct NetworkMetrics {
    pub latency_ms: f64,
    pub packet_loss_percent: f64,
    pub jitter_ms: f64,
}

pub fn check_network_health(target: IpAddr) -> Result<NetworkMetrics, String> {
    let port = 80;
    let socket_addr = SocketAddr::new(target, port);
    
    let mut latencies = Vec::new();
    let mut successful_packets = 0;
    
    for _ in 0..PACKET_COUNT {
        let start = Instant::now();
        
        match std::net::TcpStream::connect_timeout(&socket_addr, Duration::from_secs(TIMEOUT_SECONDS)) {
            Ok(_) => {
                let duration = start.elapsed();
                latencies.push(duration.as_millis() as f64);
                successful_packets += 1;
            }
            Err(_) => {
                continue;
            }
        }
        
        std::thread::sleep(Duration::from_millis(100));
    }
    
    if latencies.is_empty() {
        return Err("All packets failed to reach target".to_string());
    }
    
    let packet_loss = ((PACKET_COUNT - successful_packets) as f64 / PACKET_COUNT as f64) * 100.0;
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    
    let jitter = calculate_jitter(&latencies, avg_latency);
    
    Ok(NetworkMetrics {
        latency_ms: avg_latency,
        packet_loss_percent: packet_loss,
        jitter_ms: jitter,
    })
}

fn calculate_jitter(latencies: &[f64], avg_latency: f64) -> f64 {
    if latencies.len() < 2 {
        return 0.0;
    }
    
    let variance: f64 = latencies.iter()
        .map(|&latency| (latency - avg_latency).powi(2))
        .sum::<f64>() / latencies.len() as f64;
    
    variance.sqrt()
}

pub fn generate_random_ip() -> IpAddr {
    let mut rng = rand::thread_rng();
    IpAddr::V4(Ipv4Addr::new(
        rng.gen_range(1..255),
        rng.gen_range(0..255),
        rng.gen_range(0..255),
        rng.gen_range(1..255),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jitter_calculation() {
        let latencies = vec![10.0, 12.0, 8.0, 11.0, 9.0];
        let avg = 10.0;
        let jitter = calculate_jitter(&latencies, avg);
        assert!(jitter > 0.0);
    }
    
    #[test]
    fn test_generate_random_ip() {
        let ip = generate_random_ip();
        assert!(ip.is_ipv4());
    }
}