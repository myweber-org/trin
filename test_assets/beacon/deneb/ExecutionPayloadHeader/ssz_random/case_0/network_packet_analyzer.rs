use std::net::{IpAddr, Ipv4Addr};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub timestamp: SystemTime,
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub protocol: Protocol,
    pub payload: Vec<u8>,
    pub packet_size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

impl NetworkPacket {
    pub fn new(
        source_ip: IpAddr,
        destination_ip: IpAddr,
        protocol: Protocol,
        payload: Vec<u8>,
    ) -> Self {
        let packet_size = payload.len() + 20; // Basic header size estimation
        NetworkPacket {
            timestamp: SystemTime::now(),
            source_ip,
            destination_ip,
            protocol,
            payload,
            packet_size,
        }
    }

    pub fn is_local_traffic(&self) -> bool {
        match (self.source_ip, self.destination_ip) {
            (IpAddr::V4(src), IpAddr::V4(dst)) => {
                src.is_private() || dst.is_private() || src.is_loopback() || dst.is_loopback()
            }
            _ => false,
        }
    }

    pub fn matches_filter(&self, filter: &PacketFilter) -> bool {
        if let Some(protocol) = filter.protocol {
            if self.protocol != protocol {
                return false;
            }
        }

        if let Some(min_size) = filter.min_size {
            if self.packet_size < min_size {
                return false;
            }
        }

        if let Some(max_size) = filter.max_size {
            if self.packet_size > max_size {
                return false;
            }
        }

        if filter.local_only && !self.is_local_traffic() {
            return false;
        }

        true
    }
}

#[derive(Debug, Default)]
pub struct PacketFilter {
    pub protocol: Option<Protocol>,
    pub min_size: Option<usize>,
    pub max_size: Option<usize>,
    pub local_only: bool,
}

pub struct PacketAnalyzer {
    packets: Vec<NetworkPacket>,
    filters: Vec<PacketFilter>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packets: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn add_packet(&mut self, packet: NetworkPacket) {
        if self.filters.is_empty() || self.filters.iter().any(|f| packet.matches_filter(f)) {
            self.packets.push(packet);
        }
    }

    pub fn add_filter(&mut self, filter: PacketFilter) {
        self.filters.push(filter);
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    pub fn get_packets(&self) -> &[NetworkPacket] {
        &self.packets
    }

    pub fn get_statistics(&self) -> PacketStatistics {
        let total_packets = self.packets.len();
        let total_bytes: usize = self.packets.iter().map(|p| p.packet_size).sum();

        let mut protocol_counts = std::collections::HashMap::new();
        for packet in &self.packets {
            let count = protocol_counts.entry(packet.protocol.clone()).or_insert(0);
            *count += 1;
        }

        PacketStatistics {
            total_packets,
            total_bytes,
            protocol_counts,
        }
    }
}

#[derive(Debug)]
pub struct PacketStatistics {
    pub total_packets: usize,
    pub total_bytes: usize,
    pub protocol_counts: std::collections::HashMap<Protocol, usize>,
}

impl Protocol {
    pub fn from_u8(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            other => Protocol::Other(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let source = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let dest = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));
        let payload = vec![1, 2, 3, 4, 5];
        let packet = NetworkPacket::new(source, dest, Protocol::TCP, payload);

        assert_eq!(packet.source_ip, source);
        assert_eq!(packet.destination_ip, dest);
        assert_eq!(packet.protocol, Protocol::TCP);
        assert!(packet.is_local_traffic());
    }

    #[test]
    fn test_packet_filtering() {
        let mut analyzer = PacketAnalyzer::new();

        let filter = PacketFilter {
            protocol: Some(Protocol::TCP),
            min_size: Some(10),
            max_size: Some(100),
            local_only: true,
        };
        analyzer.add_filter(filter);

        let local_packet = NetworkPacket::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            Protocol::TCP,
            vec![0; 50],
        );

        let external_packet = NetworkPacket::new(
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            Protocol::TCP,
            vec![0; 50],
        );

        analyzer.add_packet(local_packet);
        analyzer.add_packet(external_packet);

        assert_eq!(analyzer.get_packets().len(), 1);
    }
}