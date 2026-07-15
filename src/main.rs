mod models;
mod net;
mod scanner;
mod cli;
mod dns;
mod formatter;

use clap::Parser;
use std::time::Instant;

fn main() -> nix::Result<()> {
    let args = cli::Args::parse(); 
    
    let ip = dns::resolve(&args.host)
        .expect("failed to resolve dns");

    let mut scanner = scanner::Scanner::new(ip);
    
    if let Some((start, end)) = cli::Args::parse_range(&args.range) {
            
            let started_at = Instant::now();

            let results = scanner.run(start, end)?;

            if !args.verbose {
                formatter::print_results(&args.host, &ip, started_at.elapsed(), &results);
            } else {
                formatter::print_results_verbose(&args.host, &ip, started_at.elapsed(), &results);
            }

        }

    Ok(())
}
