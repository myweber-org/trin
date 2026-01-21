
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;
use rand::Rng;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const TEST_COUNT: usize = 10;

struct NetworkMetrics {
    latency_ms: f64,
    packet_loss_percent: f64,
    jitter_ms: f64,
}

impl NetworkMetrics {
    fn new() -> Self {
        NetworkMetrics {
            latency_ms: 0.0,
            packet_loss_percent: 0.0,
            jitter_ms: 0.0,
        }
    }
}

fn generate_test_packet() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..PACKET_SIZE).map(|_| rng.gen()).collect()
}

fn measure_latency(target: SocketAddr) -> Result<NetworkMetrics, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind socket: {}", e))?;
    
    socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let mut latencies = Vec::new();
    let mut lost_packets = 0;

    for seq in 0..TEST_COUNT {
        let packet = generate_test_packet();
        let send_time = Instant::now();
        
        if socket.send_to(&packet, target).is_err() {
            lost_packets += 1;
            continue;
        }

        let mut buffer = [0u8; PACKET_SIZE];
        match socket.recv_from(&mut buffer) {
            Ok((received, _)) => {
                if received == PACKET_SIZE {
                    let latency = send_time.elapsed().as_micros() as f64 / 1000.0;
                    latencies.push(latency);
                } else {
                    lost_packets += 1;
                }
            }
            Err(_) => {
                lost_packets += 1;
            }
        }

        if seq < TEST_COUNT - 1 {
            thread::sleep(Duration::from_millis(100));
        }
    }

    if latencies.is_empty() {
        return Err("All packets lost".to_string());
    }

    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let packet_loss = (lost_packets as f64 / TEST_COUNT as f64) * 100.0;
    
    let variance: f64 = latencies.iter()
        .map(|&l| (l - avg_latency).powi(2))
        .sum::<f64>() / latencies.len() as f64;
    let jitter = variance.sqrt();

    Ok(NetworkMetrics {
        latency_ms: avg_latency,
        packet_loss_percent: packet_loss,
        jitter_ms: jitter,
    })
}

fn check_network_health(target_ip: IpAddr, port: u16) {
    let target = SocketAddr::new(target_ip, port);
    
    println!("Testing network health to {}:{}", target_ip, port);
    println!("Sending {} packets with {}ms timeout", TEST_COUNT, TIMEOUT_MS);
    
    match measure_latency(target) {
        Ok(metrics) => {
            println!("Average latency: {:.2}ms", metrics.latency_ms);
            println!("Packet loss: {:.1}%", metrics.packet_loss_percent);
            println!("Jitter: {:.2}ms", metrics.jitter_ms);
            
            if metrics.latency_ms < 50.0 && metrics.packet_loss_percent < 1.0 {
                println!("Network health: EXCELLENT");
            } else if metrics.latency_ms < 100.0 && metrics.packet_loss_percent < 5.0 {
                println!("Network health: GOOD");
            } else if metrics.latency_ms < 200.0 && metrics.packet_loss_percent < 10.0 {
                println!("Network health: FAIR");
            } else {
                println!("Network health: POOR");
            }
        }
        Err(e) => {
            println!("Measurement failed: {}", e);
        }
    }
}

fn main() {
    let test_target = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    let test_port = 53;
    
    check_network_health(test_target, test_port);
}