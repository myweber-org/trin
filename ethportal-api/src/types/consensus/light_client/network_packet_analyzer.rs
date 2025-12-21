
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: Protocol,
    pub length: usize,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_stats: HashMap<Protocol, usize>,
    source_ips: HashMap<String, usize>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            source_ips: HashMap::new(),
        }
    }

    pub fn analyze_packet(&mut self, header: &PacketHeader) {
        self.packet_count += 1;
        self.protocol_stats
            .entry(header.protocol.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        self.source_ips
            .entry(header.source_ip.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub fn get_statistics(&self) -> PacketStatistics {
        PacketStatistics {
            total_packets: self.packet_count,
            protocol_distribution: self.protocol_stats.clone(),
            top_source_ips: self.get_top_source_ips(5),
        }
    }

    fn get_top_source_ips(&self, limit: usize) -> Vec<(String, usize)> {
        let mut ip_list: Vec<_> = self.source_ips.iter().collect();
        ip_list.sort_by(|a, b| b.1.cmp(a.1));
        ip_list
            .into_iter()
            .take(limit)
            .map(|(ip, count)| (ip.clone(), *count))
            .collect()
    }
}

#[derive(Debug)]
pub struct PacketStatistics {
    pub total_packets: usize,
    pub protocol_distribution: HashMap<Protocol, usize>,
    pub top_source_ips: Vec<(String, usize)>,
}

pub fn parse_protocol(protocol_number: u8) -> Protocol {
    match protocol_number {
        6 => Protocol::TCP,
        17 => Protocol::UDP,
        1 => Protocol::ICMP,
        _ => Protocol::Unknown(protocol_number),
    }
}

pub fn validate_ipv4_address(ip: &str) -> bool {
    ip.parse::<Ipv4Addr>().is_ok()
}

pub fn validate_ipv6_address(ip: &str) -> bool {
    ip.parse::<Ipv6Addr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_protocol() {
        assert_eq!(parse_protocol(6), Protocol::TCP);
        assert_eq!(parse_protocol(17), Protocol::UDP);
        assert_eq!(parse_protocol(1), Protocol::ICMP);
        assert_eq!(parse_protocol(99), Protocol::Unknown(99));
    }

    #[test]
    fn test_packet_analyzer() {
        let mut analyzer = PacketAnalyzer::new();
        let packet = PacketHeader {
            source_ip: "192.168.1.1".to_string(),
            destination_ip: "192.168.1.2".to_string(),
            protocol: Protocol::TCP,
            length: 1500,
            timestamp: 1234567890,
        };
        analyzer.analyze_packet(&packet);
        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_packets, 1);
        assert_eq!(stats.protocol_distribution.get(&Protocol::TCP), Some(&1));
    }

    #[test]
    fn test_ip_validation() {
        assert!(validate_ipv4_address("192.168.1.1"));
        assert!(!validate_ipv4_address("256.256.256.256"));
        assert!(validate_ipv6_address("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
    }
}