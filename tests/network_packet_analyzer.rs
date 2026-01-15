
use pnet::datalink::{self, Channel, NetworkInterface};
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
    destination_ips: HashMap<String, u64>,
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

    fn update(&mut self, protocol: &str, src_ip: &str, dst_ip: &str) {
        self.total_packets += 1;
        
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        *self.source_ips.entry(src_ip.to_string()).or_insert(0) += 1;
        *self.destination_ips.entry(dst_ip.to_string()).or_insert(0) += 1;
    }

    fn display(&self) {
        println!("Packet Statistics:");
        println!("Total packets captured: {}", self.total_packets);
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            println!("  {}: {}", protocol, count);
        }
        
        println!("\nTop 5 Source IPs:");
        let mut sorted_src: Vec<_> = self.source_ips.iter().collect();
        sorted_src.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_src.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
        
        println!("\nTop 5 Destination IPs:");
        let mut sorted_dst: Vec<_> = self.destination_ips.iter().collect();
        sorted_dst.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_dst.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
    }
}

fn handle_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                let src_ip = ipv4_packet.get_source().to_string();
                let dst_ip = ipv4_packet.get_destination().to_string();
                
                match ipv4_packet.get_next_level_protocol() {
                    IpNextHeaderProtocols::Tcp => {
                        if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                            let src_port = tcp_packet.get_source();
                            let dst_port = tcp_packet.get_destination();
                            stats.update("TCP", &src_ip, &dst_ip);
                            println!("TCP Packet: {}:{} -> {}:{} (Seq: {}, Ack: {})",
                                     src_ip, src_port, dst_ip, dst_port,
                                     tcp_packet.get_sequence(),
                                     tcp_packet.get_acknowledgement());
                        }
                    }
                    IpNextHeaderProtocols::Udp => {
                        if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                            let src_port = udp_packet.get_source();
                            let dst_port = udp_packet.get_destination();
                            stats.update("UDP", &src_ip, &dst_ip);
                            println!("UDP Packet: {}:{} -> {}:{} (Length: {})",
                                     src_ip, src_port, dst_ip, dst_port,
                                     udp_packet.get_length());
                        }
                    }
                    _ => {
                        stats.update("Other-IPv4", &src_ip, &dst_ip);
                        println!("Other IPv4 Packet: {} -> {}", src_ip, dst_ip);
                    }
                }
            }
        }
        EtherTypes::Arp => {
            stats.update("ARP", "N/A", "N/A");
            println!("ARP Packet detected");
        }
        _ => {
            stats.update("Other", "N/A", "N/A");
            println!("Other Ethernet type: {:?}", ethernet.get_ethertype());
        }
    }
}

fn list_interfaces() {
    println!("Available network interfaces:");
    for interface in datalink::interfaces() {
        println!("  {}: {}", interface.name, interface.description);
        for ip in interface.ips {
            println!("    IP: {}", ip);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <interface_name>", args[0]);
        println!("\nAvailable interfaces:");
        list_interfaces();
        process::exit(1);
    }

    let interface_name = &args[1];
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == *interface_name)
        .unwrap_or_else(|| {
            println!("Interface {} not found", interface_name);
            list_interfaces();
            process::exit(1);
        });

    println!("Starting packet capture on interface: {}", interface.name);
    println!("Press Ctrl+C to stop and display statistics\n");

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            println!("Unsupported channel type");
            process::exit(1);
        }
        Err(e) => {
            println!("Error creating channel: {}", e);
            process::exit(1);
        }
    };

    let mut stats = PacketStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    ctrlc::set_handler(move || {
        println!("\nCapture interrupted by user");
    }).expect("Error setting Ctrl-C handler");

    println!("Capturing up to {} packets...", max_packets);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    handle_packet(&ethernet_packet, &mut stats);
                    packet_count += 1;
                    
                    if packet_count >= max_packets {
                        println!("\nReached maximum packet count ({})", max_packets);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    stats.display();
    println!("\nCapture complete. Analyzed {} packets.", packet_count);
}