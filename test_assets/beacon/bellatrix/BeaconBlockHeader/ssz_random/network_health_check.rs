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
            timeout: Duration::from_secs(2),
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn icmp_ping(&self) -> Result<Duration, String> {
        let socket = IcmpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Failed to bind ICMP socket: {}", e))?;

        let start = Instant::now();
        socket.set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        let payload = [0u8; 32];
        socket.send_to(&payload, (self.target, 0))
            .map_err(|e| format!("Failed to send ICMP packet: {}", e))?;

        let mut buffer = [0u8; 1024];
        socket.recv_from(&mut buffer)
            .map_err(|e| format!("Failed to receive response: {}", e))?;

        Ok(start.elapsed())
    }

    pub fn tcp_port_scan(&self, port: u16) -> Result<bool, String> {
        let addr = (self.target, port);
        match TcpStream::connect_timeout(&addr, self.timeout) {
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
    timestamp: std::time::SystemTime,
}

impl HealthReport {
    fn new(target: IpAddr) -> Self {
        Self {
            target,
            ping_latency: None,
            open_ports: Vec::new(),
            closed_ports: Vec::new(),
            errors: Vec::new(),
            timestamp: std::time::SystemTime::now(),
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
        let latency_str = match self.ping_latency {
            Some(latency) => format!("{:.2}ms", latency.as_secs_f64() * 1000.0),
            None => "N/A".to_string(),
        };

        format!(
            "Target: {}\nPing latency: {}\nOpen ports: {:?}\nClosed ports: {:?}\nErrors: {}\nHealthy: {}",
            self.target,
            latency_str,
            self.open_ports,
            self.closed_ports,
            self.errors.len(),
            self.is_healthy()
        )
    }
}