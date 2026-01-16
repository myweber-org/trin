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
    pub fn new(src: Ipv4Addr, dst: Ipv4Addr, proto: u8, data: Vec<u8>, ts: u64) -> Self {
        NetworkPacket {
            source_ip: src,
            destination_ip: dst,
            protocol: proto,
            payload: data,
            timestamp: ts,
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
}

pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_stats: HashMap<u8, usize>,
    total_bytes: usize,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            total_bytes: 0,
        }
    }

    pub fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;
        self.total_bytes += packet.payload_size();

        let count = self.protocol_stats.entry(packet.protocol).or_insert(0);
        *count += 1;
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        report.push_str(&format!("Total bytes processed: {}\n", self.total_bytes));
        report.push_str("Protocol distribution:\n");

        for (proto, count) in &self.protocol_stats {
            let proto_name = match proto {
                1 => "ICMP",
                6 => "TCP",
                17 => "UDP",
                _ => "UNKNOWN",
            };
            report.push_str(&format!("  {}: {} packets\n", proto_name, count));
        }

        if self.packet_count > 0 {
            let avg_size = self.total_bytes / self.packet_count;
            report.push_str(&format!("Average packet size: {} bytes\n", avg_size));
        }

        report
    }
}

pub fn parse_raw_packet_data(data: &[u8]) -> Option<NetworkPacket> {
    if data.len() < 20 {
        return None;
    }

    let source_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let destination_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
    let protocol = data[9];

    let payload = data[20..].to_vec();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Some(NetworkPacket::new(source_ip, destination_ip, protocol, payload, timestamp))
}