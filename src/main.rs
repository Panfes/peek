mod models;
mod net;
mod scanner;
mod cli;

// use std::net::Ipv4Addr;
use scanner::Scanner;
use clap::Parser;
use cli::Args;
use owo_colors::{OwoColorize, Style};

fn main() -> nix::Result<()> {
    // Создаем экземпляр сканера для нужного IP
    let args = Args::parse(); 

    let mut scanner = Scanner::new(args.ip);
    
    // Запускаем сканирование портов
    if let Some((start, end)) = Args::parse_range(&args.range) {

        for port in start..=end {
            let results = scanner.run(port)?;

            // Выводим результаты
            for res in results {
                let status = match res.status {
                    models::PortStatus::Open => "Open".style(Style::new().green()),
                    models::PortStatus::Closed => "Closed".style(Style::new().red()),
                    models::PortStatus::Timeout => "Timeout".style(Style::new().cyan()),
                };
                println!("Port {}: {}", res.port.bold().white(), status);
            }
        }

        }


    Ok(())
}
