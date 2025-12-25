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

    pub fn get_source_mac_string(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.source_mac[0],
            self.source_mac[1],
            self.source_mac[2],
            self.source_mac[3],
            self.source_mac[4],
            self.source_mac[5]
        )
    }

    pub fn get_destination_mac_string(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.destination_mac[0],
            self.destination_mac[1],
            self.destination_mac[2],
            self.destination_mac[3],
            self.destination_mac[4],
            self.destination_mac[5]
        )
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
            println!("Source MAC: {}", frame.get_source_mac_string());
            println!("Destination MAC: {}", frame.get_destination_mac_string());
            println!("EtherType: 0x{:04x}", frame.ethertype);

            if frame.is_ipv4() {
                if let Some((src_ip, dst_ip)) = frame.extract_ipv4_addresses() {
                    println!("IPv4 Source: {}", src_ip);
                    println!("IPv4 Destination: {}", dst_ip);
                }
            }

            println!("Payload size: {} bytes", frame.payload.len());
        }
        None => println!("Invalid Ethernet frame"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethernet_frame_parsing() {
        let sample_frame = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
            0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // Source MAC
            0x08, 0x00, // EtherType (IPv4)
            0x45, 0x00, 0x00, 0x1c, // IP header
            0x00, 0x01, 0x00, 0x00,
            0x40, 0x11, 0x00, 0x00,
            0xc0, 0xa8, 0x01, 0x01, // Source IP: 192.168.1.1
            0xc0, 0xa8, 0x01, 0x02, // Destination IP: 192.168.1.2
            0x00, 0x00, 0x00, 0x00, // Payload
        ];

        let frame = EthernetFrame::from_bytes(&sample_frame).unwrap();
        assert_eq!(frame.get_source_mac_string(), "aa:bb:cc:dd:ee:ff");
        assert_eq!(frame.get_destination_mac_string(), "00:11:22:33:44:55");
        assert!(frame.is_ipv4());

        let (src_ip, dst_ip) = frame.extract_ipv4_addresses().unwrap();
        assert_eq!(src_ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(dst_ip, Ipv4Addr::new(192, 168, 1, 2));
    }
}