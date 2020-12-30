use anyhow::Result;
use clap::{App, Arg};
use reqwest::Method;
use std::env;
use std::ffi::OsString;

pub struct Args {
    pub input: String,
    pub method: Method,
    pub url: String,
    pub parallel: usize,
    pub timeout_seconds: u64,
}

impl Args {
    pub fn parse() -> Result<Self> {
        Args::parse_from(&mut env::args_os())
    }

    pub fn parse_from<I, T>(itr: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = App::new("piper")
            .after_long_help(
                "
  Templates can have:
    - single fields: {2} - field 2 in the input record
    - multiple fields: {1,3} - fields 1 through 3
    - unbounded fields: {3,} - field 3, 4, 5, ...
    - literal string values - values not in {} are treated as literals

  example template:
    \"http://{1}.org/{2}?values={3}\"

  input records (space delimited fields and newline delimited records):
    httpbin uuid a,b,c
    httpbin image d,e,f

  will generate the urls:
    http://httpbin.org/uuid?values=a,b,c
    http://httpbin.org/image?values=d,e,f",
            )
            .arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .takes_value(true)
                    .about("A file to read input records from, defaults to reading from stdin"),
            )
            .arg(
                Arg::new("parallel")
                    .short('P')
                    .long("parallel")
                    .takes_value(true)
                    .default_value("1")
                    .about("The maximum number of requests to send in parallel. At 1, requests will be sent serially, above 1 they will be sent in parallel and will resolve in the order they are completed"),
            )
            .arg(
                Arg::new("timeout")
                    .long("timeout")
                    .takes_value(true)
                    .default_value("10")
                    .about("Request timeout in seconds"),
            )

            // .arg(
            //     Arg::new("field-separator")
            //         .short('F')
            //         .long("field-separator")
            //         .takes_value(true)
            //         .default_value(" ")
            //         .hide_default_value(true)
            //         .about("The input field separator [default: \" \"]"),
            // )
            // .arg(
            //     Arg::new("record-separator")
            //         .short('R')
            //         .long("record-separator")
            //         .takes_value(true)
            //         .default_value("\n")
            //         .hide_default_value(true)
            //         .about("The input record separator [default: \\n]"),
            // )
            .arg(
                Arg::new("url")
                    .short('u')
                    .long("url")
                    .takes_value(true)
                    .default_value("{1}")
                    .about(
                        "The url template for making requests, defaults to expecting the first field in the input is the full url",
                    ),
            )
            .arg(
                Arg::new("method")
                    .short('X')
                    .long("method")
                    .takes_value(true)
                    .default_value("GET")
                    .about("The HTTP method for requests (GET/POST/PUT/...), can specify an arbitrary string"),
            )
            .get_matches_from(itr);
        let input = matches.value_of("input").unwrap_or_default().to_string();

        let parallel: usize = matches.value_of_t("parallel")?;
        let timeout_seconds: u64 = matches.value_of_t("timeout")?;

        let method: Method = matches.value_of_t("method").unwrap_or_else(|e| e.exit());

        let url = matches.value_of("url").unwrap_or_default().to_string();

        Ok(Self {
            input,
            method,
            url,
            parallel,
            timeout_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn parse_method_success() -> Result<()> {
        assert_eq!(Args::parse_from(vec!["piper"])?.method, Method::GET);
        assert_eq!(
            Args::parse_from(vec!["piper", "--method", "GET"])?.method,
            Method::GET
        );
        assert_eq!(
            Args::parse_from(vec!["piper", "--method", "POST"])?.method,
            Method::POST
        );
        assert_eq!(
            Args::parse_from(vec!["piper", "-X", "DELETE"])?.method,
            Method::DELETE
        );

        assert_eq!(
            Args::parse_from(vec!["piper", "-X", "custom"])?
                .method
                .as_str(),
            "custom"
        );
        Ok(())
    }

    #[test]
    fn parse_parallel() -> Result<()> {
        assert_eq!(Args::parse_from(vec!["piper"])?.parallel, 1);
        assert_eq!(Args::parse_from(vec!["piper", "-P", "20"])?.parallel, 20);
        assert_eq!(Args::parse_from(vec!["piper", "-P", "a"]).is_err(), true);
        Ok(())
    }

    #[test]
    fn parse_timeout() -> Result<()> {
        assert_eq!(Args::parse_from(vec!["piper"])?.timeout_seconds, 10);
        assert_eq!(
            Args::parse_from(vec!["piper", "--timeout", "20"])?.timeout_seconds,
            20
        );
        assert_eq!(
            Args::parse_from(vec!["piper", "--timeout", "a"]).is_err(),
            true
        );
        Ok(())
    }
}
