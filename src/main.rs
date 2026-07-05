mod scanner;
use std::net::Ipv4Addr;
use owo_colors::OwoColorize;
use scanner::{PortStatus, Scanner};

fn main() -> nix::Result<()> {
    let scanner = Scanner::new(Ipv4Addr::new(192, 168, 0, 180));

    let port_list = [22, 80, 443, 8080];

    for port in port_list {
        match scanner.scan_port(port) {
            Ok(PortStatus::Open) => {
                println!("{}: {}", port.white().bold(), "Open".green());
            }

            Ok(PortStatus::Closed) => {
                println!("{}: {}", port.white().bold(), "Closed".red());
            }

            Err(e) => {
                println!("{port}: {}", e.red());
            }
        }
    }

    Ok(())
}
