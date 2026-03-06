use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct Packet {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: Protocol,
    payload_size: usize,
    timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

pub struct PacketAnalyzer {
    packet_count: u64,
    protocol_stats: HashMap<Protocol, u64>,
    ip_traffic: HashMap<Ipv4Addr, u64>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn process_packet(&mut self, packet: &Packet) {
        self.packet_count += 1;

        *self.protocol_stats.entry(packet.protocol.clone()).or_insert(0) += 1;

        *self.ip_traffic.entry(packet.source_ip).or_insert(0) += 1;
        *self.ip_traffic.entry(packet.destination_ip).or_insert(0) += 1;
    }

    pub fn get_statistics(&self) -> AnalyzerStats {
        let mut top_ips: Vec<(Ipv4Addr, u64)> = self.ip_traffic.iter()
            .map(|(&ip, &count)| (ip, count))
            .collect();
        
        top_ips.sort_by(|a, b| b.1.cmp(&a.1));
        top_ips.truncate(5);

        AnalyzerStats {
            total_packets: self.packet_count,
            protocol_distribution: self.protocol_stats.clone(),
            top_talkers: top_ips,
        }
    }

    pub fn detect_anomaly(&self, threshold: u64) -> Option<Anomaly> {
        for (&ip, &count) in &self.ip_traffic {
            if count > threshold {
                return Some(Anomaly::HighTraffic(ip, count));
            }
        }

        let tcp_count = self.protocol_stats.get(&Protocol::TCP).unwrap_or(&0);
        let udp_count = self.protocol_stats.get(&Protocol::UDP).unwrap_or(&0);
        
        if *tcp_count > 0 && *udp_count > 0 {
            let ratio = *tcp_count as f64 / *udp_count as f64;
            if ratio > 10.0 || ratio < 0.1 {
                return Some(Anomaly::ProtocolImbalance(ratio));
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct AnalyzerStats {
    pub total_packets: u64,
    pub protocol_distribution: HashMap<Protocol, u64>,
    pub top_talkers: Vec<(Ipv4Addr, u64)>,
}

#[derive(Debug)]
pub enum Anomaly {
    HighTraffic(Ipv4Addr, u64),
    ProtocolImbalance(f64),
}

impl Packet {
    pub fn new(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        protocol: Protocol,
        payload_size: usize,
        timestamp: u64,
    ) -> Self {
        Packet {
            source_ip,
            destination_ip,
            protocol,
            payload_size,
            timestamp,
        }
    }
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