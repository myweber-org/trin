use std::net::{IpAddr, IcmpSocket, TcpStream};
use std::time::{Duration, Instant};
use std::thread;

const PING_TIMEOUT: Duration = Duration::from_secs(2);
const TCP_TIMEOUT: Duration = Duration::from_secs(3);

pub struct NetworkProbe {
    target: IpAddr,
}

impl NetworkProbe {
    pub fn new(target: IpAddr) -> Self {
        NetworkProbe { target }
    }

    pub fn icmp_ping(&self) -> Result<Duration, String> {
        let socket = IcmpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Failed to create ICMP socket: {}", e))?;

        let start = Instant::now();
        socket.set_read_timeout(Some(PING_TIMEOUT))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        let payload = [0u8; 32];
        socket.send_to(&payload, (self.target, 0))
            .map_err(|e| format!("Failed to send ping: {}", e))?;

        let mut buffer = [0u8; 1024];
        socket.recv(&mut buffer)
            .map_err(|e| format!("Failed to receive response: {}", e))?;

        Ok(start.elapsed())
    }

    pub fn tcp_port_scan(&self, port: u16) -> bool {
        match TcpStream::connect_timeout(&(self.target, port).into(), TCP_TIMEOUT) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn concurrent_port_scan(&self, ports: &[u16]) -> Vec<(u16, bool)> {
        let mut handles = vec![];
        for &port in ports {
            let target = self.target;
            let handle = thread::spawn(move || {
                let result = match TcpStream::connect_timeout(&(target, port).into(), TCP_TIMEOUT) {
                    Ok(_) => true,
                    Err(_) => false,
                };
                (port, result)
            });
            handles.push(handle);
        }

        handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_localhost_scan() {
        let probe = NetworkProbe::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let ports = vec![80, 443, 8080];
        let results = probe.concurrent_port_scan(&ports);
        
        for (port, status) in results {
            println!("Port {}: {}", port, if status { "OPEN" } else { "CLOSED" });
        }
    }
}