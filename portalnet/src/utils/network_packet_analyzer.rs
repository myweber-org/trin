use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub protocol: Protocol,
    pub payload_size: usize,
}

pub struct PacketAnalyzer {
    protocol_counts: HashMap<Protocol, u32>,
    total_packets: u64,
    total_bytes: u64,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            protocol_counts: HashMap::new(),
            total_packets: 0,
            total_bytes: 0,
        }
    }

    pub fn process_packet(&mut self, header: &PacketHeader) {
        self.total_packets += 1;
        self.total_bytes += header.payload_size as u64;
        
        let count = self.protocol_counts.entry(header.protocol.clone()).or_insert(0);
        *count += 1;
    }

    pub fn get_statistics(&self) -> PacketStatistics {
        PacketStatistics {
            total_packets: self.total_packets,
            total_bytes: self.total_bytes,
            protocol_distribution: self.protocol_counts.clone(),
        }
    }

    pub fn detect_protocol(protocol_number: u8) -> Protocol {
        match protocol_number {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            n => Protocol::Unknown(n),
        }
    }
}

#[derive(Debug)]
pub struct PacketStatistics {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub protocol_distribution: HashMap<Protocol, u32>,
}

impl Default for PacketAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_detection() {
        assert_eq!(PacketAnalyzer::detect_protocol(6), Protocol::TCP);
        assert_eq!(PacketAnalyzer::detect_protocol(17), Protocol::UDP);
        assert_eq!(PacketAnalyzer::detect_protocol(1), Protocol::ICMP);
        assert_eq!(PacketAnalyzer::detect_protocol(99), Protocol::Unknown(99));
    }

    #[test]
    fn test_packet_processing() {
        let mut analyzer = PacketAnalyzer::new();
        
        let tcp_header = PacketHeader {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            protocol: Protocol::TCP,
            payload_size: 1500,
        };

        let udp_header = PacketHeader {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            protocol: Protocol::UDP,
            payload_size: 512,
        };

        analyzer.process_packet(&tcp_header);
        analyzer.process_packet(&udp_header);
        analyzer.process_packet(&tcp_header);

        let stats = analyzer.get_statistics();
        
        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.total_bytes, 3512);
        assert_eq!(*stats.protocol_distribution.get(&Protocol::TCP).unwrap(), 2);
        assert_eq!(*stats.protocol_distribution.get(&Protocol::UDP).unwrap(), 1);
    }
}