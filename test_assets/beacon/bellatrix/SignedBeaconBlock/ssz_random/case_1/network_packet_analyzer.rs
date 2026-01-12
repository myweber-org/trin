
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
use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    capture: Capture<pcap::Active>,
}

impl PacketAnalyzer {
    pub fn new(interface: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == interface)
            .ok_or("Interface not found")?;
        
        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;
        
        Ok(PacketAnalyzer { capture })
    }

    pub fn start_capture(&mut self, packet_count: i32) -> Result<(), Box<dyn Error>> {
        let mut count = 0;
        while let Ok(packet) = self.capture.next_packet() {
            println!("Packet {} captured:", count + 1);
            println!("  Timestamp: {:?}", packet.header.ts);
            println!("  Length: {} bytes", packet.header.len);
            println!("  Captured length: {} bytes", packet.header.caplen);
            
            self.analyze_packet(&packet.data);
            
            count += 1;
            if count >= packet_count {
                break;
            }
        }
        Ok(())
    }

    fn analyze_packet(&self, data: &[u8]) {
        if data.len() >= 14 {
            let dest_mac = &data[0..6];
            let src_mac = &data[6..12];
            let ethertype = u16::from_be_bytes([data[12], data[13]]);
            
            println!("  Ethernet Header:");
            println!("    Destination MAC: {:02X?}", dest_mac);
            println!("    Source MAC: {:02X?}", src_mac);
            println!("    EtherType: 0x{:04X}", ethertype);
            
            match ethertype {
                0x0800 => self.analyze_ipv4(&data[14..]),
                0x0806 => println!("    Protocol: ARP"),
                0x86DD => println!("    Protocol: IPv6"),
                _ => println!("    Unknown protocol"),
            }
        }
    }

    fn analyze_ipv4(&self, data: &[u8]) {
        if data.len() >= 20 {
            let version = data[0] >> 4;
            let ihl = (data[0] & 0x0F) * 4;
            let protocol = data[9];
            let src_ip = &data[12..16];
            let dst_ip = &data[16..20];
            
            println!("  IPv4 Header:");
            println!("    Version: {}", version);
            println!("    IHL: {} bytes", ihl);
            println!("    Protocol: {}", protocol);
            println!("    Source IP: {}.{}.{}.{}", src_ip[0], src_ip[1], src_ip[2], src_ip[3]);
            println!("    Destination IP: {}.{}.{}.{}", dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3]);
            
            match protocol {
                6 => self.analyze_tcp(&data[ihl as usize..]),
                17 => self.analyze_udp(&data[ihl as usize..]),
                _ => println!("    Unknown transport protocol"),
            }
        }
    }

    fn analyze_tcp(&self, data: &[u8]) {
        if data.len() >= 20 {
            let src_port = u16::from_be_bytes([data[0], data[1]]);
            let dst_port = u16::from_be_bytes([data[2], data[3]]);
            let data_offset = (data[12] >> 4) * 4;
            
            println!("  TCP Header:");
            println!("    Source Port: {}", src_port);
            println!("    Destination Port: {}", dst_port);
            println!("    Data Offset: {} bytes", data_offset);
            
            if data.len() > data_offset as usize {
                let payload = &data[data_offset as usize..];
                if !payload.is_empty() {
                    println!("    Payload length: {} bytes", payload.len());
                }
            }
        }
    }

    fn analyze_udp(&self, data: &[u8]) {
        if data.len() >= 8 {
            let src_port = u16::from_be_bytes([data[0], data[1]]);
            let dst_port = u16::from_be_bytes([data[2], data[3]]);
            let length = u16::from_be_bytes([data[4], data[5]]);
            
            println!("  UDP Header:");
            println!("    Source Port: {}", src_port);
            println!("    Destination Port: {}", dst_port);
            println!("    Length: {} bytes", length);
            
            if data.len() > 8 {
                let payload = &data[8..];
                if !payload.is_empty() {
                    println!("    Payload length: {} bytes", payload.len());
                }
            }
        }
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  {}: {}", device.name, device.desc.unwrap_or_default());
    }
    Ok(())
}