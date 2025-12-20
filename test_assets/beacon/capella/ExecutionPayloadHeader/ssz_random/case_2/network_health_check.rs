
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetworkTarget {
    pub host: String,
    pub ip: IpAddr,
    pub ports: Vec<u16>,
}

impl NetworkTarget {
    pub fn new(host: &str, ports: &[u16]) -> Result<Self, String> {
        let ip = resolve_host(host)?;
        Ok(Self {
            host: host.to_string(),
            ip,
            ports: ports.to_vec(),
        })
    }
}

pub struct HealthChecker {
    timeout: Duration,
    ping_count: u8,
}

impl HealthChecker {
    pub fn new(timeout_secs: u64, ping_count: u8) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
            ping_count,
        }
    }

    pub fn check_target(&self, target: &NetworkTarget) -> HealthReport {
        let ping_result = self.perform_ping(&target.ip);
        let port_results = self.check_ports(target);
        
        HealthReport {
            target: target.clone(),
            ping_latency: ping_result,
            port_statuses: port_results,
            timestamp: chrono::Utc::now(),
        }
    }

    fn perform_ping(&self, ip: &IpAddr) -> Option<Duration> {
        if cfg!(target_os = "windows") {
            self.ping_windows(ip)
        } else {
            self.ping_unix(ip)
        }
    }

    fn ping_windows(&self, ip: &IpAddr) -> Option<Duration> {
        let output = Command::new("ping")
            .arg("-n")
            .arg(self.ping_count.to_string())
            .arg("-w")
            .arg(self.timeout.as_millis().to_string())
            .arg(ip.to_string())
            .output()
            .ok()?;

        parse_ping_output(&String::from_utf8_lossy(&output.stdout))
    }

    fn ping_unix(&self, ip: &IpAddr) -> Option<Duration> {
        let output = Command::new("ping")
            .arg("-c")
            .arg(self.ping_count.to_string())
            .arg("-W")
            .arg(self.timeout.as_secs().to_string())
            .arg(ip.to_string())
            .output()
            .ok()?;

        parse_ping_output(&String::from_utf8_lossy(&output.stdout))
    }

    fn check_ports(&self, target: &NetworkTarget) -> Vec<PortStatus> {
        target.ports.iter().map(|port| {
            let addr = SocketAddr::new(target.ip, *port);
            let result = TcpStream::connect_timeout(&addr, self.timeout);
            
            PortStatus {
                port: *port,
                is_open: result.is_ok(),
                error: result.err().map(|e| e.to_string()),
            }
        }).collect()
    }
}

fn parse_ping_output(output: &str) -> Option<Duration> {
    let lines: Vec<&str> = output.lines().collect();
    
    for line in lines {
        if line.contains("time=") || line.contains("time<") {
            if let Some(start) = line.find("time=") {
                let sub = &line[start + 5..];
                if let Some(end) = sub.find("ms") {
                    let time_str = &sub[..end];
                    if let Ok(millis) = time_str.parse::<u64>() {
                        return Some(Duration::from_millis(millis));
                    }
                }
            }
        }
    }
    None
}

fn resolve_host(host: &str) -> Result<IpAddr, String> {
    if let Ok(ip) = IpAddr::from_str(host) {
        return Ok(ip);
    }
    
    match dns_lookup::lookup_host(host) {
        Ok(ips) => ips.first().cloned().ok_or_else(|| "No IP found".to_string()),
        Err(e) => Err(format!("DNS resolution failed: {}", e)),
    }
}

#[derive(Debug, Clone)]
pub struct PortStatus {
    pub port: u16,
    pub is_open: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HealthReport {
    pub target: NetworkTarget,
    pub ping_latency: Option<Duration>,
    pub port_statuses: Vec<PortStatus>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthReport {
    pub fn is_healthy(&self) -> bool {
        self.ping_latency.is_some() && self.port_statuses.iter().all(|p| p.is_open)
    }
    
    pub fn summary(&self) -> String {
        let ping_status = if self.ping_latency.is_some() {
            format!("Ping: {}ms", self.ping_latency.unwrap().as_millis())
        } else {
            "Ping: Failed".to_string()
        };
        
        let open_ports: Vec<String> = self.port_statuses
            .iter()
            .filter(|p| p.is_open)
            .map(|p| p.port.to_string())
            .collect();
            
        let closed_ports: Vec<String> = self.port_statuses
            .iter()
            .filter(|p| !p.is_open)
            .map(|p| p.port.to_string())
            .collect();
            
        format!(
            "Target: {} ({})\n{}\nOpen ports: {}\nClosed ports: {}\nHealthy: {}",
            self.target.host,
            self.target.ip,
            ping_status,
            if open_ports.is_empty() { "None".to_string() } else { open_ports.join(", ") },
            if closed_ports.is_empty() { "None".to_string() } else { closed_ports.join(", ") },
            self.is_healthy()
        )
    }
}