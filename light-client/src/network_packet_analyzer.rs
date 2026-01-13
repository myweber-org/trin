use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
struct NetworkPacket {
    source: IpAddr,
    destination: IpAddr,
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
    protocol_stats: HashMap<Protocol, u64>,
    source_ip_stats: HashMap<IpAddr, u64>,
    total_payload: usize,
}

impl PacketAnalyzer {
    fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            source_ip_stats: HashMap::new(),
            total_payload: 0,
        }
    }

    fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;
        self.total_payload += packet.payload_size;

        *self.protocol_stats
            .entry(packet.protocol.clone())
            .or_insert(0) += 1;

        *self.source_ip_stats
            .entry(packet.source)
            .or_insert(0) += 1;
    }

    fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        report.push_str(&format!("Total payload size: {} bytes\n", self.total_payload));
        
        if self.packet_count > 0 {
            report.push_str(&format!("Average packet size: {:.2} bytes\n", 
                self.total_payload as f64 / self.packet_count as f64));
        }

        report.push_str("\nProtocol distribution:\n");
        for (protocol, count) in &self.protocol_stats {
            let percentage = (*count as f64 / self.packet_count as f64) * 100.0;
            report.push_str(&format!("  {:?}: {} ({:.1}%)\n", protocol, count, percentage));
        }

        report.push_str("\nTop source IPs:\n");
        let mut sorted_ips: Vec<_> = self.source_ip_stats.iter().collect();
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
            source: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            destination: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            protocol: Protocol::TCP,
            payload_size: 1500,
            timestamp: 1234567890,
        },
        NetworkPacket {
            source: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101)),
            destination: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            protocol: Protocol::UDP,
            payload_size: 512,
            timestamp: 1234567891,
        },
        NetworkPacket {
            source: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            destination: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3)),
            protocol: Protocol::TCP,
            payload_size: 1024,
            timestamp: 1234567892,
        },
        NetworkPacket {
            source: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 102)),
            destination: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            protocol: Protocol::ICMP,
            payload_size: 64,
            timestamp: 1234567893,
        },
    ]
}

fn main() {
    let mut analyzer = PacketAnalyzer::new();
    let packets = create_sample_packets();

    for packet in &packets {
        analyzer.process_packet(packet);
    }

    println!("{}", analyzer.generate_report());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_analyzer_initialization() {
        let analyzer = PacketAnalyzer::new();
        assert_eq!(analyzer.packet_count, 0);
        assert_eq!(analyzer.total_payload, 0);
        assert!(analyzer.protocol_stats.is_empty());
        assert!(analyzer.source_ip_stats.is_empty());
    }

    #[test]
    fn test_packet_processing() {
        let mut analyzer = PacketAnalyzer::new();
        let packet = NetworkPacket {
            source: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            destination: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            protocol: Protocol::TCP,
            payload_size: 100,
            timestamp: 1234567890,
        };

        analyzer.process_packet(&packet);
        assert_eq!(analyzer.packet_count, 1);
        assert_eq!(analyzer.total_payload, 100);
        assert_eq!(analyzer.protocol_stats.get(&Protocol::TCP), Some(&1));
        assert_eq!(analyzer.source_ip_stats.get(&packet.source), Some(&1));
    }

    #[test]
    fn test_report_generation() {
        let mut analyzer = PacketAnalyzer::new();
        let packets = create_sample_packets();

        for packet in &packets {
            analyzer.process_packet(packet);
        }

        let report = analyzer.generate_report();
        assert!(report.contains("Total packets analyzed: 4"));
        assert!(report.contains("Total payload size: 3100 bytes"));
        assert!(report.contains("Average packet size: 775.00 bytes"));
    }
}