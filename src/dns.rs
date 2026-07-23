// src/dns.rs
// Для резолвинга днс

use std::net::{Ipv4Addr, ToSocketAddrs};

pub fn resolve(host: &str) -> Option<Ipv4Addr> {
    let address = (host, 0).to_socket_addrs().ok()?;
    
    for addr in address {
        if let std::net::IpAddr::V4(ip) = addr.ip() {
            return Some(ip);
        }
    }

    None
}
