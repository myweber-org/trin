
use std::process::Command;
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr};
use std::str;

const PACKET_COUNT: usize = 4;
const TIMEOUT_SECONDS: u64 = 2;

#[derive(Debug)]
pub struct PingResult {
    pub destination: String,
    pub packets_transmitted: usize,
    pub packets_received: usize,
    pub packet_loss: f32,
    pub avg_latency_ms: Option<f32>,
    pub reachable: bool,
}

pub fn check_host(host: &str) -> Result<PingResult, String> {
    let start_time = Instant::now();
    
    let output = Command::new("ping")
        .arg("-c")
        .arg(PACKET_COUNT.to_string())
        .arg("-W")
        .arg(TIMEOUT_SECONDS.to_string())
        .arg(host)
        .output()
        .map_err(|e| format!("Failed to execute ping: {}", e))?;

    let elapsed = start_time.elapsed();
    
    if !output.status.success() {
        return Ok(PingResult {
            destination: host.to_string(),
            packets_transmitted: PACKET_COUNT,
            packets_received: 0,
            packet_loss: 100.0,
            avg_latency_ms: None,
            reachable: false,
        });
    }

    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in ping output: {}", e))?;

    parse_ping_output(host, output_str, elapsed)
}

fn parse_ping_output(host: &str, output: &str, elapsed: Duration) -> Result<PingResult, String> {
    let lines: Vec<&str> = output.lines().collect();
    
    let mut packets_transmitted = 0;
    let mut packets_received = 0;
    let mut packet_loss = 100.0;
    let mut avg_latency = None;

    for line in lines {
        if line.contains("packets transmitted") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                if let Some(transmitted_part) = parts.get(0) {
                    if let Some(count_str) = transmitted_part.split_whitespace().next() {
                        packets_transmitted = count_str.parse().unwrap_or(PACKET_COUNT);
                    }
                }
                
                if let Some(received_part) = parts.get(1) {
                    if let Some(count_str) = received_part.split_whitespace().next() {
                        packets_received = count_str.parse().unwrap_or(0);
                    }
                }
                
                if let Some(loss_part) = parts.get(2) {
                    if let Some(loss_str) = loss_part.split_whitespace().next() {
                        if let Ok(loss_value) = loss_str.parse::<f32>() {
                            packet_loss = loss_value;
                        }
                    }
                }
            }
        }
        
        if line.contains("min/avg/max/mdev") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let latency_parts: Vec<&str> = parts[1].trim().split('/').collect();
                if latency_parts.len() >= 2 {
                    if let Ok(avg) = latency_parts[1].parse::<f32>() {
                        avg_latency = Some(avg);
                    }
                }
            }
        }
    }

    let reachable = packets_received > 0 && packet_loss < 100.0;
    
    Ok(PingResult {
        destination: host.to_string(),
        packets_transmitted,
        packets_received,
        packet_loss,
        avg_latency_ms: avg_latency,
        reachable,
    })
}

pub fn check_default_gateway() -> Result<PingResult, String> {
    let gateway = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    check_host(&gateway.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_check() {
        let result = check_host("127.0.0.1");
        assert!(result.is_ok());
        
        if let Ok(ping_result) = result {
            assert_eq!(ping_result.destination, "127.0.0.1");
            assert!(ping_result.reachable);
            assert!(ping_result.packet_loss < 100.0);
        }
    }
}
use std::net::{IpAddr, IcmpSocket, TcpStream};
use std::time::{Duration, Instant};
use std::thread;

pub struct NetworkProbe {
    target: IpAddr,
    timeout: Duration,
}

impl NetworkProbe {
    pub fn new(target: IpAddr) -> Self {
        Self {
            target,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn icmp_ping(&self) -> Result<Duration, String> {
        let socket = IcmpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Failed to bind ICMP socket: {}", e))?;

        let start = Instant::now();
        socket.set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        let payload = [0u8; 56];
        socket.send_to(&payload, (self.target, 0))
            .map_err(|e| format!("Failed to send ICMP packet: {}", e))?;

        let mut buffer = [0u8; 1024];
        socket.recv_from(&mut buffer)
            .map_err(|e| format!("Failed to receive response: {}", e))?;

        Ok(start.elapsed())
    }

    pub fn tcp_port_scan(&self, port: u16) -> Result<bool, String> {
        let addr = format!("{}:{}", self.target, port);
        match TcpStream::connect_timeout(&addr.parse().unwrap(), self.timeout) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(false),
            Err(e) => Err(format!("Connection error: {}", e)),
        }
    }

    pub fn perform_health_check(&self, ports: &[u16]) -> HealthReport {
        let mut report = HealthReport::new(self.target);

        match self.icmp_ping() {
            Ok(latency) => report.set_ping_latency(latency),
            Err(e) => report.add_error(e),
        }

        for &port in ports {
            thread::sleep(Duration::from_millis(100));
            match self.tcp_port_scan(port) {
                Ok(true) => report.add_open_port(port),
                Ok(false) => report.add_closed_port(port),
                Err(e) => report.add_error(format!("Port {}: {}", port, e)),
            }
        }

        report
    }
}

pub struct HealthReport {
    target: IpAddr,
    ping_latency: Option<Duration>,
    open_ports: Vec<u16>,
    closed_ports: Vec<u16>,
    errors: Vec<String>,
}

impl HealthReport {
    fn new(target: IpAddr) -> Self {
        Self {
            target,
            ping_latency: None,
            open_ports: Vec::new(),
            closed_ports: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn set_ping_latency(&mut self, latency: Duration) {
        self.ping_latency = Some(latency);
    }

    fn add_open_port(&mut self, port: u16) {
        self.open_ports.push(port);
    }

    fn add_closed_port(&mut self, port: u16) {
        self.closed_ports.push(port);
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn is_healthy(&self) -> bool {
        self.ping_latency.is_some() && self.errors.is_empty()
    }

    pub fn summary(&self) -> String {
        let mut summary = format!("Target: {}\n", self.target);
        
        if let Some(latency) = self.ping_latency {
            summary.push_str(&format!("Ping latency: {:?}\n", latency));
        } else {
            summary.push_str("Ping: Failed\n");
        }

        summary.push_str(&format!("Open ports: {:?}\n", self.open_ports));
        summary.push_str(&format!("Closed ports: {:?}\n", self.closed_ports));
        
        if !self.errors.is_empty() {
            summary.push_str(&format!("Errors: {:?}\n", self.errors));
        }

        summary.push_str(&format!("Overall health: {}", self.is_healthy()));
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_localhost_scan() {
        let probe = NetworkProbe::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .with_timeout(Duration::from_secs(2));

        let report = probe.perform_health_check(&[80, 443, 8080]);
        
        println!("{}", report.summary());
        assert!(report.ping_latency.is_some());
    }
}