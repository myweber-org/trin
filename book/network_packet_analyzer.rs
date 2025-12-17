
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PacketStats {
    total_packets: usize,
    protocol_counts: HashMap<String, usize>,
    source_ips: HashMap<String, usize>,
    destination_ports: HashMap<u16, usize>,
    start_time: Instant,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ports: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, packet: &[u8]) {
        self.total_packets += 1;

        if let Some(eth_packet) = EthernetPacket::new(packet) {
            match eth_packet.get_ethertype() {
                EtherTypes::Ipv4 => {
                    self.increment_protocol("IPv4");
                    
                    if let Some(ip_packet) = Ipv4Packet::new(eth_packet.payload()) {
                        let source_ip = ip_packet.get_source().to_string();
                        *self.source_ips.entry(source_ip).or_insert(0) += 1;

                        match ip_packet.get_next_level_protocol() {
                            IpNextHeaderProtocols::Tcp => {
                                self.increment_protocol("TCP");
                                if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {
                                    let dst_port = tcp_packet.get_destination();
                                    *self.destination_ports.entry(dst_port).or_insert(0) += 1;
                                }
                            }
                            IpNextHeaderProtocols::Udp => {
                                self.increment_protocol("UDP");
                                if let Some(udp_packet) = UdpPacket::new(ip_packet.payload()) {
                                    let dst_port = udp_packet.get_destination();
                                    *self.destination_ports.entry(dst_port).or_insert(0) += 1;
                                }
                            }
                            _ => self.increment_protocol("Other-IP"),
                        }
                    }
                }
                EtherTypes::Arp => self.increment_protocol("ARP"),
                _ => self.increment_protocol("Other-Ethernet"),
            }
        }
    }

    fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    fn display_stats(&self) {
        let duration = self.start_time.elapsed();
        println!("Packet Capture Statistics:");
        println!("Duration: {:.2} seconds", duration.as_secs_f64());
        println!("Total packets captured: {}", self.total_packets);
        println!("Packets per second: {:.2}", 
                 self.total_packets as f64 / duration.as_secs_f64());
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }

        if !self.source_ips.is_empty() {
            println!("\nTop Source IPs:");
            let mut sorted_ips: Vec<_> = self.source_ips.iter().collect();
            sorted_ips.sort_by(|a, b| b.1.cmp(a.1));
            for (ip, count) in sorted_ips.iter().take(5) {
                println!("  {}: {}", ip, count);
            }
        }

        if !self.destination_ports.is_empty() {
            println!("\nTop Destination Ports:");
            let mut sorted_ports: Vec<_> = self.destination_ports.iter().collect();
            sorted_ports.sort_by(|a, b| b.1.cmp(a.1));
            for (port, count) in sorted_ports.iter().take(5) {
                println!("  {}: {}", port, count);
            }
        }
    }
}

fn capture_packets(interface_name: &str, duration_secs: u64) -> Result<PacketStats, String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    let mut stats = PacketStats::new();
    let timeout = Duration::from_secs(duration_secs);
    let start_time = Instant::now();

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Capture will run for {} seconds", duration_secs);
    println!("Press Ctrl+C to stop early\n");

    while start_time.elapsed() < timeout {
        match rx.next() {
            Ok(packet) => {
                stats.update(&packet);
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    Ok(stats)
}

fn main() {
    let interface_name = "eth0";
    let capture_duration = 10;

    match capture_packets(interface_name, capture_duration) {
        Ok(stats) => {
            println!("\n{}", "=".repeat(50));
            stats.display_stats();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}