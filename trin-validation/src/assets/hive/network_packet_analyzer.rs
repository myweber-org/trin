extern crate pnet;

use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::env;

fn handle_transport_protocol(source: &str, destination: &str, protocol: u8, payload: &[u8]) {
    match protocol {
        6 => {
            if let Some(tcp_packet) = TcpPacket::new(payload) {
                println!(
                    "{} TCP Packet: {}:{} -> {}:{} [Flags: {:?}]",
                    source,
                    tcp_packet.get_source(),
                    tcp_packet.get_destination(),
                    destination,
                    tcp_packet.get_flags()
                );
            }
        }
        17 => {
            if let Some(udp_packet) = UdpPacket::new(payload) {
                println!(
                    "{} UDP Packet: {} -> {} Length: {}",
                    source,
                    udp_packet.get_source(),
                    udp_packet.get_destination(),
                    udp_packet.get_length()
                );
            }
        }
        _ => println!("{} Unknown Transport Protocol: {}", source, protocol),
    }
}

fn handle_ipv4_packet(ethernet: &EthernetPacket) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        let source = ipv4_packet.get_source().to_string();
        let destination = ipv4_packet.get_destination().to_string();
        let protocol = ipv4_packet.get_next_level_protocol();

        println!(
            "IPv4 Packet: {} -> {} Protocol: {}",
            source, destination, protocol
        );

        handle_transport_protocol(&source, &destination, protocol.0, ipv4_packet.payload());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <interface>", args[0]);
        std::process::exit(1);
    }

    let interface_name = &args[1];
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == *interface_name)
        .expect("Interface not found");

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create datalink channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => handle_ipv4_packet(&ethernet_packet),
                        EtherTypes::Ipv6 => println!("IPv6 Packet (not analyzed)"),
                        _ => println!("Other Ethernet Type: {:?}", ethernet_packet.get_ethertype()),
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}