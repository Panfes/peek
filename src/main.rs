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
    
    // Парсим диапазон
    if let Some((start, end)) = Args::parse_range(&args.range) {
            // запускаем сканнирование
            let results = scanner.run(start, end)?;

            // Выводим результаты
            for res in results {
                let status = match res.status {
                    models::PortStatus::Open => "OPEN".style(Style::new().green()),
                    models::PortStatus::Closed => "CLOSED".style(Style::new().red()),
                    models::PortStatus::Timeout => "TIMEOUT".style(Style::new().cyan()),
                };
                println!("{:<8} {}", res.port.bold().white(), status);
            }
        }


    Ok(())
}
