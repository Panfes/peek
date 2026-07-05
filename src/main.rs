mod scanner;
use std::net::Ipv4Addr;
use scanner::Scanner;

fn main() -> nix::Result<()> {
    let mut scanner = Scanner::new(Ipv4Addr::new(192, 168, 0, 180));

    let port_list: [u16; 4] = [22, 40, 443, 8080];
    for port in port_list {
        scanner.run(port)?;
    }

    Ok(())
}
