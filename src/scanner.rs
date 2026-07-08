use crate::models::{PortStatus, ScanResult};
use crate::net::create_nonblocking_tcp_socket;
use nix::errno::Errno;
use nix::poll::{self, PollFd, PollFlags};
use nix::sys::socket::{connect, getsockopt, sockopt, SockaddrIn};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};

pub struct PendingConnection {
    fd: OwnedFd,
    port: u16,
}

pub struct Scanner {
    ip: Ipv4Addr,
    pending: Vec<PendingConnection>,
}

impl Scanner {
    pub fn new(ip: Ipv4Addr) -> Self {
        Self { ip, pending: Vec::new() }
    }

    pub fn start_connection(&mut self, port: u16) -> nix::Result<Option<ScanResult>> {
        let sockaddr = SockaddrIn::from(SocketAddrV4::new(self.ip, port));
        let fd = create_nonblocking_tcp_socket()?;

        match connect(fd.as_raw_fd(), &sockaddr) {
            Ok(_) => Ok(Some(ScanResult {
                port,
                status: PortStatus::Open,
            })),
            Err(Errno::EINPROGRESS) => {
                self.pending.push(PendingConnection { fd, port });
                Ok(None)
            }
            Err(Errno::ECONNREFUSED) => Ok(Some(ScanResult {
                port,
                status: PortStatus::Closed,
            })),
            Err(Errno::ETIMEDOUT) => Ok(Some(ScanResult {
                port,
                status: PortStatus::Timeout,
            })),
            Err(_) => todo!(),
        }
    }

    pub fn wait(&mut self) -> nix::Result<Vec<usize>> {
        let mut fds: Vec<PollFd> = self
            .pending
            .iter()
            .map(|c| PollFd::new(c.fd.as_fd(), PollFlags::POLLOUT))
            .collect();

        poll::poll(&mut fds, 1000u16)?;

        let mut ready = Vec::new();
        for (i, fd) in fds.iter().enumerate() {
            if let Some(events) = fd.revents() 
                && events.contains(PollFlags::POLLOUT) {
                    ready.push(i);
            }
        }
        Ok(ready)
    }

    pub fn connection_results(&mut self, ready: Vec<usize>) -> nix::Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        for i in ready.into_iter().rev() {
            let conn = self.pending.remove(i);
            let err = getsockopt(&conn.fd, sockopt::SocketError)?;

            if err == 0 {
                results.push(ScanResult {
                    port: conn.port,
                    status: PortStatus::Open,
                });
            } else {
                results.push(ScanResult {
                    port: conn.port,
                    status: PortStatus::Closed,
                });
            }
        }
        Ok(results)
    }

    pub fn run(&mut self, port: u16) -> nix::Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        if let Some(res) = self.start_connection(port)? {
            results.push(res);

        }

        while !self.pending.is_empty() {
            let ready = self.wait()?;
            let mut batch = self.connection_results(ready)?;
            results.append(&mut batch);
        }
        Ok(results)
    }
}
