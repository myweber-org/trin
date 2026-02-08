use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::env;
use std::process;

struct PacketStats {
    total_packets: usize,
    protocol_counts: HashMap<String, usize>,
    source_ips: HashMap<String, usize>,
    destination_ips: HashMap<String, usize>,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ips: HashMap::new(),
        }
    }

    fn update(&mut self, packet: &EthernetPacket) {
        self.total_packets += 1;

        match packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                self.protocol_counts
                    .entry("IPv4".to_string())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);

                if let Some(ipv4_packet) = Ipv4Packet::new(packet.payload()) {
                    let src_ip = ipv4_packet.get_source().to_string();
                    let dst_ip = ipv4_packet.get_destination().to_string();

                    self.source_ips
                        .entry(src_ip)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                    self.destination_ips
                        .entry(dst_ip)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);

                    match ipv4_packet.get_next_level_protocol() {
                        IpNextHeaderProtocols::Tcp => {
                            self.protocol_counts
                                .entry("TCP".to_string())
                                .and_modify(|count| *count += 1)
                                .or_insert(1);
                        }
                        IpNextHeaderProtocols::Udp => {
                            self.protocol_counts
                                .entry("UDP".to_string())
                                .and_modify(|count| *count += 1)
                                .or_insert(1);
                        }
                        _ => {
                            self.protocol_counts
                                .entry("Other".to_string())
                                .and_modify(|count| *count += 1)
                                .or_insert(1);
                        }
                    }
                }
            }
            EtherTypes::Ipv6 => {
                self.protocol_counts
                    .entry("IPv6".to_string())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            _ => {
                self.protocol_counts
                    .entry("Other".to_string())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
    }

    fn display(&self) {
        println!("Packet Statistics:");
        println!("Total packets captured: {}", self.total_packets);
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            println!("  {}: {}", protocol, count);
        }
        println!("\nTop 5 Source IPs:");
        let mut source_vec: Vec<(&String, &usize)> = self.source_ips.iter().collect();
        source_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in source_vec.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
        println!("\nTop 5 Destination IPs:");
        let mut dest_vec: Vec<(&String, &usize)> = self.destination_ips.iter().collect();
        dest_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in dest_vec.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
    }
}

fn capture_packets(interface_name: &str, packet_limit: usize) -> Result<PacketStats, String> {
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

    let mut stats = PacketStats::new();
    let mut packet_count = 0;

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop capture and display statistics\n");

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    stats.update(&ethernet_packet);
                    packet_count += 1;

                    if packet_count >= packet_limit {
                        println!("Reached packet limit of {}", packet_limit);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                continue;
            }
        }
    }

    Ok(stats)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <interface_name> [packet_limit]", args[0]);
        eprintln!("Example: {} eth0 1000", args[0]);
        process::exit(1);
    }

    let interface_name = &args[1];
    let packet_limit = if args.len() > 2 {
        args[2].parse().unwrap_or(1000)
    } else {
        1000
    };

    match capture_packets(interface_name, packet_limit) {
        Ok(stats) => stats.display(),
        Err(e) => eprintln!("Error: {}", e),
    }
}