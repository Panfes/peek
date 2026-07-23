use std::net::SocketAddrV4;
use std::os::fd::AsRawFd;
use std::os::fd::OwnedFd;
use nix::sys::socket::SockaddrIn;
use nix::sys::socket;
use nix::sys::socket::{sendto, MsgFlags};
use nix::errno::Errno;

use super::packet;
    
pub struct Sender {
    fd: OwnedFd,
}

impl Sender {
    pub fn new(fd: OwnedFd) -> Self {
        Self { fd, }
    }

    pub fn get_fd(&self) -> &OwnedFd {
        &self.fd
    }

    pub fn send_syn(
        &self,
        source: SocketAddrV4,
        destination: SocketAddrV4,
        sequence_number: u32,
    ) -> Result<usize, Errno> {
        let mut buffer = [0u8; 40];
        packet::create_syn_packet(&mut buffer, source, destination, sequence_number);

        let addr = SockaddrIn::from(destination);

        sendto(
            self.fd.as_raw_fd(),
            &buffer,
            &addr,
            MsgFlags::empty(),
        )
    }

    pub fn recieve(&self, buf: &mut [u8]) -> nix::Result<usize> {
        match socket::recvfrom::<SockaddrIn>(self.fd.as_raw_fd(), buf) {
            Ok((size, _)) => Ok(size),

            Err(Errno::EAGAIN) => {
                Err(Errno::EAGAIN)
            }

            Err(err) => {
                Err(err)
            }
        }
    }
}
