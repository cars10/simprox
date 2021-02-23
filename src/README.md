# simprox

| **Sim**ple **Prox**y Server

Simprox is a simple and easy-to-use local proxy server. 

You can use it to bypass browser restrictions like CORS or invalid SSL certificates when working with external services in your browser.  
It forwards the complete original request to your proxy target and returns the response to your service.

## Usage

```
> simprox --help
simprox 0.1.0
Simple proxy server

USAGE:
    simprox [FLAGS] [OPTIONS] --target_host <host:port>

FLAGS:
        --help               Prints help information
        --skip-ssl-verify    Disable ssl certificate verification
    -V, --version            Prints version information

OPTIONS:
    -h, --listen_host <host:port>    Set the host for the proxy server itself [default: 127.0.0.1:7000]
    -t, --target_host <host:port>    Sets the proxy target (required)
```

### Examples

Listen on `127.0.0.1:7000` (default) and proxy requests to `http://localhost:9200`:

```bash
simprox -t http://localhost:9200 
```

Listen on `0.0.0.0:7000`, proxy requests to `https://localhost:9200` and ignore invalid SSL certificates:

```bash
simprox -h 0.0.0.0:7000 -t https://localhost:9200 --skip-ssl-verify
```

## Download

## Building

Dependencies

* [rust](https://rustup.rs/)
* SSL (depending on your platform). See [rust-native-tls](https://github.com/sfackler/rust-native-tls) for more information
    * Windows: SChannel
    * macOS: Secure Transport
    * Linux: openssl

Build

```bash
git clone git@github.com:cars10/simprox.git
cd simprox
cargo build --release
./target/release/simprox --help
```


## Why

Simprox was originally written for [elasticvue](http://github.com/cars10/elasticvue), so users can access elasticsearch clusters that do not use trusted certificates.

Instead of connecting directly to your cluster `https://my.cluster:9200` in elasticvue, you can use simprox to proxy the requests:  
Simply run `simprox -t https://my.cluster:9200 --skip-ssl-verify` and connect to `http://localhost:7000` in elasticvue.

Yet simprox is completely generic and can be used for any combination of services where you need to proxy requests to bypass browser restrictions.
