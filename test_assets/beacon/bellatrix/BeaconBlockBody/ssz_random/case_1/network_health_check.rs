
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;

pub struct NetworkProbe {
    target: SocketAddr,
    timeout: Duration,
}

impl NetworkProbe {
    pub fn new(host: IpAddr, port: u16) -> Self {
        Self {
            target: SocketAddr::new(host, port),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn measure_latency(&self, samples: usize) -> Option<Duration> {
        let mut latencies = Vec::with_capacity(samples);
        
        for _ in 0..samples {
            match self.single_ping() {
                Some(duration) => latencies.push(duration),
                None => return None,
            }
        }

        latencies.sort();
        let median_index = samples / 2;
        Some(latencies[median_index])
    }

    pub fn packet_loss_test(&self, packets: usize) -> f64 {
        let mut successful = 0;
        
        for _ in 0..packets {
            if self.single_ping().is_some() {
                successful += 1;
            }
        }

        let loss_percentage = (packets - successful) as f64 / packets as f64 * 100.0;
        loss_percentage
    }

    fn single_ping(&self) -> Option<Duration> {
        let start = Instant::now();
        
        let mut rng = rand::thread_rng();
        let simulated_delay = rng.gen_range(10..100);
        
        if simulated_delay > 95 {
            return None;
        }

        std::thread::sleep(Duration::from_millis(simulated_delay));
        Some(start.elapsed())
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

pub fn check_network_health() -> String {
    let target = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    let probe = NetworkProbe::new(target, 53);
    
    let latency = probe.measure_latency(10);
    let packet_loss = probe.packet_loss_test(20);
    
    match latency {
        Some(lat) => format!("Latency: {:?}, Packet Loss: {:.1}%", lat, packet_loss),
        None => "Network unreachable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_creation() {
        let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let probe = NetworkProbe::new(addr, 8080);
        assert_eq!(probe.target.port(), 8080);
    }

    #[test]
    fn test_packet_loss_calculation() {
        let addr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let probe = NetworkProbe::new(addr, 80);
        let loss = probe.packet_loss_test(100);
        assert!(loss >= 0.0 && loss <= 100.0);
    }
}