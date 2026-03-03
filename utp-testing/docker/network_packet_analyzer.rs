use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    capture: Capture<pcap::Active>,
}

impl PacketAnalyzer {
    pub fn new(interface_name: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == interface_name)
            .ok_or_else(|| format!("Interface {} not found", interface_name))?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        Ok(PacketAnalyzer { capture })
    }

    pub fn start_capture(&mut self, packet_count: usize) -> Result<(), Box<dyn Error>> {
        println!("Starting packet capture on interface...");
        
        for i in 0..packet_count {
            match self.capture.next_packet() {
                Ok(packet) => {
                    println!("Packet {}: {} bytes captured", i + 1, packet.header.len);
                    self.analyze_packet(&packet);
                }
                Err(e) => eprintln!("Error capturing packet: {}", e),
            }
        }
        
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        let data = packet.data;
        
        if data.len() >= 14 {
            let eth_type = u16::from_be_bytes([data[12], data[13]]);
            
            match eth_type {
                0x0800 => println!("  IPv4 Packet"),
                0x0806 => println!("  ARP Packet"),
                0x86DD => println!("  IPv6 Packet"),
                _ => println!("  Unknown Ethernet Type: 0x{:04x}", eth_type),
            }
            
            if eth_type == 0x0800 && data.len() >= 34 {
                let protocol = data[23];
                let src_ip = format!("{}.{}.{}.{}", data[26], data[27], data[28], data[29]);
                let dst_ip = format!("{}.{}.{}.{}", data[30], data[31], data[32], data[33]);
                
                println!("  Source IP: {}", src_ip);
                println!("  Destination IP: {}", dst_ip);
                
                match protocol {
                    6 => println!("  Protocol: TCP"),
                    17 => println!("  Protocol: UDP"),
                    1 => println!("  Protocol: ICMP"),
                    _ => println!("  Protocol: {}", protocol),
                }
            }
        }
        
        println!("  Raw data (first 64 bytes): {:02x?}", &data[..data.len().min(64)]);
        println!();
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  {}: {}", device.name, device.desc.unwrap_or_default());
    }
    Ok(())
}