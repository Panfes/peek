// src/scanner/connect/scanner.rs

use crate::models::{PortStatus, ScanResult};
use crate::net::create_nonblocking_tcp_socket;
use nix::errno::Errno;
use nix::poll::{self, PollFd, PollFlags};
use nix::sys::socket::{connect, getsockopt, sockopt, SockaddrIn};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::time::{Instant, Duration};

const CONNECTION_TIMEOUT_MS: u16 = 400;

// Соединения которые не завершились
pub struct PendingConnection {
    fd: OwnedFd,
    port: u16,
    started_at: Instant,
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
            // Соединение сразу готово
            Ok(_) => Ok(Some(ScanResult {
                port,
                status: PortStatus::Open,
            })),

            // Соединение начало устанавливаться 
            Err(Errno::EINPROGRESS) => {
                self.pending.push(PendingConnection { fd, port, started_at: Instant::now(), });
                Ok(None)
            }

            // Сразу отказ
            Err(Errno::ECONNREFUSED) => Ok(Some(ScanResult {
                port,
                status: PortStatus::Closed,
            })),

            // Сразу таймаут
            Err(Errno::ETIMEDOUT) => Ok(Some(ScanResult {
                port,
                status: PortStatus::Timeout,
            })),

            Err(err) => {
                eprintln!("{err}");
                todo!();
            }
        }
    }

    // Важно! эта функция только ждет, а потом возвращает то что получила
    pub fn wait(&mut self) -> nix::Result<(Vec<u16>, Vec<u16>)> {

        // Превращение обычного дескриптора, в структуру для poll()
        let mut fds: Vec<PollFd> = self
            .pending
            .iter()
            .map(|c| PollFd::new(c.fd.as_fd(), PollFlags::POLLOUT))
            .collect();

        poll::poll(&mut fds, CONNECTION_TIMEOUT_MS)?;

        // Готовые порты
        let mut ready_ports = Vec::new();

        for (i, fd) in fds.iter().enumerate() {
            if let Some(events) = fd.revents() 
                && events.contains(PollFlags::POLLOUT) {
                    ready_ports.push(self.pending[i].port);
            }
        }

        let timeout = Duration::from_millis(CONNECTION_TIMEOUT_MS as u64);
        
        let mut timeout_ports: Vec<u16> = Vec::new();

        // Выявление порта с таймаутом
        for (index, pending) in self.pending.iter().enumerate() {
            let duration = pending.started_at.elapsed();
            if duration >= timeout {
                timeout_ports.push(self.pending[index].port);
            }
        }

        // Нужно для того чтобы обработать каждый элемент кортежа, и вытянуть из него статус
        Ok((ready_ports, timeout_ports))
    }

    // Важно! функция возвращает результат для портов с таймаутом, и удаляет порт из пендинг
    pub fn timeout_results(&mut self, timeout_ports: Vec<u16>) -> nix::Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        for timeout_port in timeout_ports {
            if let Some(index) = self.pending
                .iter()
                .position(|conn| conn.port == timeout_port) {
                    // Уадляем соединение из pending, потому-что оно больше не должно
                    // участвовать в следующих вызовах poll()
                    let conn = self.pending.remove(index);

                    results.push(ScanResult {
                        port: conn.port,
                        status: PortStatus::Timeout,
                    });
            }
        }
        Ok(results)
    }

    pub fn ready_results(&mut self, ready_ports: Vec<u16>) -> nix::Result<Vec<ScanResult>> {
        let mut results = Vec::new();
        
        for ready_port in ready_ports {
            if let Some(index) = self.pending
                .iter()
                .position(|conn| conn.port == ready_port) {
                    let conn = self.pending.remove(index);
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
            }

        Ok(results)
    }

    pub fn run(&mut self, start: u16, end: u16) -> nix::Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        for port in start..=end {
            if let Some(res) = self.start_connection(port)? {
                results.push(res);
            }
        }

        while !self.pending.is_empty() {
            let (ready, timeout) = self.wait()?;

            let mut ready_batch = self.ready_results(ready)?;
            let mut timeout_batch = self.timeout_results(timeout)?;

            results.append(&mut ready_batch);
            results.append(&mut timeout_batch);
        }
        Ok(results)
    }
}
