rust
use pnet::datalink::{self, Channel, Config};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    source_ips: HashMap<String, u64>,
    destination_ips: HashMap<String, u64>,
    start_time: u64,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ips: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn update(&mut self, protocol: &str, src_ip: &str, dst_ip: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        *self.source_ips.entry(src_ip.to_string()).or_insert(0) += 1;
        *self.destination_ips.entry(dst_ip.to_string()).or_insert(0) += 1;
    }

    fn display_summary(&self) {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.start_time;

        println!("Packet Capture Summary");
        println!("======================");
        println!("Duration: {} seconds", duration);
        println!("Total packets: {}", self.total_packets);
        println!("Packets/sec: {:.2}", self.total_packets as f64 / duration as f64);
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
        println!("\nTop 5 Source IPs:");
        self.display_top_ips(&self.source_ips);
        println!("\nTop 5 Destination IPs:");
        self.display_top_ips(&self.destination_ips);
    }

    fn display_top_ips(&self, ip_map: &HashMap<String, u64>) {
        let mut sorted_ips: Vec<_> = ip_map.iter().collect();
        sorted_ips.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (ip, count)) in sorted_ips.iter().take(5).enumerate() {
            let percentage = (**count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}. {}: {} ({:.1}%)", i + 1, ip, count, percentage);
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
                    pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                        if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                            let src_port = tcp_packet.get_source();
                            let dst_port = tcp_packet.get_destination();
                            stats.update("TCP", &src_ip, &dst_ip);
                            println!("TCP: {}:{} -> {}:{}", src_ip, src_port, dst_ip, dst_port);
                        }
                    }
                    pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                        if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                            let src_port = udp_packet.get_source();
                            let dst_port = udp_packet.get_destination();
                            stats.update("UDP", &src_ip, &dst_ip);
                            println!("UDP: {}:{} -> {}:{}", src_ip, src_port, dst_ip, dst_port);
                        }
                    }
                    _ => {
                        stats.update("IPv4-Other", &src_ip, &dst_ip);
                        println!("IPv4: {} -> {}", src_ip, dst_ip);
                    }
                }
            }
        }
        EtherTypes::Ipv6 => {
            if let Some(ipv6_packet) = Ipv6Packet::new(ethernet.payload()) {
                let src_ip = ipv6_packet.get_source().to_string();
                let dst_ip = ipv6_packet.get_destination().to_string();
                stats.update("IPv6", &src_ip, &dst_ip);
                println!("IPv6: {} -> {}", src_ip, dst_ip);
            }
        }
        _ => {
            stats.update("Other", "Unknown", "Unknown");
            println!("Other protocol: {:?}", ethernet.get_ethertype());
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

    let config = Config::default();
    let (mut tx, mut rx) = match datalink::channel(&interface, config) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = PacketStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    println!("Capturing up to {} packets...", max_packets);
    println!("Press Ctrl+C to stop early\n");

    while packet_count < max_packets {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    handle_packet(&ethernet_packet, &mut stats);
                    packet_count += 1;
                    
                    if packet_count % 10 == 0 {
                        println!("Captured {} packets...", packet_count);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    println!("\nCapture complete!");
    stats.display_summary();

    Ok(())
}
```