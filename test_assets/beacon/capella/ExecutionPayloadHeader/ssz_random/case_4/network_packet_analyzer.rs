use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
struct NetworkPacket {
    source_ip: IpAddr,
    destination_ip: IpAddr,
    protocol: Protocol,
    payload_size: usize,
    timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

struct PacketAnalyzer {
    packet_count: u64,
    protocol_distribution: HashMap<Protocol, u64>,
    source_ip_counter: HashMap<IpAddr, u64>,
    total_payload_size: u64,
}

impl PacketAnalyzer {
    fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_distribution: HashMap::new(),
            source_ip_counter: HashMap::new(),
            total_payload_size: 0,
        }
    }

    fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;
        self.total_payload_size += packet.payload_size as u64;

        *self.protocol_distribution
            .entry(packet.protocol.clone())
            .or_insert(0) += 1;

        *self.source_ip_counter
            .entry(packet.source_ip)
            .or_insert(0) += 1;
    }

    fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        report.push_str(&format!("Total payload size: {} bytes\n", self.total_payload_size));
        
        if self.packet_count > 0 {
            let avg_payload = self.total_payload_size / self.packet_count;
            report.push_str(&format!("Average payload size: {} bytes\n", avg_payload));
        }

        report.push_str("\nProtocol distribution:\n");
        for (protocol, count) in &self.protocol_distribution {
            let percentage = (*count as f64 / self.packet_count as f64) * 100.0;
            report.push_str(&format!("  {:?}: {} ({:.2}%)\n", protocol, count, percentage));
        }

        report.push_str("\nTop source IPs:\n");
        let mut sorted_ips: Vec<(&IpAddr, &u64)> = self.source_ip_counter.iter().collect();
        sorted_ips.sort_by(|a, b| b.1.cmp(a.1));
        
        for (ip, count) in sorted_ips.iter().take(5) {
            report.push_str(&format!("  {}: {}\n", ip, count));
        }

        report
    }
}

fn create_sample_packets() -> Vec<NetworkPacket> {
    vec![
        NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            protocol: Protocol::TCP,
            payload_size: 1500,
            timestamp: 1625097600,
        },
        NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            protocol: Protocol::UDP,
            payload_size: 512,
            timestamp: 1625097601,
        },
        NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            protocol: Protocol::TCP,
            payload_size: 800,
            timestamp: 1625097602,
        },
        NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            protocol: Protocol::ICMP,
            payload_size: 64,
            timestamp: 1625097603,
        },
    ]
}

fn main() {
    let mut analyzer = PacketAnalyzer::new();
    let packets = create_sample_packets();

    for packet in &packets {
        analyzer.process_packet(packet);
    }

    let report = analyzer.generate_report();
    println!("{}", report);
}