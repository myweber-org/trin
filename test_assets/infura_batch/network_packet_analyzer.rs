
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
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
        .unwrap_or_else(|| {
            eprintln!("Interface {} not found", interface_name);
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
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                if let Some(ethernet) = EthernetPacket::new(packet) {
                    analyze_ethernet_packet(&ethernet);
                }
                if packet_count >= 100 {
                    println!("Captured {} packets. Stopping.", packet_count);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn analyze_ethernet_packet(ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                analyze_ipv4_packet(&ipv4);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Other Ethernet type: {:?}", ethernet.get_ethertype());
        }
    }
}

fn analyze_ipv4_packet(ipv4: &Ipv4Packet) {
    let src = ipv4.get_source();
    let dst = ipv4.get_destination();
    let protocol = ipv4.get_next_level_protocol();

    match protocol {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                println!(
                    "TCP Packet: {}:{} -> {}:{}",
                    src,
                    tcp.get_source(),
                    dst,
                    tcp.get_destination()
                );
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                println!(
                    "UDP Packet: {}:{} -> {}:{}",
                    src,
                    udp.get_source(),
                    dst,
                    udp.get_destination()
                );
            }
        }
        IpNextHeaderProtocols::Icmp => {
            println!("ICMP Packet: {} -> {}", src, dst);
        }
        _ => {
            println!("Other IP protocol: {:?} {} -> {}", protocol, src, dst);
        }
    }
}