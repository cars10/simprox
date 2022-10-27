use clap::{Command, Arg, ArgMatches};

#[derive(Debug)]
pub struct Config {
    pub listen: String,
    pub target_host: String,
    pub skip_ssl_verify: bool,
}

impl Config {
    pub fn build() -> Self {
        let matches = cli_args();
        let listen = matches
            .get_one::<String>("listen")
            .expect("invalid listen")
            .to_string();

        let mut target_host = matches
            .get_one::<String>("target_host")
            .expect("invalid target")
            .to_string();
        if !target_host.starts_with("http") {
            target_host = format!("http://{}", target_host)
        }

        let skip_ssl_verify = matches.get_flag("skip-ssl-verify");

        Config {
            listen,
            target_host,
            skip_ssl_verify,
        }
    }
}

fn cli_args() -> ArgMatches {
    Command::new("simprox")
        .version(crate_version!())
        .about("Simple proxy server")
        .arg(
            Arg::new("listen")
                .env_os("LISTEN")
                .long("listen")
                .short('l')
                .value_name("host:port")
                .default_value("127.0.0.1:7000")
                .help("Set the host for the proxy server itself"),
        )
        .arg(
            Arg::new("target_host")
                .env_os("TARGET_HOST")
                .long("target-host")
                .short('t')
                .value_name("host:port")
                .required(true)
                .help("Sets the proxy target (required)"),
        )
        .arg(
            Arg::new("skip-ssl-verify")
                .default_value("false")
                .env_os("SKIP_SSL_VERIFY")
                .long("skip-ssl-verify")
                .value_parser(clap::value_parser!(bool))
                .help("Disable ssl certificate verification"),
        )
        .get_matches()
}
