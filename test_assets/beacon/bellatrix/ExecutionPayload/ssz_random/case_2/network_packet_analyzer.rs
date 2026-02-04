use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: u8,
    payload: Vec<u8>,
    timestamp: u64,
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

    pub fn get_protocol_name(&self) -> &'static str {
        match self.protocol {
            1 => "ICMP",
            6 => "TCP",
            17 => "UDP",
            _ => "UNKNOWN",
        }
    }

    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }

    pub fn is_local_traffic(&self) -> bool {
        let source_octets = self.source_ip.octets();
        let dest_octets = self.destination_ip.octets();
        source_octets[0] == 192 && source_octets[1] == 168
            && dest_octets[0] == 192 && dest_octets[1] == 168
    }
}

pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_stats: HashMap<String, usize>,
    traffic_by_ip: HashMap<Ipv4Addr, usize>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            traffic_by_ip: HashMap::new(),
        }
    }

    pub fn analyze_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;

        let protocol_name = packet.get_protocol_name().to_string();
        *self.protocol_stats.entry(protocol_name).or_insert(0) += 1;

        *self.traffic_by_ip.entry(packet.source_ip).or_insert(0) += packet.payload_size();
        *self.traffic_by_ip.entry(packet.destination_ip).or_insert(0) += packet.payload_size();
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        report.push_str("Protocol distribution:\n");

        for (protocol, count) in &self.protocol_stats {
            report.push_str(&format!("  {}: {}\n", protocol, count));
        }

        report.push_str("Top traffic sources/destinations:\n");
        let mut ip_traffic: Vec<(&Ipv4Addr, &usize)> = self.traffic_by_ip.iter().collect();
        ip_traffic.sort_by(|a, b| b.1.cmp(a.1));

        for (ip, traffic) in ip_traffic.iter().take(5) {
            report.push_str(&format!("  {}: {} bytes\n", ip, traffic));
        }

        report
    }

    pub fn reset(&mut self) {
        self.packet_count = 0;
        self.protocol_stats.clear();
        self.traffic_by_ip.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_packet_analysis() {
        let mut analyzer = PacketAnalyzer::new();

        let packet1 = NetworkPacket::new(
            Ipv4Addr::from_str("192.168.1.10").unwrap(),
            Ipv4Addr::from_str("192.168.1.20").unwrap(),
            6,
            vec![0u8; 100],
            1234567890,
        );

        let packet2 = NetworkPacket::new(
            Ipv4Addr::from_str("10.0.0.5").unwrap(),
            Ipv4Addr::from_str("8.8.8.8").unwrap(),
            17,
            vec![0u8; 200],
            1234567891,
        );

        analyzer.analyze_packet(&packet1);
        analyzer.analyze_packet(&packet2);

        assert_eq!(analyzer.packet_count, 2);
        assert_eq!(analyzer.protocol_stats.get("TCP"), Some(&1));
        assert_eq!(analyzer.protocol_stats.get("UDP"), Some(&1));
        assert!(packet1.is_local_traffic());
        assert!(!packet2.is_local_traffic());
    }
}