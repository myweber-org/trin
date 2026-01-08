use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface_name = env::args().nth(1).unwrap_or_else(|| "eth0".to_string());
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .expect("Interface not found");

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create datalink channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                if let Some(eth_packet) = EthernetPacket::new(packet) {
                    analyze_ethernet_frame(&eth_packet, packet_count);
                }
                if packet_count >= 100 {
                    println!("Captured {} packets. Stopping.", packet_count);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to receive packet: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn analyze_ethernet_frame(eth_packet: &EthernetPacket, count: usize) {
    println!("\n=== Packet #{} ===", count);
    println!("Source MAC: {}", eth_packet.get_source());
    println!("Destination MAC: {}", eth_packet.get_destination());
    println!("EtherType: 0x{:04x}", eth_packet.get_ethertype().0);

    match eth_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(eth_packet.payload()) {
                analyze_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Other protocol: 0x{:04x}", eth_packet.get_ethertype().0);
        }
    }
}

fn analyze_ipv4_packet(ipv4_packet: &Ipv4Packet) {
    println!("IPv4 Packet:");
    println!("  Source IP: {}", ipv4_packet.get_source());
    println!("  Destination IP: {}", ipv4_packet.get_destination());
    println!("  Protocol: {}", ipv4_packet.get_next_level_protocol());
    println!("  TTL: {}", ipv4_packet.get_ttl());
    println!("  Length: {} bytes", ipv4_packet.get_total_length());

    match ipv4_packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                analyze_tcp_packet(&tcp_packet);
            }
        }
        IpNextHeaderProtocols::Udp => {
            println!("  UDP packet");
        }
        IpNextHeaderProtocols::Icmp => {
            println!("  ICMP packet");
        }
        _ => {
            println!("  Other IP protocol: {}", ipv4_packet.get_next_level_protocol());
        }
    }
}

fn analyze_tcp_packet(tcp_packet: &TcpPacket) {
    println!("  TCP Segment:");
    println!("    Source Port: {}", tcp_packet.get_source());
    println!("    Destination Port: {}", tcp_packet.get_destination());
    println!("    Sequence Number: {}", tcp_packet.get_sequence());
    println!("    Acknowledgment Number: {}", tcp_packet.get_acknowledgement());
    println!("    Flags: SYN={}, ACK={}, FIN={}, RST={}",
             tcp_packet.get_syn(),
             tcp_packet.get_ack(),
             tcp_packet.get_fin(),
             tcp_packet.get_rst());
    println!("    Window Size: {}", tcp_packet.get_window());
    
    let payload_len = tcp_packet.payload().len();
    if payload_len > 0 {
        println!("    Payload length: {} bytes", payload_len);
        if payload_len <= 32 {
            println!("    First {} bytes of payload: {:?}", 
                    payload_len.min(32), 
                    &tcp_packet.payload()[..payload_len.min(32)]);
        }
    }
}