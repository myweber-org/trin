extern crate pnet;

use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
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
                    "{} -> {} TCP | Sport: {} Dport: {} Seq: {} Ack: {} Flags: {:?}",
                    source,
                    destination,
                    tcp_packet.get_source(),
                    tcp_packet.get_destination(),
                    tcp_packet.get_sequence(),
                    tcp_packet.get_acknowledgement(),
                    tcp_packet.get_flags()
                );
            }
        }
        17 => {
            if let Some(udp_packet) = UdpPacket::new(payload) {
                println!(
                    "{} -> {} UDP | Sport: {} Dport: {} Length: {}",
                    source,
                    destination,
                    udp_packet.get_source(),
                    udp_packet.get_destination(),
                    udp_packet.get_length()
                );
            }
        }
        _ => println!(
            "{} -> {} Protocol: {} | Payload length: {}",
            source,
            destination,
            protocol,
            payload.len()
        ),
    }
}

fn handle_ipv4_packet(ethernet: &EthernetPacket) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        let source = ipv4_packet.get_source();
        let destination = ipv4_packet.get_destination();
        let protocol = ipv4_packet.get_next_level_protocol();

        println!(
            "IPv4 | {} -> {} | Protocol: {} | TTL: {}",
            source,
            destination,
            protocol,
            ipv4_packet.get_ttl()
        );

        handle_transport_protocol(&source.to_string(), &destination.to_string(), protocol.0, ipv4_packet.payload());
    }
}

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

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            eprintln!("Unsupported channel type");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error creating channel: {}", e);
            std::process::exit(1);
        }
    };

    println!("Starting packet capture on interface: {}", interface_name);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => handle_ipv4_packet(&ethernet_packet),
                        EtherTypes::Ipv6 => println!("IPv6 packet detected (not processed)"),
                        EtherTypes::Arp => println!("ARP packet detected"),
                        _ => println!("Unknown ethertype: {:?}", ethernet_packet.get_ethertype()),
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