use clap::{App, Arg};

pub struct Args {
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
            .get_matches();
        let input = matches.value_of("input").unwrap_or_default().to_string();

        Self { input }
    }
}
