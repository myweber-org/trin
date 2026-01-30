use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use pnet_datalink;

fn list_network_interfaces() -> Vec<String> {
    let interfaces = pnet_datalink::interfaces();
    let mut interface_names = Vec::new();

    for interface in interfaces {
        let name = interface.name.clone();
        let mac = match interface.mac {
            Some(mac_addr) => format!("{}", mac_addr),
            None => "No MAC".to_string(),
        };

        let mut ipv4_addrs = Vec::new();
        let mut ipv6_addrs = Vec::new();

        for ip_network in interface.ips {
            match ip_network.ip() {
                IpAddr::V4(ipv4) => {
                    ipv4_addrs.push(ipv4.to_string());
                }
                IpAddr::V6(ipv6) => {
                    ipv6_addrs.push(ipv6.to_string());
                }
            }
        }

        let interface_info = format!(
            "Interface: {}, MAC: {}, IPv4: {:?}, IPv6: {:?}",
            name, mac, ipv4_addrs, ipv6_addrs
        );
        interface_names.push(interface_info);
    }

    interface_names
}

fn main() {
    let interfaces = list_network_interfaces();
    println!("Available network interfaces:");
    for interface in interfaces {
        println!("{}", interface);
    }
}