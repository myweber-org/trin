
use std::process::Command;
use std::time::Duration;

pub struct PingConfig {
    pub target: String,
    pub count: u8,
    pub timeout_secs: u64,
}

pub fn check_connectivity(config: &PingConfig) -> Result<bool, String> {
    let output = Command::new("ping")
        .arg("-c")
        .arg(config.count.to_string())
        .arg("-W")
        .arg(config.timeout_secs.to_string())
        .arg(&config.target)
        .output()
        .map_err(|e| format!("Failed to execute ping: {}", e))?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(false)
    }
}

pub fn perform_health_check() {
    let config = PingConfig {
        target: String::from("8.8.8.8"),
        count: 4,
        timeout_secs: 2,
    };

    match check_connectivity(&config) {
        Ok(true) => println!("Network connectivity check passed."),
        Ok(false) => println!("Network connectivity check failed."),
        Err(e) => println!("Error during check: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connectivity() {
        let config = PingConfig {
            target: String::from("127.0.0.1"),
            count: 2,
            timeout_secs: 1,
        };
        let result = check_connectivity(&config);
        assert!(result.is_ok());
    }
}