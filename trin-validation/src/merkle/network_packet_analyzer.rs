use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    device_name: String,
    capture: Option<Capture<pcap::Active>>,
}

impl PacketAnalyzer {
    pub fn new(device_name: &str) -> Self {
        PacketAnalyzer {
            device_name: device_name.to_string(),
            capture: None,
        }
    }

    pub fn start_capture(&mut self) -> Result<(), Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == self.device_name)
            .ok_or("Network device not found")?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        self.capture = Some(capture);
        println!("Started capturing on {}", self.device_name);
        Ok(())
    }

    pub fn analyze_next_packet(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(capture) = &mut self.capture {
            match capture.next_packet() {
                Ok(packet) => {
                    println!("Packet captured: {} bytes", packet.header.len);
                    self.print_packet_info(&packet);
                }
                Err(e) => return Err(Box::new(e)),
            }
        } else {
            return Err("Capture not started".into());
        }
        Ok(())
    }

    fn print_packet_info(&self, packet: &pcap::Packet) {
        let data = packet.data;
        if data.len() >= 14 {
            let dest_mac = &data[0..6];
            let src_mac = &data[6..12];
            let ethertype = u16::from_be_bytes([data[12], data[13]]);
            
            println!("Destination MAC: {:02x?}", dest_mac);
            println!("Source MAC: {:02x?}", src_mac);
            println!("EtherType: 0x{:04x}", ethertype);
            
            match ethertype {
                0x0800 => println!("Protocol: IPv4"),
                0x0806 => println!("Protocol: ARP"),
                0x86DD => println!("Protocol: IPv6"),
                _ => println!("Protocol: Unknown"),
            }
        }
    }

    pub fn stop_capture(&mut self) {
        self.capture = None;
        println!("Stopped capture on {}", self.device_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = PacketAnalyzer::new("eth0");
        assert_eq!(analyzer.device_name, "eth0");
        assert!(analyzer.capture.is_none());
    }
}