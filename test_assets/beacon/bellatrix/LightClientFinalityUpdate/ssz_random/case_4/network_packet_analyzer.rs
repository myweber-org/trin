use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct TcpHeader {
    pub source_port: u16,
    pub dest_port: u16,
    pub sequence_number: u32,
    pub acknowledgement_number: u32,
    pub data_offset: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
}

impl TcpHeader {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        Some(TcpHeader {
            source_port: u16::from_be_bytes([data[0], data[1]]),
            dest_port: u16::from_be_bytes([data[2], data[3]]),
            sequence_number: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            acknowledgement_number: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            data_offset: (data[12] >> 4) & 0x0F,
            flags: data[13] & 0x3F,
            window_size: u16::from_be_bytes([data[14], data[15]]),
            checksum: u16::from_be_bytes([data[16], data[17]]),
            urgent_pointer: u16::from_be_bytes([data[18], data[19]]),
        })
    }

    pub fn has_flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

#[derive(Debug)]
pub struct NetworkPacket {
    pub source_ip: Ipv4Addr,
    pub dest_ip: Ipv4Addr,
    pub protocol: u8,
    pub tcp_header: Option<TcpHeader>,
    pub payload: Vec<u8>,
}

impl NetworkPacket {
    pub fn parse_ipv4_packet(data: &[u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        let version = data[0] >> 4;
        if version != 4 {
            return None;
        }

        let ihl = (data[0] & 0x0F) as usize * 4;
        if data.len() < ihl {
            return None;
        }

        let source_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
        let dest_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
        let protocol = data[9];

        let tcp_header = if protocol == 6 && data.len() > ihl {
            TcpHeader::from_bytes(&data[ihl..])
        } else {
            None
        };

        let payload = if data.len() > ihl {
            data[ihl..].to_vec()
        } else {
            Vec::new()
        };

        Some(NetworkPacket {
            source_ip,
            dest_ip,
            protocol,
            tcp_header,
            payload,
        })
    }

    pub fn is_tcp(&self) -> bool {
        self.protocol == 6
    }

    pub fn get_source_port(&self) -> Option<u16> {
        self.tcp_header.as_ref().map(|h| h.source_port)
    }

    pub fn get_dest_port(&self) -> Option<u16> {
        self.tcp_header.as_ref().map(|h| h.dest_port)
    }
}

pub fn analyze_packet(packet_data: &[u8]) {
    match NetworkPacket::parse_ipv4_packet(packet_data) {
        Some(packet) => {
            println!("Packet analysis:");
            println!("  Source IP: {}", packet.source_ip);
            println!("  Destination IP: {}", packet.dest_ip);
            println!("  Protocol: {}", packet.protocol);

            if let Some(tcp) = packet.tcp_header {
                println!("  TCP Header:");
                println!("    Source Port: {}", tcp.source_port);
                println!("    Destination Port: {}", tcp.dest_port);
                println!("    Sequence Number: {}", tcp.sequence_number);
                println!("    Flags: 0x{:02x}", tcp.flags);

                if tcp.has_flag(0x02) {
                    println!("      SYN flag set");
                }
                if tcp.has_flag(0x10) {
                    println!("      ACK flag set");
                }
                if tcp.has_flag(0x01) {
                    println!("      FIN flag set");
                }
            }

            println!("  Payload size: {} bytes", packet.payload.len());
        }
        None => {
            println!("Failed to parse packet");
        }
    }
}