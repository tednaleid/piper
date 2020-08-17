use piper::args::Args;
use std::fs::File;
use std::io::{self, BufReader, Result, BufRead};


fn main() -> Result<()> {
    let args = Args::parse();

    let Args { input, output } = args;

    let mut reader: Box<dyn BufRead> = if !input.is_empty() {
        Box::new(BufReader::new(File::open(input)?))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    for url in reader.lines() {
        eprintln!("url: {}", url.unwrap());
    }

    Ok(())
}

#[derive(Debug)]
pub struct RequestContext<'a> {
    url: &'a str,
}

impl PartialEq for RequestContext<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

