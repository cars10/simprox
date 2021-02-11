use clap::{App, Arg, ArgMatches};
use regex::Regex;

#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub use_ssl: bool,
    pub skip_ssl_verify: bool
}

impl Config {
    pub fn build() -> Self {
        let matches = cli_args();
        let host = matches.value_of("host").expect("invalid host").to_string();

        let ssl_re = Regex::new(r"^https://").unwrap();
        let use_ssl = ssl_re.is_match(&host);

        let skip_ssl_verify = matches.is_present("skip-ssl-verify");

        Config {
            host,
            use_ssl,
            skip_ssl_verify
        }
    }
}

fn cli_args() -> ArgMatches<'static> {
    App::new("proxy")
        .version("0.1")
        .author("Carsten Koenig <carstenkoenig92@gmail.com>")
        .about("Simple proxy server")
        .arg(
            Arg::with_name("host")
                .long("host")
                .short("h")
                .takes_value(true)
                .value_name("host")
                .required(true)
                .help("Sets the proxy host (required)"),
        )
        .arg(
            Arg::with_name("skip-ssl-verify")
                .long("skip-ssl-verify")
                .help("Disable ssl cert and hostname verification"),
        )
        .get_matches()
}
