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

        let mut target_host = matches
            .value_of("target_host")
            .expect("invalid target")
            .to_string();
        if !target_host.starts_with("http") {
            target_host = format!("http://{}", target_host)
        }

        let skip_ssl_verify = matches.is_present("skip-ssl-verify");

        Config {
            listen_host,
            target_host,
            skip_ssl_verify,
        }
    }
}

fn cli_args() -> ArgMatches<'static> {
    App::new("simprox")
        .version(crate_version!())
        .about("Simple proxy server")
        .arg(
            Arg::with_name("listen_host")
                .long("listen_host")
                .short("h")
                .takes_value(true)
                .value_name("host:port")
                .default_value("127.0.0.1:7000")
                .help("Set the host for the proxy server itself"),
        )
        .arg(
            Arg::with_name("target_host")
                .long("target_host")
                .short("t")
                .takes_value(true)
                .value_name("host:port")
                .required(true)
                .help("Sets the proxy target (required)"),
        )
        .arg(
            Arg::with_name("skip-ssl-verify")
                .long("skip-ssl-verify")
                .help("Disable ssl certificate verification"),
        )
        .get_matches()
}
