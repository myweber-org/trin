use pnet::datalink::{self, Channel, NetworkInterface};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::env;

fn main() {
    let interface_name = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: {} <interface>", env::args().next().unwrap());
        std::process::exit(1);
    });

    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .unwrap_or_else(|| {
            eprintln!("Interface '{}' not found", interface_name);
            std::process::exit(1);
        });

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            eprintln!("Unsupported channel type");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to create channel: {}", e);
            std::process::exit(1);
        }
    };

    println!("Capturing packets on interface: {}", interface_name);

    loop {
        match rx.next() {
            Ok(packet) => {
                let ethernet = EthernetPacket::new(packet).unwrap();
                match ethernet.get_ethertype() {
                    EtherTypes::Ipv4 => handle_ipv4_packet(&ethernet),
                    EtherTypes::Ipv6 => handle_ipv6_packet(&ethernet),
                    _ => println!("Unknown EtherType: {:?}", ethernet.get_ethertype()),
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn handle_ipv4_packet(ethernet: &EthernetPacket) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        println!(
            "IPv4 Packet: {} -> {}",
            ipv4_packet.get_source(),
            ipv4_packet.get_destination()
        );
        match ipv4_packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => handle_tcp_packet(&ipv4_packet),
            IpNextHeaderProtocols::Udp => handle_udp_packet(&ipv4_packet),
            _ => println!("Unsupported IPv4 protocol"),
        }
    }
}

fn handle_ipv6_packet(ethernet: &EthernetPacket) {
    if let Some(ipv6_packet) = Ipv6Packet::new(ethernet.payload()) {
        println!(
            "IPv6 Packet: {} -> {}",
            ipv6_packet.get_source(),
            ipv6_packet.get_destination()
        );
        match ipv6_packet.get_next_header() {
            IpNextHeaderProtocols::Tcp => handle_tcp_packet(&ipv6_packet),
            IpNextHeaderProtocols::Udp => handle_udp_packet(&ipv6_packet),
            _ => println!("Unsupported IPv6 protocol"),
        }
    }
}

fn handle_tcp_packet(ip_packet: &dyn Packet) {
    if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {
        println!(
            "TCP Packet: {} -> {} | Seq: {} Ack: {}",
            tcp_packet.get_source(),
            tcp_packet.get_destination(),
            tcp_packet.get_sequence(),
            tcp_packet.get_acknowledgement()
        );
    }
}

fn handle_udp_packet(ip_packet: &dyn Packet) {
    if let Some(udp_packet) = UdpPacket::new(ip_packet.payload()) {
        println!(
            "UDP Packet: {} -> {} | Length: {}",
            udp_packet.get_source(),
            udp_packet.get_destination(),
            udp_packet.get_length()
        );
    }
}