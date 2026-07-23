// src/net
// Делает сокет

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::socket::{self, AddressFamily, SockFlag, SockType, SockProtocol};
use std::os::fd::OwnedFd;

pub fn create_nonblocking_tcp_socket() -> nix::Result<OwnedFd> {
    let fd = socket::socket(
        AddressFamily::Inet,
        SockType::Stream,
        SockFlag::empty(),
        SockProtocol::Tcp,
    )?;
    
    let current_flags = fcntl(&fd, FcntlArg::F_GETFL)?;
    let new_flags = OFlag::from_bits_truncate(current_flags) | OFlag::O_NONBLOCK;
    fcntl(&fd, FcntlArg::F_SETFL(new_flags))?;

    Ok(fd)
}
