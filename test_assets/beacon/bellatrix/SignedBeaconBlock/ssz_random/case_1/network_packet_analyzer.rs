
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
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
                if let Some(ethernet) = EthernetPacket::new(packet) {
                    analyze_packet(&ethernet);
                }
                if packet_count >= 100 {
                    println!("Captured 100 packets. Stopping.");
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

fn analyze_packet(ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                println!(
                    "IPv4 Packet: {} -> {} | Protocol: {}",
                    ipv4.get_source(),
                    ipv4.get_destination(),
                    ipv4.get_next_level_protocol()
                );

                match ipv4.get_next_level_protocol() {
                    pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                        if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                            println!(
                                "  TCP: {} -> {} | Flags: {:?}",
                                tcp.get_source(),
                                tcp.get_destination(),
                                tcp.get_flags()
                            );
                        }
                    }
                    pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                        if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                            println!(
                                "  UDP: {} -> {} | Length: {}",
                                udp.get_source(),
                                udp.get_destination(),
                                udp.get_length()
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        EtherTypes::Arp => {
            println!("ARP Packet detected");
        }
        _ => {
            println!("Other Ethernet type: {:?}", ethernet.get_ethertype());
        }
    }
}