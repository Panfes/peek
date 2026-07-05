use nix::errno::Errno;
use nix::sys::socket::{self, connect, getsockopt, sockopt};
use nix::sys::socket::{AddressFamily, SockFlag, SockType, SockaddrIn};
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::poll::{self, PollFd, PollFlags};
use owo_colors::OwoColorize;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};

// pub enum PortStatus {
//     Open,
//     Closed,
//     Timeout,
// }

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
        Self { ip, pending: Vec::new(), }
    }

    fn create_nonblocking_tcp_socket() -> nix::Result<OwnedFd> {
        let fd = socket::socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )?;
        
        let current_flags = fcntl(&fd, FcntlArg::F_GETFL)?;
        let new_flags = OFlag::from_bits_truncate(current_flags) | OFlag::O_NONBLOCK;
        fcntl(&fd, FcntlArg::F_SETFL(new_flags))?;

        Ok(fd)
    }
    
    pub fn start_connection(&mut self, port: u16) -> nix::Result<()> {
        let sockaddr = SockaddrIn::from(SocketAddrV4::new(self.ip, port));
        let fd = Self::create_nonblocking_tcp_socket()?;

        match connect(fd.as_raw_fd(), &sockaddr) {
            Ok(_) => {
                println!("{}: {}", port.bold().white(), "Open".green());
                Ok(())
            }
            Err(Errno::EINPROGRESS) => {
                self.pending.push(PendingConnection { fd, port });
                Ok(())
            }
            Err(Errno::ECONNREFUSED) => {
                println!("{}: {}", port.bold().white(), "Closed".red());
                Ok(())
            }
            Err(e) => {
                eprintln!("{}: {}", port.bold().white(), e.red());
                Err(e)
            }
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

    pub fn connection_results(&mut self, ready: Vec<usize>) -> nix::Result<()> {
        for i in ready.into_iter().rev() {
            let conn = &self.pending.remove(i);

            let err = getsockopt(&conn.fd, sockopt::SocketError)?;

            if err == 0 {
                println!("{}: {}", conn.port.bold().white(), "Open".green());
            } else {
                println!("{}: {}", conn.port.bold().white(), "Closed".red());
            }
        }
        Ok(())
    }

    pub fn run(&mut self, port: u16) -> nix::Result<()> {
        self.start_connection(port)?;
        
        while !self.pending.is_empty() {
            let ready = self.wait()?;
            self.connection_results(ready)?;
        }

        Ok(())
    }
}
