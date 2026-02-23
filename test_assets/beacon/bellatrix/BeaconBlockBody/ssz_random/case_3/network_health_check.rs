
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;

pub struct NetworkProbe {
    target: SocketAddr,
    timeout: Duration,
}

impl NetworkProbe {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self {
            target: SocketAddr::new(IpAddr::V4(ip), port),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn measure_latency(&self, attempts: usize) -> Result<Vec<Duration>, String> {
        let mut results = Vec::with_capacity(attempts);
        
        for _ in 0..attempts {
            let start = Instant::now();
            
            match std::net::TcpStream::connect_timeout(&self.target, self.timeout) {
                Ok(_) => {
                    let duration = start.elapsed();
                    results.push(duration);
                }
                Err(e) => return Err(format!("Connection failed: {}", e)),
            }
            
            std::thread::sleep(Duration::from_millis(100));
        }
        
        Ok(results)
    }

    pub fn simulate_packet_loss(&self, packets: usize) -> f64 {
        let mut rng = rand::thread_rng();
        let mut lost = 0;
        
        for _ in 0..packets {
            if rng.gen_range(0.0..1.0) > 0.95 {
                lost += 1;
            }
        }
        
        (lost as f64 / packets as f64) * 100.0
    }

    pub fn generate_report(&self) -> String {
        let latency_result = self.measure_latency(10);
        let packet_loss = self.simulate_packet_loss(1000);
        
        match latency_result {
            Ok(latencies) => {
                let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
                let max_latency = latencies.iter().max().unwrap_or(&Duration::ZERO);
                let min_latency = latencies.iter().min().unwrap_or(&Duration::ZERO);
                
                format!(
                    "Network Health Report for {}:\n\
                    Average Latency: {:.2?}\n\
                    Minimum Latency: {:.2?}\n\
                    Maximum Latency: {:.2?}\n\
                    Simulated Packet Loss: {:.2}%\n\
                    Timeout Setting: {:?}",
                    self.target, avg_latency, min_latency, max_latency, 
                    packet_loss, self.timeout
                )
            }
            Err(e) => format!("Failed to generate report: {}", e),
        }
    }
}

pub fn check_connectivity(host: &str, port: u16) -> bool {
    if let Ok(addr) = host.parse::<Ipv4Addr>() {
        let probe = NetworkProbe::new(addr, port);
        probe.measure_latency(3).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_creation() {
        let probe = NetworkProbe::new(Ipv4Addr::new(8, 8, 8, 8), 53);
        assert_eq!(probe.target.port(), 53);
    }

    #[test]
    fn test_connectivity_check() {
        assert!(check_connectivity("8.8.8.8", 53));
    }
}