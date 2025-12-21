rust
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let interface_name = if args.len() > 1 {
        &args[1]
    } else {
        "eth0"
    };

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
                let ethernet = EthernetPacket::new(packet).unwrap();
                process_ethernet_frame(&ethernet, packet_count);
            }
            Err(e) => {
                eprintln!("Failed to read packet: {}", e);
                break;
            }
        }
    }
}

fn process_ethernet_frame(ethernet: &EthernetPacket, count: u32) {
    println!("\n[Packet #{}]", count);
    println!("Source MAC: {}", ethernet.get_source());
    println!("Destination MAC: {}", ethernet.get_destination());
    println!("EtherType: 0x{:04x}", ethernet.get_ethertype().0);

    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            if let Some(ipv6_packet) = Ipv6Packet::new(ethernet.payload()) {
                process_ipv6_packet(&ipv6_packet);
            }
        }
        _ => {
            println!("Unsupported EtherType, packet length: {}", ethernet.packet().len());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet) {
    println!("IPv4 Packet:");
    println!("  Source: {}", ipv4.get_source());
    println!("  Destination: {}", ipv4.get_destination());
    println!("  Protocol: {}", ipv4.get_next_level_protocol());
    println!("  TTL: {}", ipv4.get_ttl());
    println!("  Length: {}", ipv4.get_total_length());

    match ipv4.get_next_level_protocol() {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                process_tcp_packet(&tcp_packet);
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                process_udp_packet(&udp_packet);
            }
        }
        _ => {
            println!("  Unsupported transport protocol");
        }
    }
}

fn process_ipv6_packet(ipv6: &Ipv6Packet) {
    println!("IPv6 Packet:");
    println!("  Source: {}", ipv6.get_source());
    println!("  Destination: {}", ipv6.get_destination());
    println!("  Next Header: {}", ipv6.get_next_header());
    println!("  Payload Length: {}", ipv6.get_payload_length());
}

fn process_tcp_packet(tcp: &TcpPacket) {
    println!("  TCP Segment:");
    println!("    Source Port: {}", tcp.get_source());
    println!("    Destination Port: {}", tcp.get_destination());
    println!("    Sequence Number: {}", tcp.get_sequence());
    println!("    Acknowledgment Number: {}", tcp.get_acknowledgement());
    println!("    Flags: SYN={}, ACK={}, FIN={}, RST={}",
             tcp.get_syn(), tcp.get_ack(), tcp.get_fin(), tcp.get_rst());
    println!("    Window Size: {}", tcp.get_window());
}

fn process_udp_packet(udp: &UdpPacket) {
    println!("  UDP Datagram:");
    println!("    Source Port: {}", udp.get_source());
    println!("    Destination Port: {}", udp.get_destination());
    println!("    Length: {}", udp.get_length());
    println!("    Checksum: 0x{:04x}", udp.get_checksum());
}
```