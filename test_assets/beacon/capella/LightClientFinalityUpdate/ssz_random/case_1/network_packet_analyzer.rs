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
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    source_ips: HashMap<String, u64>,
    destination_ports: HashMap<u16, u64>,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ports: HashMap::new(),
        }
    }

    fn update(&mut self, packet: &[u8]) {
        self.total_packets += 1;

        if let Some(eth_packet) = EthernetPacket::new(packet) {
            match eth_packet.get_ethertype() {
                EtherTypes::Ipv4 => {
                    self.protocol_counts
                        .entry("IPv4".to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);

                    if let Some(ipv4_packet) = Ipv4Packet::new(eth_packet.payload()) {
                        let src_ip = ipv4_packet.get_source().to_string();
                        self.source_ips
                            .entry(src_ip)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);

                        match ipv4_packet.get_next_level_protocol() {
                            IpNextHeaderProtocols::Tcp => {
                                self.protocol_counts
                                    .entry("TCP".to_string())
                                    .and_modify(|count| *count += 1)
                                    .or_insert(1);

                                if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                    let dst_port = tcp_packet.get_destination();
                                    self.destination_ports
                                        .entry(dst_port)
                                        .and_modify(|count| *count += 1)
                                        .or_insert(1);
                                }
                            }
                            IpNextHeaderProtocols::Udp => {
                                self.protocol_counts
                                    .entry("UDP".to_string())
                                    .and_modify(|count| *count += 1)
                                    .or_insert(1);

                                if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                                    let dst_port = udp_packet.get_destination();
                                    self.destination_ports
                                        .entry(dst_port)
                                        .and_modify(|count| *count += 1)
                                        .or_insert(1);
                                }
                            }
                            _ => {
                                self.protocol_counts
                                    .entry("Other-IPv4".to_string())
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
                        .entry("Other-Ethernet".to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }
        }
    }

    fn display(&self) {
        println!("Packet Capture Statistics:");
        println!("Total Packets: {}", self.total_packets);
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            println!("  {}: {}", protocol, count);
        }
        println!("\nTop Source IPs:");
        let mut sorted_ips: Vec<_> = self.source_ips.iter().collect();
        sorted_ips.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_ips.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
        println!("\nTop Destination Ports:");
        let mut sorted_ports: Vec<_> = self.destination_ports.iter().collect();
        sorted_ports.sort_by(|a, b| b.1.cmp(a.1));
        for (port, count) in sorted_ports.iter().take(10) {
            println!("  {}: {}", port, count);
        }
    }
}

fn capture_packets(interface_name: &str, packet_limit: u64) -> Result<(), String> {
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
    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop capture and display statistics\n");

    ctrlc::set_handler(move || {
        println!("\nCapture interrupted. Displaying statistics...");
    })
    .expect("Error setting Ctrl-C handler");

    let mut packet_count = 0;
    loop {
        match rx.next() {
            Ok(packet) => {
                stats.update(&packet);
                packet_count += 1;

                if packet_count >= packet_limit {
                    println!("Reached packet limit of {}", packet_limit);
                    break;
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <interface> <packet_limit>", args[0]);
        eprintln!("Example: {} eth0 1000", args[0]);
        process::exit(1);
    }

    let interface = &args[1];
    let packet_limit = args[2]
        .parse::<u64>()
        .unwrap_or_else(|_| {
            eprintln!("Invalid packet limit. Using default: 100");
            100
        });

    if let Err(e) = capture_packets(interface, packet_limit) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}