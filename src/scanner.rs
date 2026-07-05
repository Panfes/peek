use nix::errno::Errno;
use std::os::fd::{AsRawFd, OwnedFd};
use nix::sys::socket::{self, connect};
use nix::sys::socket::{AddressFamily, SockFlag, SockType, SockaddrIn};
use std::net::{Ipv4Addr, SocketAddrV4};

pub enum PortStatus {
    Open,
    Closed,
}

pub struct Scanner {
    ip: Ipv4Addr,
}

impl Scanner {
    pub fn new(ip: Ipv4Addr) -> Self {
        Self { ip }
    }

    fn create_tcp_socket() -> nix::Result<OwnedFd> {
        let fd = socket::socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )?;
        Ok(fd)
    }
    
    pub fn scan_port(&self, port: u16) -> nix::Result<PortStatus> {
        let sockaddr = SockaddrIn::from(SocketAddrV4::new(self.ip, port));
        let fd = Self::create_tcp_socket()?;

        match connect(fd.as_raw_fd(), &sockaddr) {
            Ok(_) => Ok(PortStatus::Open),
            Err(Errno::ECONNREFUSED) => Ok(PortStatus::Closed),
            Err(e) => Err(e),
        }
    }
}

