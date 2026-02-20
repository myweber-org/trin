use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use pnet_datalink;

pub fn list_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    
    for iface in pnet_datalink::interfaces() {
        let mut info = format!("Interface: {}", iface.name);
        
        if let Some(mac) = iface.mac {
            info.push_str(&format!("\n  MAC: {}", mac));
        }
        
        for ip in iface.ips {
            match ip.ip() {
                IpAddr::V4(ipv4) => {
                    info.push_str(&format!("\n  IPv4: {}", ipv4));
                }
                IpAddr::V6(ipv6) => {
                    info.push_str(&format!("\n  IPv6: {}", ipv6));
                }
            }
        }
        
        if iface.is_up() {
            info.push_str("\n  Status: UP");
        } else {
            info.push_str("\n  Status: DOWN");
        }
        
        if iface.is_loopback() {
            info.push_str("\n  Type: Loopback");
        } else if iface.is_broadcast() {
            info.push_str("\n  Type: Broadcast");
        } else if iface.is_point_to_point() {
            info.push_str("\n  Type: Point-to-Point");
        }
        
        if let Some(mtu) = iface.mtu {
            info.push_str(&format!("\n  MTU: {}", mtu));
        }
        
        interfaces.push(info);
    }
    
    interfaces
}

pub fn find_interface_by_ip(target_ip: IpAddr) -> Option<String> {
    for iface in pnet_datalink::interfaces() {
        for ip in iface.ips {
            if ip.ip() == target_ip {
                return Some(iface.name.clone());
            }
        }
    }
    None
}

pub fn get_interface_mac(interface_name: &str) -> Option<String> {
    for iface in pnet_datalink::interfaces() {
        if iface.name == interface_name {
            if let Some(mac) = iface.mac {
                return Some(mac.to_string());
            }
        }
    }
    None
}