// src/models.rs
// Структуры

#[derive(PartialEq)]
pub enum PortStatus {
    Open,
    Closed,
    Timeout,
}

pub struct ScanResult {
    pub port: u16,
    pub status: PortStatus,
}
