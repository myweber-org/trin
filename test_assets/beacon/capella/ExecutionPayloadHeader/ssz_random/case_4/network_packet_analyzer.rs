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
}use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: u8,
    payload: Vec<u8>,
    timestamp: u64,
}

pub struct PacketAnalyzer {
    packet_count: u64,
    protocol_distribution: HashMap<u8, u64>,
    ip_traffic: HashMap<Ipv4Addr, u64>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_distribution: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;

        *self.protocol_distribution
            .entry(packet.protocol)
            .or_insert(0) += 1;

        *self.ip_traffic
            .entry(packet.source_ip)
            .or_insert(0) += 1;

        *self.ip_traffic
            .entry(packet.destination_ip)
            .or_insert(0) += 1;
    }

    pub fn get_statistics(&self) -> AnalyzerStats {
        AnalyzerStats {
            total_packets: self.packet_count,
            unique_protocols: self.protocol_distribution.len(),
            unique_ips: self.ip_traffic.len(),
            top_protocol: self.find_top_protocol(),
            busiest_ip: self.find_busiest_ip(),
        }
    }

    fn find_top_protocol(&self) -> Option<(u8, u64)> {
        self.protocol_distribution
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(&protocol, &count)| (protocol, count))
    }

    fn find_busiest_ip(&self) -> Option<(Ipv4Addr, u64)> {
        self.ip_traffic
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(&ip, &count)| (ip, count))
    }
}

#[derive(Debug)]
pub struct AnalyzerStats {
    pub total_packets: u64,
    pub unique_protocols: usize,
    pub unique_ips: usize,
    pub top_protocol: Option<(u8, u64)>,
    pub busiest_ip: Option<(Ipv4Addr, u64)>,
}

impl NetworkPacket {
    pub fn new(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        protocol: u8,
        payload: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        NetworkPacket {
            source_ip,
            destination_ip,
            protocol,
            payload,
            timestamp,
        }
    }

    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }

    pub fn is_local_traffic(&self) -> bool {
        self.source_ip.is_private() && self.destination_ip.is_private()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_packet_analyzer() {
        let mut analyzer = PacketAnalyzer::new();

        let packet1 = NetworkPacket::new(
            Ipv4Addr::from_str("192.168.1.1").unwrap(),
            Ipv4Addr::from_str("192.168.1.2").unwrap(),
            6,
            vec![1, 2, 3, 4, 5],
            1234567890,
        );

        let packet2 = NetworkPacket::new(
            Ipv4Addr::from_str("10.0.0.1").unwrap(),
            Ipv4Addr::from_str("8.8.8.8").unwrap(),
            17,
            vec![6, 7, 8, 9, 10],
            1234567891,
        );

        analyzer.process_packet(&packet1);
        analyzer.process_packet(&packet2);

        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_packets, 2);
        assert_eq!(stats.unique_protocols, 2);
        assert_eq!(stats.unique_ips, 4);
    }

    #[test]
    fn test_local_traffic() {
        let local_packet = NetworkPacket::new(
            Ipv4Addr::from_str("192.168.1.1").unwrap(),
            Ipv4Addr::from_str("192.168.1.2").unwrap(),
            6,
            vec![],
            1234567890,
        );

        let external_packet = NetworkPacket::new(
            Ipv4Addr::from_str("192.168.1.1").unwrap(),
            Ipv4Addr::from_str("8.8.8.8").unwrap(),
            6,
            vec![],
            1234567891,
        );

        assert!(local_packet.is_local_traffic());
        assert!(!external_packet.is_local_traffic());
    }
}