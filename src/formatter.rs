// src/formatter.rs
// Для красивого вывода

use std::net::Ipv4Addr;
use std::time::Duration;

use crate::models::{ScanResult, PortStatus};
use crate::scanner::syn::target::{ScanState, ScanTarget};
use owo_colors::{OwoColorize, Style};

pub fn print_results(host: &str, host_ip: &Ipv4Addr, elapsed: Duration, results: &Vec<ScanResult>) {
    print_data(host, host_ip);
    println!();

    println!("{:<8} {}", "PORT".bold(), "STATE".bold());
    println!("{}", "-".repeat(15));

    for res in results {
        if res.status == PortStatus::Open {
            println!("{:<8} {}", res.port.bold(), "OPEN".green());
        }
    }
    print_summary(elapsed, results);
}

pub fn print_half_open_results(host: &str, host_ip: &Ipv4Addr, elapsed: Duration, results: &Vec<ScanTarget>) {
    print_data(host, host_ip);
    println!();

    println!("{:<8} {}", "PORT".bold(), "STATE".bold());
    println!("{}", "-".repeat(15));

    for res in results {
        if res.state == ScanState::Open {
            println!("{:<8} {}", res.destination.port(), "Open".green());
        }
    }
    print_half_open_summary(elapsed, results);
}


pub fn print_results_verbose(host: &str, host_ip: &Ipv4Addr, elapsed: Duration, results: &Vec<ScanResult>) {
    println!();
    print_data(host, host_ip);

    println!();
    for res in results {
        let status = match res.status {
            PortStatus::Open => "OPEN".style(Style::new().green()),
            PortStatus::Closed => "CLOSED".style(Style::new().red()),
            PortStatus::Timeout => "TIMEOUT".style(Style::new().cyan()),
        };
        println!(
            "{:<8} {}", res.port.bold().white(),
            status
        );
    }
    print_summary(elapsed, results);
}

fn print_data(host: &str, host_ip: &Ipv4Addr) {
    println!("{}\n", "peek - TCP Scanner".bold());
    println!("Target: {:<5}", host.underline());
    println!("IP: {:<5}", host_ip);
}



fn print_summary(elapsed: Duration, results: &Vec<ScanResult>) {
    let mut open_ports = 0;
    let mut closed_ports = 0;
    let mut timeout_ports = 0;

    for res in results {
        match res.status {
            PortStatus::Open => open_ports += 1,
            PortStatus::Closed => closed_ports += 1,
            PortStatus::Timeout => timeout_ports += 1,
        }
    }

    println!();
    println!("{} {:.3}s", "Scan finished in".bold(), elapsed.as_secs_f64());

    println!(
        "{:<18} {}",
        "✔ Open ports:".green(),
        open_ports.to_string().bold()
    );

    println!(
        "{:<18} {}",
        "✘ Closed ports:".red(),
        closed_ports.to_string().bold()
    );

    println!(
        "{:<18} {}",
        "? Timeout ports:".cyan(),
        timeout_ports.to_string().bold()
    );
}

fn print_half_open_summary(elapsed: Duration, results: &Vec<ScanTarget>) {
    let mut open_ports = 0;
    let mut closed_ports = 0;
    let mut timeout_ports = 0;

    for res in results {
        match res.state {
            ScanState::Open => open_ports += 1,
            ScanState::Closed => closed_ports += 1,
            ScanState::Filtered => timeout_ports +=1,
            ScanState::Waiting => (),
            ScanState::Pending => {
                println!("{}", "CONNECTION IN PENDING AFTER FINISH".red());
            }
        }
    }

    println!();
    println!("{} {:.3}s", "Scan finished in".bold(), elapsed.as_secs_f64());

    println!(
        "{:<18} {}",
        "✔ Open ports:".green(),
        open_ports.to_string().bold()
    );

    println!(
        "{:<18} {}",
        "✘ Closed ports:".red(),
        closed_ports.to_string().bold()
    );

    println!(
        "{:<18} {}",
        "? Timeout ports:".cyan(),
        timeout_ports.to_string().bold()
    );
}

