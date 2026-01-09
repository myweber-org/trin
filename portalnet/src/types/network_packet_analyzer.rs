
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, PartialEq)]
enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
struct PacketHeader {
    source_ip: String,
    destination_ip: String,
    protocol: Protocol,
    payload_size: usize,
    timestamp: u64,
}

struct PacketAnalyzer {
    packet_count: u64,
    protocol_stats: HashMap<Protocol, u64>,
    traffic_by_ip: HashMap<String, u64>,
}

impl PacketAnalyzer {
    fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            traffic_by_ip: HashMap::new(),
        }
    }

    fn parse_protocol(&self, protocol_num: u8) -> Protocol {
        match protocol_num {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown(protocol_num),
        }
    }

    fn process_packet(&mut self, header: PacketHeader) {
        self.packet_count += 1;
        
        *self.protocol_stats.entry(header.protocol.clone()).or_insert(0) += 1;
        
        *self.traffic_by_ip.entry(header.source_ip.clone()).or_insert(0) += header.payload_size as u64;
        *self.traffic_by_ip.entry(header.destination_ip.clone()).or_insert(0) += header.payload_size as u64;
    }

    fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        
        report.push_str("\nProtocol Statistics:\n");
        for (protocol, count) in &self.protocol_stats {
            report.push_str(&format!("  {:?}: {}\n", protocol, count));
        }
        
        report.push_str("\nTop 5 IPs by traffic:\n");
        let mut ip_traffic: Vec<(&String, &u64)> = self.traffic_by_ip.iter().collect();
        ip_traffic.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (ip, traffic)) in ip_traffic.iter().take(5).enumerate() {
            report.push_str(&format!("  {}. {}: {} bytes\n", i + 1, ip, traffic));
        }
        
        report
    }
}

fn create_sample_packets() -> Vec<PacketHeader> {
    vec![
        PacketHeader {
            source_ip: "192.168.1.100".to_string(),
            destination_ip: "10.0.0.1".to_string(),
            protocol: Protocol::TCP,
            payload_size: 1500,
            timestamp: 1633027200,
        },
        PacketHeader {
            source_ip: "10.0.0.1".to_string(),
            destination_ip: "192.168.1.100".to_string(),
            protocol: Protocol::TCP,
            payload_size: 800,
            timestamp: 1633027201,
        },
        PacketHeader {
            source_ip: "192.168.1.101".to_string(),
            destination_ip: "8.8.8.8".to_string(),
            protocol: Protocol::UDP,
            payload_size: 512,
            timestamp: 1633027202,
        },
        PacketHeader {
            source_ip: "8.8.8.8".to_string(),
            destination_ip: "192.168.1.101".to_string(),
            protocol: Protocol::ICMP,
            payload_size: 64,
            timestamp: 1633027203,
        },
    ]
}

fn main() {
    let mut analyzer = PacketAnalyzer::new();
    let packets = create_sample_packets();
    
    for packet in packets {
        analyzer.process_packet(packet);
    }
    
    println!("{}", analyzer.generate_report());
}