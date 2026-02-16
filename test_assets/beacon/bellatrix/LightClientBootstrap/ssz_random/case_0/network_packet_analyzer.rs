use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PacketStats {
    pub total_packets: u64,
    pub protocol_counts: HashMap<String, u64>,
    pub start_time: u64,
}

impl PacketStats {
    pub fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn update(&mut self, protocol: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    pub fn display(&self) {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.start_time;
        
        println!("Packet Capture Statistics:");
        println!("Duration: {} seconds", duration);
        println!("Total packets: {}", self.total_packets);
        
        if duration > 0 {
            println!("Packets/sec: {:.2}", self.total_packets as f64 / duration as f64);
        }
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.2}%)", protocol, count, percentage);
        }
    }
}

fn process_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        pnet::packet::ethernet::EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                match ipv4_packet.get_next_level_protocol() {
                    IpNextHeaderProtocols::Tcp => {
                        if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                            stats.update("TCP");
                            println!(
                                "TCP Packet: {}:{} -> {}:{} [Seq: {} Ack: {}]",
                                ipv4_packet.get_source(),
                                tcp_packet.get_source(),
                                ipv4_packet.get_destination(),
                                tcp_packet.get_destination(),
                                tcp_packet.get_sequence(),
                                tcp_packet.get_acknowledgement()
                            );
                        }
                    }
                    IpNextHeaderProtocols::Udp => {
                        if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                            stats.update("UDP");
                            println!(
                                "UDP Packet: {}:{} -> {}:{}",
                                ipv4_packet.get_source(),
                                udp_packet.get_source(),
                                ipv4_packet.get_destination(),
                                udp_packet.get_destination()
                            );
                        }
                    }
                    IpNextHeaderProtocols::Icmp => {
                        stats.update("ICMP");
                        println!(
                            "ICMP Packet: {} -> {}",
                            ipv4_packet.get_source(),
                            ipv4_packet.get_destination()
                        );
                    }
                    _ => {
                        stats.update("Other-IPv4");
                    }
                }
            }
        }
        pnet::packet::ethernet::EtherTypes::Ipv6 => {
            stats.update("IPv6");
        }
        pnet::packet::ethernet::EtherTypes::Arp => {
            stats.update("ARP");
        }
        _ => {
            stats.update("Other");
        }
    }
}

pub fn start_capture(interface_name: &str) -> Result<(), String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop and display statistics\n");

    let mut stats = PacketStats::new();
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_packet(&ethernet_packet, &mut stats);
                    packet_count += 1;

                    if packet_count % 100 == 0 {
                        println!("Processed {} packets...", packet_count);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    stats.display();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_stats() {
        let mut stats = PacketStats::new();
        assert_eq!(stats.total_packets, 0);
        assert!(stats.protocol_counts.is_empty());

        stats.update("TCP");
        stats.update("TCP");
        stats.update("UDP");

        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.protocol_counts.get("TCP"), Some(&2));
        assert_eq!(stats.protocol_counts.get("UDP"), Some(&1));
    }
}