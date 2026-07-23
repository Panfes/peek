use std::net::SocketAddrV4;
use rand::prelude::*;

#[derive(PartialEq, Debug)]
pub enum ScanState {
    Waiting,
    Pending,
    Open,
    Closed,
    Filtered,
}

pub struct ScanTarget {
    pub source: SocketAddrV4,
    pub destination: SocketAddrV4,
    pub state: ScanState,
    pub sequence_number: u32,
}

impl ScanTarget {
    pub fn new(source: SocketAddrV4, destination: SocketAddrV4) -> Self {
        let mut rng = rand::rng();
        let sequence_number: u32 = rng.random();

        Self {
            source,
            destination,
            sequence_number,
            state: ScanState::Waiting,
        }
    }
}

