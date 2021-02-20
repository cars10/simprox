use clap::{App, Arg, ArgMatches};

#[derive(Debug)]
pub struct Config {
    pub listen_host: String,
    pub target_host: String,
    pub skip_ssl_verify: bool,
}

impl Config {
    pub fn build() -> Self {
        let matches = cli_args();
        let listen_host = matches
            .value_of("listen_host")
            .expect("invalid listen_host")
            .to_string();
        let target_host = matches
            .value_of("target_host")
            .expect("invalid target")
            .to_string();
        let skip_ssl_verify = matches.is_present("skip-ssl-verify");

        Config {
            listen_host,
            target_host,
            skip_ssl_verify,
        }
    }
}

fn cli_args() -> ArgMatches<'static> {
    App::new("sipose")
        .version(crate_version!())
        .author("Carsten Koenig <carstenkoenig92@gmail.com>")
        .about("Simple proxy server")
        .arg(
            Arg::with_name("listen_host")
                .long("listen_host")
                .short("h")
                .takes_value(true)
                .value_name("listen_host")
                .default_value("127.0.0.1:7000")
                .help("Sets the ip/port where the proxy server itself should listen"),
        )
        .arg(
            Arg::with_name("target_host")
                .long("target_host")
                .short("t")
                .takes_value(true)
                .value_name("target_host")
                .required(true)
                .help("Sets the proxy target (required)"),
        )
        .arg(
            Arg::with_name("skip-ssl-verify")
                .long("skip-ssl-verify")
                .help("Disable ssl cert and hostname verification"),
        )
        .get_matches()
}
