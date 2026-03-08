rust
use pnet::datalink::{self, Channel, Config};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    start_time: u128,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }

    fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        self.total_packets += 1;
    }

    fn display_stats(&self) {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - self.start_time;
        
        println!("Packet Capture Statistics:");
        println!("Total packets: {}", self.total_packets);
        println!("Capture duration: {} ms", elapsed);
        println!("Packets per second: {:.2}", 
                 self.total_packets as f64 / (elapsed as f64 / 1000.0));
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.2}%)", protocol, count, percentage);
        }
    }
}

fn handle_ethernet_frame(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            stats.increment_protocol("IPv4");
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                handle_ipv4_packet(&ipv4_packet, stats);
            }
        }
        EtherTypes::Ipv6 => {
            stats.increment_protocol("IPv6");
            if let Some(ipv6_packet) = Ipv6Packet::new(ethernet.payload()) {
                handle_ipv6_packet(&ipv6_packet, stats);
            }
        }
        EtherTypes::Arp => {
            stats.increment_protocol("ARP");
        }
        _ => {
            stats.increment_protocol("Other Ethernet");
        }
    }
}

fn handle_ipv4_packet(ipv4: &Ipv4Packet, stats: &mut PacketStats) {
    match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            stats.increment_protocol("TCP");
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                analyze_tcp_packet(&tcp_packet);
            }
        }
        IpNextHeaderProtocols::Udp => {
            stats.increment_protocol("UDP");
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                analyze_udp_packet(&udp_packet);
            }
        }
        IpNextHeaderProtocols::Icmp => {
            stats.increment_protocol("ICMP");
        }
        _ => {
            stats.increment_protocol("Other IPv4");
        }
    }
}

fn handle_ipv6_packet(ipv6: &Ipv6Packet, stats: &mut PacketStats) {
    match ipv6.get_next_header() {
        IpNextHeaderProtocols::Tcp => {
            stats.increment_protocol("TCPv6");
        }
        IpNextHeaderProtocols::Udp => {
            stats.increment_protocol("UDPv6");
        }
        IpNextHeaderProtocols::Icmpv6 => {
            stats.increment_protocol("ICMPv6");
        }
        _ => {
            stats.increment_protocol("Other IPv6");
        }
    }
}

fn analyze_tcp_packet(tcp: &TcpPacket) {
    println!("TCP Packet: {} -> {} | Seq: {} Ack: {} Window: {}",
             tcp.get_source(),
             tcp.get_destination(),
             tcp.get_sequence(),
             tcp.get_acknowledgement(),
             tcp.get_window());
}

fn analyze_udp_packet(udp: &UdpPacket) {
    println!("UDP Packet: {} -> {} | Length: {}",
             udp.get_source(),
             udp.get_destination(),
             udp.get_length());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
        .ok_or("No suitable network interface found")?;

    println!("Starting packet capture on interface: {}", interface.name);

    let mut config = Config::default();
    config.promiscuous = true;
    
    let (mut tx, mut rx) = match datalink::channel(&interface, config) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = PacketStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    println!("Capturing up to {} packets...", max_packets);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    handle_ethernet_frame(&ethernet_packet, &mut stats);
                    packet_count += 1;
                    
                    if packet_count >= max_packets {
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    stats.display_stats();
    Ok(())
}
```use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    start_time: u64,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        self.total_packets += 1;
    }

    fn display_stats(&self) {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.start_time;
        
        println!("Packet Capture Statistics:");
        println!("Duration: {} seconds", duration);
        println!("Total packets: {}", self.total_packets);
        
        if duration > 0 {
            println!("Packets/sec: {:.2}", self.total_packets as f64 / duration as f64);
        }
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn handle_ethernet_frame(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    stats.increment_protocol("Ethernet");
    
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                handle_ipv4_packet(&ipv4_packet, stats);
            }
        }
        EtherTypes::Arp => {
            stats.increment_protocol("ARP");
        }
        _ => {
            stats.increment_protocol("Other");
        }
    }
}

fn handle_ipv4_packet(ipv4: &Ipv4Packet, stats: &mut PacketStats) {
    stats.increment_protocol("IPv4");
    
    match ipv4.get_next_level_protocol() {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                stats.increment_protocol("TCP");
                println!(
                    "TCP Packet: {}:{} -> {}:{} [Flags: {:?}]",
                    ipv4.get_source(),
                    tcp_packet.get_source(),
                    ipv4.get_destination(),
                    tcp_packet.get_destination(),
                    tcp_packet.get_flags()
                );
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                stats.increment_protocol("UDP");
                println!(
                    "UDP Packet: {}:{} -> {}:{}",
                    ipv4.get_source(),
                    udp_packet.get_source(),
                    ipv4.get_destination(),
                    udp_packet.get_destination()
                );
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
            stats.increment_protocol("ICMP");
            println!("ICMP Packet from {}", ipv4.get_source());
        }
        _ => {
            stats.increment_protocol("Other-IP");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
        .ok_or("No suitable network interface found")?;

    println!("Starting packet capture on interface: {}", interface.name);

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = PacketStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    println!("Capturing up to {} packets...", max_packets);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    handle_ethernet_frame(&ethernet_packet, &mut stats);
                    packet_count += 1;
                    
                    if packet_count >= max_packets {
                        println!("\nReached maximum packet count.");
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    stats.display_stats();
    Ok(())
}