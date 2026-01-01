use std::net::UdpSocket;
use std::str;

struct PacketAnalyzer {
    socket: UdpSocket,
    packet_count: u32,
}

impl PacketAnalyzer {
    fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        Ok(PacketAnalyzer {
            socket,
            packet_count: 0,
        })
    }

    fn capture_packets(&mut self, buffer_size: usize) {
        let mut buffer = vec![0u8; buffer_size];
        
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, src_addr)) => {
                    self.packet_count += 1;
                    println!("Packet #{} from {} ({} bytes)", 
                           self.packet_count, src_addr, size);
                    
                    if let Ok(payload) = str::from_utf8(&buffer[..size]) {
                        if !payload.is_empty() {
                            println!("Payload preview: {}...", 
                                   &payload[..payload.len().min(20)]);
                        }
                    }
                    
                    self.analyze_packet(&buffer[..size]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    eprintln!("Error receiving packet: {}", e);
                    break;
                }
            }
        }
    }

    fn analyze_packet(&self, data: &[u8]) {
        if data.len() >= 20 {
            let protocol_type = match data[9] {
                6 => "TCP",
                17 => "UDP",
                1 => "ICMP",
                _ => "Unknown",
            };
            println!("Protocol: {}, Total length: {}", protocol_type, data.len());
            
            let source_port = u16::from_be_bytes([data[0], data[1]]);
            let dest_port = u16::from_be_bytes([data[2], data[3]]);
            println!("Ports: {} -> {}", source_port, dest_port);
        }
    }

    fn get_statistics(&self) -> (u32, String) {
        (self.packet_count, format!("Analyzer active on {:?}", self.socket.local_addr().unwrap()))
    }
}

fn main() {
    let mut analyzer = PacketAnalyzer::new("127.0.0.1:8080")
        .expect("Failed to create packet analyzer");
    
    println!("Starting packet capture...");
    analyzer.capture_packets(1024);
    
    let stats = analyzer.get_statistics();
    println!("Capture complete. Packets captured: {}", stats.0);
}