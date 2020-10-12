use clap::{App, Arg};

pub struct Args {
    pub input: String,
    pub url: String,
}

impl Args {
    pub fn parse() -> Self {
        let matches = App::new("piper")
            .arg(
                Arg::with_name("input")
                    .short("i")
                    .long("input")
                    .takes_value(true)
                    .help("Read input from a file instead of stdin"),
            )
            .arg(
                Arg::with_name("field-separator")
                    .short("F")
                    .long("field-separator")
                    .takes_value(true)
                    .default_value(" ")
                    .help("The input field separator, defaults to space"),
            )
            .arg(
                Arg::with_name("record-separator")
                    .short("R")
                    .long("record-separator")
                    .takes_value(true)
                    .default_value("\n")
                    .help("The input record separator, defaults to newline"),
            )
            .arg(
                Arg::with_name("url")
                    .short("u")
                    .long("url")
                    .takes_value(true)
                    .default_value("{1}")
                    .help("The url template for making requests, defaults the first field"),
            )
            .get_matches();
        let input = matches.value_of("input").unwrap_or_default().to_string();
        let url = matches.value_of("url").unwrap_or_default().to_string();

        Self { input, url }
    }
}
