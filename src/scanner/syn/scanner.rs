// SYN scanner

use nix::errno::Errno;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use pnet_packet::tcp::{TcpFlags, TcpPacket};
use nix::sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags};

use crate::scanner::syn::sender::Sender;
use super::packet;
use crate::scanner::syn::target::ScanTarget;
use crate::scanner::syn::target::ScanState;


const TIMEOUT: Duration = Duration::from_millis(150);
const WINDOW_SIZE: usize = 5000;

pub struct Scanner {
    sender: Sender,
    next_to_send: usize,
    pub targets: Vec<ScanTarget>,

    // u32 - sequence number, usize - индекс в targets
    index: HashMap<u32, usize>, 
    
    // u32 - sequence, Instant - время начала
    pending: HashMap<u32, Instant>,
}

impl Scanner {
    pub fn new(sender: Sender) -> Self {
        Self {
            sender,
            next_to_send: 0,
            targets: Vec::new(),
            index: HashMap::new(),
            pending: HashMap::new(),
        }
    }
    
    pub fn add_target(&mut self, target: ScanTarget) {
        let index = self.targets.len();
        let sequence = target.sequence_number;

        self.targets.push(target);
        self.index.insert(sequence, index);
    }   

    pub fn update_state(
        &mut self,
        sequence: u32,
        tcp: &TcpPacket,
    ) {

        let flags = tcp.get_flags();

        let Some(target_index) = self.index.get(&sequence) else {
            return;
        };

        let target_index = *target_index;

        if flags & TcpFlags::SYN != 0 &&
            flags & TcpFlags::ACK != 0
        {
            self.targets[target_index].state = ScanState::Open;

        } else if flags & TcpFlags::RST != 0 {
            self.targets[target_index].state = ScanState::Closed;
        }
        self.pending.remove(&sequence);
        
    }

    fn check_timeouts(&mut self) {
        let now = Instant::now();

        let expired: Vec<u32> = self.pending
            .iter()
            .filter_map(|(&sequence, &started_at)| {
                if now.duration_since(started_at) >= TIMEOUT {
                    Some(sequence)
                } else {
                    None
                }
            })
            .collect();
        

        for sequence in expired {
                self.index.get(&sequence);
            if let Some(target_index) = self.index.get(&sequence) {
                self.targets[*target_index].state = ScanState::Filtered;
            }
            self.pending.remove(&sequence);
        }
    }

    pub fn send_pack(&mut self) {
            while self.next_to_send < self.targets.len()
                && self.pending.len() < WINDOW_SIZE
            {
            let index = self.next_to_send;
            let source = self.targets[index].source;
            let destination = self.targets[index].destination;
            let sequence_number = self.targets[index].sequence_number;

            match self.sender.send_syn(
                source,
                destination,
                sequence_number,
            ) {
                Ok(_) => {
                    self.targets[index].state = ScanState::Pending;

                    self.pending.insert(
                        sequence_number,
                        Instant::now(),
                    );
                    self.next_to_send += 1;
                }

                Err(Errno::EAGAIN) => {
                    break;
                }
                Err(Errno::ENETUNREACH) => {
                    panic!("\nNetwork unreacheble. Please check your network connection!\n");
                }
                Err(err) => {
                    self.targets[index].state = ScanState::Filtered;
                    self.next_to_send += 1;
                    eprintln!("{:?}", err);
                }
            }
        }
    } 

    pub fn event_loop(&mut self) -> nix::Result<()> {

        let epoll = Epoll::new(EpollCreateFlags::empty())?;

        epoll.add(
            self.sender.get_fd(),
            EpollEvent::new(EpollFlags::EPOLLIN, 1),
        )?;

        let mut buf = [0u8; 4096];
        let mut events = [EpollEvent::empty(); 16];

        while self.next_to_send < self.targets.len()
            || !self.pending.is_empty() {

            self.check_timeouts();

            self.send_pack();

            let count = epoll.wait(
                &mut events,
                1u16,
            )?;

            if count == 0 {
                continue;
            }

            for event in &events[..count] {
                if !event.events().contains(EpollFlags::EPOLLIN) {
                    continue;
                }
                loop {
                    match self.sender.recieve(&mut buf) {
                        Ok(size) => {
                            let Some((_, tcp)) = packet::parse_packet(&buf[..size]) else {
                                continue;
                            };

                            let sequence = tcp.get_acknowledgement().wrapping_sub(1);
                            self.update_state(sequence, &tcp);
                        }
                        Err(Errno::EAGAIN) => {
                            break;
                        }
                        Err(err) => {
                            println!("FATAL: {:?}", err);
                            return Err(err);
                        }
                    }
                }
            }
        }

        Ok(())
    } 
}
