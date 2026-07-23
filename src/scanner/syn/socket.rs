use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::socket::{self, AddressFamily, SockFlag, SockProtocol, SockType};
use std::os::fd::{AsRawFd, OwnedFd};

pub fn create_nonblocking_syn_socket() -> nix::Result<OwnedFd> {
    let fd = socket::socket(
        AddressFamily::Inet,
        SockType::Raw,
        SockFlag::empty(),
        Some(SockProtocol::Tcp),
    )?;

    // Пришлось использовать libc для флага IP_HDRINCL, так как в nix, нужного мне флага я не нашел
    let enable: libc::c_int = 1;

    let result = unsafe {
        libc::setsockopt(
            fd.as_raw_fd(),
            libc::IPPROTO_IP,
            libc::IP_HDRINCL,
            &enable as *const _ as *const libc::c_void,
            std::mem::size_of_val(&enable) as libc::socklen_t,
        )
    };

    if result < 0 {
        return Err(nix::Error::last());
    }

    let current_flags = fcntl(&fd, FcntlArg::F_GETFL)?;
    let new_flags = OFlag::from_bits_truncate(current_flags) | OFlag::O_NONBLOCK;
    fcntl(&fd, FcntlArg::F_SETFL(new_flags))?;

    Ok(fd)
}
