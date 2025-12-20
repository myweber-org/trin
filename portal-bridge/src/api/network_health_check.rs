use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use rand::Rng;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const MAX_PACKETS: usize = 10;

pub struct NetworkProbe {
    target: SocketAddr,
    socket: UdpSocket,
}

impl NetworkProbe {
    pub fn new(target_ip: Ipv4Addr, port: u16) -> std::io::Result<Self> {
        let target = SocketAddr::new(IpAddr::V4(target_ip), port);
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;
        
        Ok(Self { target, socket })
    }

    pub fn measure_latency(&self, attempts: usize) -> Vec<Duration> {
        let mut latencies = Vec::with_capacity(attempts);
        let mut rng = rand::thread_rng();
        
        for _ in 0..attempts {
            let mut packet = [0u8; PACKET_SIZE];
            rng.fill(&mut packet[..]);
            
            let start = Instant::now();
            if self.socket.send_to(&packet, self.target).is_ok() {
                let mut response = [0u8; PACKET_SIZE];
                if self.socket.recv_from(&mut response).is_ok() {
                    latencies.push(start.elapsed());
                }
            }
        }
        
        latencies
    }

    pub fn packet_loss_test(&self, packet_count: usize) -> f64 {
        let mut successful = 0;
        let mut rng = rand::thread_rng();
        
        for _ in 0..packet_count {
            let mut packet = [0u8; PACKET_SIZE];
            rng.fill(&mut packet[..]);
            
            if self.socket.send_to(&packet, self.target).is_ok() {
                let mut response = [0u8; PACKET_SIZE];
                if self.socket.recv_from(&mut response).is_ok() {
                    successful += 1;
                }
            }
        }
        
        let loss_percentage = (packet_count - successful) as f64 / packet_count as f64 * 100.0;
        loss_percentage
    }
}

pub fn analyze_latency(latencies: &[Duration]) -> (Duration, Duration, Duration) {
    if latencies.is_empty() {
        return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
    }
    
    let min = *latencies.iter().min().unwrap();
    let max = *latencies.iter().max().unwrap();
    let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    
    (min, max, avg)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_latency_analysis() {
        let latencies = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
        ];
        
        let (min, max, avg) = analyze_latency(&latencies);
        
        assert_eq!(min, Duration::from_millis(10));
        assert_eq!(max, Duration::from_millis(30));
        assert_eq!(avg, Duration::from_millis(20));
    }
}