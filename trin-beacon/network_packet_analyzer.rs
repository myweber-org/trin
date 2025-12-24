
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct EthernetFrame {
    pub destination_mac: [u8; 6],
    pub source_mac: [u8; 6],
    pub ethertype: u16,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 14 {
            return None;
        }

        let mut dest_mac = [0u8; 6];
        dest_mac.copy_from_slice(&data[0..6]);

        let mut src_mac = [0u8; 6];
        src_mac.copy_from_slice(&data[6..12]);

        let ethertype = u16::from_be_bytes([data[12], data[13]]);

        let payload = data[14..].to_vec();

        Some(EthernetFrame {
            destination_mac: dest_mac,
            source_mac: src_mac,
            ethertype,
            payload,
        })
    }

    pub fn mac_to_string(mac: &[u8; 6]) -> String {
        mac.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join(":")
    }

    pub fn is_ipv4(&self) -> bool {
        self.ethertype == 0x0800
    }

    pub fn extract_ipv4_addresses(&self) -> Option<(Ipv4Addr, Ipv4Addr)> {
        if !self.is_ipv4() || self.payload.len() < 20 {
            return None;
        }

        let src_ip = Ipv4Addr::new(
            self.payload[12],
            self.payload[13],
            self.payload[14],
            self.payload[15],
        );

        let dst_ip = Ipv4Addr::new(
            self.payload[16],
            self.payload[17],
            self.payload[18],
            self.payload[19],
        );

        Some((src_ip, dst_ip))
    }
}

pub fn analyze_packet(packet_data: &[u8]) {
    match EthernetFrame::from_bytes(packet_data) {
        Some(frame) => {
            println!("Ethernet Frame Analysis:");
            println!("  Destination MAC: {}", EthernetFrame::mac_to_string(&frame.destination_mac));
            println!("  Source MAC: {}", EthernetFrame::mac_to_string(&frame.source_mac));
            println!("  EtherType: 0x{:04x}", frame.ethertype);
            println!("  Payload size: {} bytes", frame.payload.len());

            if frame.is_ipv4() {
                println!("  Protocol: IPv4");
                if let Some((src_ip, dst_ip)) = frame.extract_ipv4_addresses() {
                    println!("  Source IP: {}", src_ip);
                    println!("  Destination IP: {}", dst_ip);
                }
            } else if frame.ethertype == 0x0806 {
                println!("  Protocol: ARP");
            } else if frame.ethertype == 0x86DD {
                println!("  Protocol: IPv6");
            } else {
                println!("  Protocol: Unknown");
            }
        }
        None => println!("Invalid packet data or insufficient length"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethernet_frame_parsing() {
        let sample_packet = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0x08, 0x00,
            0x45, 0x00, 0x00, 0x54, 0x00, 0x00,
            0x40, 0x00, 0x40, 0x01, 0x00, 0x00,
            0xc0, 0xa8, 0x01, 0x01,
            0xc0, 0xa8, 0x01, 0x02,
        ];

        let frame = EthernetFrame::from_bytes(&sample_packet).unwrap();
        
        assert_eq!(frame.destination_mac, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(frame.source_mac, [0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb]);
        assert_eq!(frame.ethertype, 0x0800);
        assert!(frame.is_ipv4());
        
        let (src_ip, dst_ip) = frame.extract_ipv4_addresses().unwrap();
        assert_eq!(src_ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(dst_ip, Ipv4Addr::new(192, 168, 1, 2));
    }
}