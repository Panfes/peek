// src/main.rs
mod cli;
mod dns;
mod formatter;
mod route;
mod scanner;

use clap::Parser;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Instant;
use rand::prelude::*;

use crate::scanner::connect::Scanner;
use crate::scanner::syn::ScanTarget;
use crate::scanner::syn::{
    Scanner as SynScanner,
    Sender,
    create_nonblocking_syn_socket,
};

fn main() -> nix::Result<()> {
    let args = cli::Args::parse();

    let ip = dns::resolve(&args.host)
        .expect("failed to resolve dns");

    if !args.sS {

        let mut scanner = Scanner::new(ip);

        if let Some((start, end)) = cli::Args::parse_range(&args.range) {
            let started_at = Instant::now();

            let results = scanner.run(start, end)?;

            if args.verbose {
                formatter::print_results_verbose(
                    &args.host,
                    &ip,
                    started_at.elapsed(),
                    &results,
                );
            } else {
                formatter::print_results(
                    &args.host,
                    &ip,
                    started_at.elapsed(),
                    &results,
                );
            }
        }
    } else {

        // заменить на автоматическое определение айпи устройства
        // HARDCODE!!!
        let source_ip = Ipv4Addr::new(172, 28, 33, 102);
        let source_port = rand::rng().random_range(1024..65535);

        let source = SocketAddrV4::new(source_ip, source_port);

        let fd = create_nonblocking_syn_socket()?;

        let sender = Sender::new(fd);
        let mut scanner = SynScanner::new(sender);

        let started_at_for_formatter = Instant::now();

        if let Some((start, end)) = cli::Args::parse_range(&args.range) {

            for port in start..=end {
                let destination = SocketAddrV4::new(ip, port);
                let target = ScanTarget::new(source, destination);
                scanner.add_target(target);
            }
            scanner.event_loop()?;

            formatter::print_half_open_results(&args.host, &ip, started_at_for_formatter.elapsed(), &scanner.targets);
        }
    }

    Ok(())
}
