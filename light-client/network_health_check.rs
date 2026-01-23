
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use rand::Rng;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const TEST_COUNT: usize = 10;

pub struct NetworkHealth {
    target: SocketAddr,
    socket: UdpSocket,
}

impl NetworkHealth {
    pub fn new(host: IpAddr, port: u16) -> std::io::Result<Self> {
        let target = SocketAddr::new(host, port);
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;
        
        Ok(Self { target, socket })
    }

    pub fn measure_latency(&self) -> Result<Duration, String> {
        let mut buffer = [0u8; PACKET_SIZE];
        let mut rng = rand::thread_rng();
        rng.fill(&mut buffer);

        let start = Instant::now();
        self.socket.send_to(&buffer, self.target)
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut recv_buffer = [0u8; PACKET_SIZE];
        match self.socket.recv_from(&mut recv_buffer) {
            Ok((size, _)) if size == PACKET_SIZE => Ok(start.elapsed()),
            Ok(_) => Err("Invalid packet size".to_string()),
            Err(e) => Err(format!("Receive failed: {}", e)),
        }
    }

    pub fn run_diagnostics(&self) -> NetworkStats {
        let mut latencies = Vec::new();
        let mut lost_packets = 0;

        for _ in 0..TEST_COUNT {
            match self.measure_latency() {
                Ok(latency) => latencies.push(latency),
                Err(_) => lost_packets += 1,
            }
        }

        NetworkStats::from_measurements(&latencies, lost_packets)
    }
}

pub struct NetworkStats {
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub avg_latency: Duration,
    pub packet_loss: f32,
}

impl NetworkStats {
    fn from_measurements(latencies: &[Duration], lost: usize) -> Self {
        if latencies.is_empty() {
            return Self {
                min_latency: Duration::from_millis(0),
                max_latency: Duration::from_millis(0),
                avg_latency: Duration::from_millis(0),
                packet_loss: 100.0,
            };
        }

        let total = latencies.iter().sum::<Duration>();
        let avg = total / latencies.len() as u32;
        let min = *latencies.iter().min().unwrap();
        let max = *latencies.iter().max().unwrap();
        let loss_percentage = (lost as f32 / TEST_COUNT as f32) * 100.0;

        Self {
            min_latency: min,
            max_latency: max,
            avg_latency: avg,
            packet_loss: loss_percentage,
        }
    }

    pub fn display(&self) {
        println!("Network Diagnostics:");
        println!("  Minimum latency: {:.2}ms", self.min_latency.as_secs_f64() * 1000.0);
        println!("  Maximum latency: {:.2}ms", self.max_latency.as_secs_f64() * 1000.0);
        println!("  Average latency: {:.2}ms", self.avg_latency.as_secs_f64() * 1000.0);
        println!("  Packet loss: {:.1}%", self.packet_loss);
    }
}

pub fn check_localhost_health() {
    let checker = NetworkHealth::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
        .expect("Failed to create network checker");

    let stats = checker.run_diagnostics();
    stats.display();
}