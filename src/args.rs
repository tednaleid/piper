use clap::{App, Arg};

pub struct Args {
    pub output: String,
    pub input: String,
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
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .help("Write output to a file instead of stdout"),
            )
            .get_matches();
        let input = matches.value_of("input").unwrap_or_default().to_string();
        let output = matches.value_of("output").unwrap_or_default().to_string();

        Self { input, output }
    }
}
