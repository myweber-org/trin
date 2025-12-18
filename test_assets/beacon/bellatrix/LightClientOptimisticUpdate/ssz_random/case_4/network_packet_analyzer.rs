rust
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
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
            eprintln!("Interface {} not found", interface_name);
            std::process::exit(1);
        });

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            eprintln!("Unsupported channel type");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to create channel: {}", e);
            std::process::exit(1);
        }
    };

    println!("Starting packet capture on interface: {}", interface_name);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_frame(&ethernet_packet);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn process_ethernet_frame(ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected (not processed in this example)");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Other Ethernet type: {:?}", ethernet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet) {
    let source = ipv4.get_source();
    let destination = ipv4.get_destination();
    let protocol = ipv4.get_next_level_protocol();
    
    println!(
        "IPv4 Packet: {} -> {} | Protocol: {:?} | Length: {}",
        source,
        destination,
        protocol,
        ipv4.get_total_length()
    );

    match protocol {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                process_tcp_packet(&tcp_packet, source, destination);
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                process_udp_packet(&udp_packet, source, destination);
            }
        }
        IpNextHeaderProtocols::Icmp => {
            println!("ICMP packet from {} to {}", source, destination);
        }
        _ => {
            println!("Other IP protocol: {:?}", protocol);
        }
    }
}

fn process_tcp_packet(tcp: &TcpPacket, source: std::net::Ipv4Addr, destination: std::net::Ipv4Addr) {
    println!(
        "TCP Segment: {}:{} -> {}:{} | Flags: {:?} | Window: {}",
        source,
        tcp.get_source(),
        destination,
        tcp.get_destination(),
        get_tcp_flags(tcp),
        tcp.get_window()
    );
}

fn process_udp_packet(udp: &UdpPacket, source: std::net::Ipv4Addr, destination: std::net::Ipv4Addr) {
    println!(
        "UDP Datagram: {}:{} -> {}:{} | Length: {}",
        source,
        udp.get_source(),
        destination,
        udp.get_destination(),
        udp.get_length()
    );
}

fn get_tcp_flags(tcp: &TcpPacket) -> String {
    let mut flags = Vec::new();
    
    if tcp.get_fin() { flags.push("FIN"); }
    if tcp.get_syn() { flags.push("SYN"); }
    if tcp.get_rst() { flags.push("RST"); }
    if tcp.get_psh() { flags.push("PSH"); }
    if tcp.get_ack() { flags.push("ACK"); }
    if tcp.get_urg() { flags.push("URG"); }
    if tcp.get_ece() { flags.push("ECE"); }
    if tcp.get_cwr() { flags.push("CWR"); }
    
    flags.join("|")
}
```