
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, PartialEq)]
enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
struct PacketHeader {
    source_ip: String,
    destination_ip: String,
    protocol: Protocol,
    payload_size: usize,
    timestamp: u64,
}

struct PacketAnalyzer {
    packet_count: u64,
    protocol_stats: HashMap<Protocol, u64>,
    traffic_by_ip: HashMap<String, u64>,
}

impl PacketAnalyzer {
    fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            traffic_by_ip: HashMap::new(),
        }
    }

    fn parse_protocol(&self, protocol_num: u8) -> Protocol {
        match protocol_num {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown(protocol_num),
        }
    }

    fn process_packet(&mut self, header: PacketHeader) {
        self.packet_count += 1;
        
        *self.protocol_stats.entry(header.protocol.clone()).or_insert(0) += 1;
        
        *self.traffic_by_ip.entry(header.source_ip.clone()).or_insert(0) += header.payload_size as u64;
        *self.traffic_by_ip.entry(header.destination_ip.clone()).or_insert(0) += header.payload_size as u64;
    }

    fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        
        report.push_str("\nProtocol Statistics:\n");
        for (protocol, count) in &self.protocol_stats {
            report.push_str(&format!("  {:?}: {}\n", protocol, count));
        }
        
        report.push_str("\nTop 5 IPs by traffic:\n");
        let mut ip_traffic: Vec<(&String, &u64)> = self.traffic_by_ip.iter().collect();
        ip_traffic.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (ip, traffic)) in ip_traffic.iter().take(5).enumerate() {
            report.push_str(&format!("  {}. {}: {} bytes\n", i + 1, ip, traffic));
        }
        
        report
    }
}

fn create_sample_packets() -> Vec<PacketHeader> {
    vec![
        PacketHeader {
            source_ip: "192.168.1.100".to_string(),
            destination_ip: "10.0.0.1".to_string(),
            protocol: Protocol::TCP,
            payload_size: 1500,
            timestamp: 1633027200,
        },
        PacketHeader {
            source_ip: "10.0.0.1".to_string(),
            destination_ip: "192.168.1.100".to_string(),
            protocol: Protocol::TCP,
            payload_size: 800,
            timestamp: 1633027201,
        },
        PacketHeader {
            source_ip: "192.168.1.101".to_string(),
            destination_ip: "8.8.8.8".to_string(),
            protocol: Protocol::UDP,
            payload_size: 512,
            timestamp: 1633027202,
        },
        PacketHeader {
            source_ip: "8.8.8.8".to_string(),
            destination_ip: "192.168.1.101".to_string(),
            protocol: Protocol::ICMP,
            payload_size: 64,
            timestamp: 1633027203,
        },
    ]
}

fn main() {
    let mut analyzer = PacketAnalyzer::new();
    let packets = create_sample_packets();
    
    for packet in packets {
        analyzer.process_packet(packet);
    }
    
    println!("{}", analyzer.generate_report());
}
use pnet::datalink::{self, Channel, NetworkInterface};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PacketStats {
    pub total_packets: u64,
    pub protocol_counts: HashMap<String, u64>,
    pub start_time: u64,
}

impl PacketStats {
    pub fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    pub fn display_summary(&self) {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.start_time;

        println!("Packet Capture Summary:");
        println!("Duration: {} seconds", duration);
        println!("Total packets: {}", self.total_packets);
        println!("Packets per second: {:.2}", self.total_packets as f64 / duration as f64);
        println!("\nProtocol Distribution:");
        
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

pub fn capture_packets(interface_name: &str, max_packets: u64) -> Result<(), String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    let mut stats = PacketStats::new();
    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop capture\n");

    for i in 0..max_packets {
        match rx.next() {
            Ok(packet) => {
                stats.total_packets += 1;
                process_packet(&packet, &mut stats);
                
                if i % 100 == 0 {
                    print!("\rPackets captured: {}", stats.total_packets);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                continue;
            }
        }
    }

    println!("\n\nCapture complete!");
    stats.display_summary();
    Ok(())
}

fn process_packet(ethernet_data: &[u8], stats: &mut PacketStats) {
    if let Some(ethernet_packet) = EthernetPacket::new(ethernet_data) {
        match ethernet_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                stats.increment_protocol("IPv4");
                if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                    match ipv4_packet.get_next_level_protocol() {
                        IpNextHeaderProtocols::Tcp => {
                            stats.increment_protocol("TCP");
                            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                analyze_tcp_packet(&tcp_packet, stats);
                            }
                        }
                        IpNextHeaderProtocols::Udp => {
                            stats.increment_protocol("UDP");
                            if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                                analyze_udp_packet(&udp_packet, stats);
                            }
                        }
                        IpNextHeaderProtocols::Icmp => {
                            stats.increment_protocol("ICMP");
                        }
                        _ => {
                            stats.increment_protocol("Other-IPv4");
                        }
                    }
                }
            }
            EtherTypes::Ipv6 => {
                stats.increment_protocol("IPv6");
            }
            EtherTypes::Arp => {
                stats.increment_protocol("ARP");
            }
            _ => {
                stats.increment_protocol("Other-Ethernet");
            }
        }
    }
}

fn analyze_tcp_packet(tcp_packet: &TcpPacket, stats: &mut PacketStats) {
    let flags = tcp_packet.get_flags();
    
    if flags & 0x02 != 0 {
        stats.increment_protocol("TCP-SYN");
    }
    if flags & 0x10 != 0 {
        stats.increment_protocol("TCP-ACK");
    }
    if flags & 0x01 != 0 {
        stats.increment_protocol("TCP-FIN");
    }
    if flags & 0x04 != 0 {
        stats.increment_protocol("TCP-RST");
    }
}

fn analyze_udp_packet(udp_packet: &UdpPacket, stats: &mut PacketStats) {
    let payload_len = udp_packet.payload().len();
    
    if payload_len < 100 {
        stats.increment_protocol("UDP-Small");
    } else if payload_len < 1000 {
        stats.increment_protocol("UDP-Medium");
    } else {
        stats.increment_protocol("UDP-Large");
    }
}

pub fn list_interfaces() {
    println!("Available network interfaces:");
    for interface in datalink::interfaces() {
        println!("  {}: {}", interface.name, interface.description);
        for ip in interface.ips {
            println!("    IP: {}", ip);
        }
    }
}