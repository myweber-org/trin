
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub protocol: Protocol,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub timestamp: SystemTime,
    pub payload_length: usize,
}

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

impl NetworkPacket {
    pub fn new(
        source_ip: IpAddr,
        dest_ip: IpAddr,
        protocol: Protocol,
        payload: Vec<u8>,
    ) -> Self {
        let header = PacketHeader {
            source_ip,
            destination_ip: dest_ip,
            protocol,
            source_port: None,
            destination_port: None,
            timestamp: SystemTime::now(),
            payload_length: payload.len(),
        };

        NetworkPacket {
            header,
            payload,
            metadata: HashMap::new(),
        }
    }

    pub fn with_ports(mut self, src_port: u16, dest_port: u16) -> Self {
        self.header.source_port = Some(src_port);
        self.header.destination_port = Some(dest_port);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn is_local_traffic(&self) -> bool {
        match (self.header.source_ip, self.header.destination_ip) {
            (IpAddr::V4(src), IpAddr::V4(dst)) => {
                src.is_private() || dst.is_private() || src.is_loopback() || dst.is_loopback()
            }
            (IpAddr::V6(src), IpAddr::V6(dst)) => {
                src.is_loopback() || dst.is_loopback()
            }
            _ => false,
        }
    }

    pub fn matches_filter(&self, filter: &PacketFilter) -> bool {
        filter.matches(self)
    }
}

pub struct PacketFilter {
    pub source_ip: Option<IpAddr>,
    pub destination_ip: Option<IpAddr>,
    pub protocol: Option<Protocol>,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub min_payload_size: Option<usize>,
    pub max_payload_size: Option<usize>,
}

impl PacketFilter {
    pub fn new() -> Self {
        PacketFilter {
            source_ip: None,
            destination_ip: None,
            protocol: None,
            source_port: None,
            destination_port: None,
            min_payload_size: None,
            max_payload_size: None,
        }
    }

    pub fn matches(&self, packet: &NetworkPacket) -> bool {
        if let Some(ref src_ip) = self.source_ip {
            if &packet.header.source_ip != src_ip {
                return false;
            }
        }

        if let Some(ref dest_ip) = self.destination_ip {
            if &packet.header.destination_ip != dest_ip {
                return false;
            }
        }

        if let Some(ref protocol) = self.protocol {
            if &packet.header.protocol != protocol {
                return false;
            }
        }

        if let Some(src_port) = self.source_port {
            if packet.header.source_port != Some(src_port) {
                return false;
            }
        }

        if let Some(dest_port) = self.destination_port {
            if packet.header.destination_port != Some(dest_port) {
                return false;
            }
        }

        if let Some(min_size) = self.min_payload_size {
            if packet.payload.len() < min_size {
                return false;
            }
        }

        if let Some(max_size) = self.max_payload_size {
            if packet.payload.len() > max_size {
                return false;
            }
        }

        true
    }

    pub fn with_source_ip(mut self, ip: IpAddr) -> Self {
        self.source_ip = Some(ip);
        self
    }

    pub fn with_destination_ip(mut self, ip: IpAddr) -> Self {
        self.destination_ip = Some(ip);
        self
    }

    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn with_source_port(mut self, port: u16) -> Self {
        self.source_port = Some(port);
        self
    }

    pub fn with_destination_port(mut self, port: u16) -> Self {
        self.destination_port = Some(port);
        self
    }
}

pub struct PacketAnalyzer {
    packets: Vec<NetworkPacket>,
    statistics: PacketStatistics,
}

#[derive(Debug, Default)]
pub struct PacketStatistics {
    pub total_packets: usize,
    pub tcp_packets: usize,
    pub udp_packets: usize,
    pub icmp_packets: usize,
    pub other_protocols: usize,
    pub total_payload_size: usize,
    pub local_traffic_count: usize,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packets: Vec::new(),
            statistics: PacketStatistics::default(),
        }
    }

    pub fn add_packet(&mut self, packet: NetworkPacket) {
        self.update_statistics(&packet);
        self.packets.push(packet);
    }

    fn update_statistics(&mut self, packet: &NetworkPacket) {
        self.statistics.total_packets += 1;
        self.statistics.total_payload_size += packet.payload.len();

        if packet.is_local_traffic() {
            self.statistics.local_traffic_count += 1;
        }

        match packet.header.protocol {
            Protocol::TCP => self.statistics.tcp_packets += 1,
            Protocol::UDP => self.statistics.udp_packets += 1,
            Protocol::ICMP => self.statistics.icmp_packets += 1,
            Protocol::Other(_) => self.statistics.other_protocols += 1,
        }
    }

    pub fn filter_packets(&self, filter: &PacketFilter) -> Vec<&NetworkPacket> {
        self.packets
            .iter()
            .filter(|packet| packet.matches_filter(filter))
            .collect()
    }

    pub fn get_statistics(&self) -> &PacketStatistics {
        &self.statistics
    }

    pub fn clear(&mut self) {
        self.packets.clear();
        self.statistics = PacketStatistics::default();
    }

    pub fn average_payload_size(&self) -> f64 {
        if self.statistics.total_packets == 0 {
            0.0
        } else {
            self.statistics.total_payload_size as f64 / self.statistics.total_packets as f64
        }
    }
}

impl Default for PacketFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PacketAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}extern crate pnet;

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
                    "{}:{} -> {}:{} [TCP] Flags: {:?} Seq: {} Ack: {} Win: {} Len: {}",
                    source,
                    tcp_packet.get_source(),
                    destination,
                    tcp_packet.get_destination(),
                    tcp_packet.get_flags(),
                    tcp_packet.get_sequence(),
                    tcp_packet.get_acknowledgement(),
                    tcp_packet.get_window(),
                    tcp_packet.payload().len()
                );
            }
        }
        17 => {
            if let Some(udp_packet) = UdpPacket::new(payload) {
                println!(
                    "{}:{} -> {}:{} [UDP] Length: {} Checksum: {}",
                    source,
                    udp_packet.get_source(),
                    destination,
                    udp_packet.get_destination(),
                    udp_packet.get_length(),
                    udp_packet.get_checksum()
                );
            }
        }
        _ => println!(
            "{} -> {} [Protocol {}] Length: {}",
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
            "IPv4 Packet: {} -> {} Protocol: {} TTL: {}",
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

    let mut packet_count = 0;
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    packet_count += 1;
                    println!("\nPacket #{}", packet_count);
                    println!("Source MAC: {}", ethernet_packet.get_source());
                    println!("Destination MAC: {}", ethernet_packet.get_destination());

                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => handle_ipv4_packet(&ethernet_packet),
                        EtherTypes::Ipv6 => println!("IPv6 Packet (not decoded)"),
                        EtherTypes::Arp => println!("ARP Packet"),
                        _ => println!("Unknown EtherType: {:?}", ethernet_packet.get_ethertype()),
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