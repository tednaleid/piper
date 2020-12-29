use clap::{App, Arg};
use reqwest::Method;
use std::env;
use std::ffi::OsString;

pub struct Args {
    pub input: String,
    pub method: Method,
    pub url: String,
}

impl Args {
    pub fn parse() -> Self {
        Args::parse_from(&mut env::args_os())
    }

    pub fn parse_from<I, T>(itr: I) -> Self
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
                Arg::new("field-separator")
                    .short('F')
                    .long("field-separator")
                    .takes_value(true)
                    .default_value(" ")
                    .hide_default_value(true)
                    .about("The input field separator [default: \" \"]"),
            )
            .arg(
                Arg::new("record-separator")
                    .short('R')
                    .long("record-separator")
                    .takes_value(true)
                    .default_value("\n")
                    .hide_default_value(true)
                    .about("The input record separator [default: \\n]"),
            )
            .arg(
                Arg::new("url")
                    .short('u')
                    .long("url")
                    .takes_value(true)
                    .default_value("{1}")
                    .hide_default_value(true)
                    .long_about(
                        "The url template for making requests, defaults to expecting the first
                           field in the input is the full url (\"{1}\")
                           ",
                    ),
            )
            .arg(
                Arg::new("method")
                    .short('X')
                    .long("method")
                    .takes_value(true)
                    .default_value("GET")
                    .about("The HTTP method for requests, defaults to GET"),
            )
            .get_matches_from(itr);
        let input = matches.value_of("input").unwrap_or_default().to_string();

        let method: Method = matches.value_of_t("method").unwrap_or_else(|e| e.exit());

        let url = matches.value_of("url").unwrap_or_default().to_string();

        Self { input, method, url }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_method_success() {
        assert_eq!(Args::parse_from(vec!["piper"]).method, Method::GET);
        assert_eq!(
            Args::parse_from(vec!["piper", "--method", "GET"]).method,
            Method::GET
        );
        assert_eq!(
            Args::parse_from(vec!["piper", "--method", "POST"]).method,
            Method::POST
        );
        assert_eq!(
            Args::parse_from(vec!["piper", "-X", "DELETE"]).method,
            Method::DELETE
        );

        assert_eq!(
            Args::parse_from(vec!["piper", "-X", "custom"])
                .method
                .as_str(),
            "custom"
        );
    }
}
