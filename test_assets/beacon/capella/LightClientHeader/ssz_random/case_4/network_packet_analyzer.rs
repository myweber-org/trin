use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    device: String,
    capture: Option<Capture<pcap::Active>>,
}

impl PacketAnalyzer {
    pub fn new(device_name: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == device_name)
            .ok_or("Device not found")?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .open()?;

        Ok(PacketAnalyzer {
            device: device_name.to_string(),
            capture: Some(capture),
        })
    }

    pub fn start_capture(&mut self, packet_count: i32) -> Result<(), Box<dyn Error>> {
        let capture = self.capture.as_mut().ok_or("Capture not initialized")?;
        
        for i in 0..packet_count {
            let packet = capture.next_packet()?;
            println!("Packet {}: {} bytes captured", i + 1, packet.header.len);
            self.analyze_packet(&packet);
        }
        
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        let data = packet.data;
        if data.len() >= 14 {
            let eth_type = u16::from_be_bytes([data[12], data[13]]);
            match eth_type {
                0x0800 => println!("  Protocol: IPv4"),
                0x0806 => println!("  Protocol: ARP"),
                0x86DD => println!("  Protocol: IPv6"),
                _ => println!("  Protocol: Unknown (0x{:04x})", eth_type),
            }
        }
    }

    pub fn get_statistics(&self) -> Result<String, Box<dyn Error>> {
        let capture = self.capture.as_ref().ok_or("Capture not initialized")?;
        let stats = capture.stats()?;
        
        Ok(format!(
            "Device: {}\nPackets received: {}\nPackets dropped: {}\nPackets dropped by interface: {}",
            self.device, stats.received, stats.dropped, stats.if_dropped
        ))
    }
}

impl Drop for PacketAnalyzer {
    fn drop(&mut self) {
        if let Some(capture) = &self.capture {
            let _ = capture.stats();
        }
    }
}