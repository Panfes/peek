use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // ip - internet protocol
    pub host: String,

    // port range
    pub range: String,

    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    pub fn parse_range(range: &str) -> Option<(u16, u16)> {
        let (start, end) = range.split_once('-')?;
        let start = start.parse().ok()?;
        let end = end.parse().ok()?;

        Some((start, end))
    }
}
