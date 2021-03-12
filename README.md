# simprox

[![Build release](https://github.com/cars10/simprox/actions/workflows/build.yml/badge.svg)](https://github.com/cars10/simprox/actions/workflows/build.yml)
[![Docker build](https://img.shields.io/docker/cloud/automated/cars10/simprox.svg)](https://hub.docker.com/r/cars10/simprox)

| **Sim**ple **Prox**y Server

Simprox is a fast and simple local proxy server.

You can use it to bypass browser restrictions like CORS or invalid SSL certificates when working with external services in your browser.  
It forwards the complete original request to your proxy target and returns the response to your service.


## Download

### Binary

You can download the latest binary for linux, macos and windows from [github](https://github.com/cars10/simprox/releases).

### Docker

Download the [image](https://hub.docker.com/r/cars10/simprox):

```bash
docker pull cars10/simprox
```

## Usage

```
> simprox --help
simprox 0.1.0
Simple proxy server

USAGE:
    simprox [OPTIONS] --target_host <host:port>

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --listen_host <host:port>              Set the host for the proxy server itself [env: LISTEN_HOST=]  
                                               [default: 127.0.0.1:7000]
        --skip-ssl-verify <skip-ssl-verify>    Disable ssl certificate verification [env: SKIP_SSL_VERIFY=]
    -t, --target_host <host:port>              Sets the proxy target (required) [env: TARGET_HOST=]
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

You can also use environment variables for configuration:

```bash
LISTEN_HOST=0.0.0.0:7000 TARGET_HOST=https://localhost:9200 SKIP_SSL_VERIFY= simprox
```

### Docker

When using the docker image you have to make sure that the docker container can access the service that you want to proxy. Some examples:

a) When your service is accessible via `http://localhost:9200` you need to set `--net host`
```bash
docker run --rm \
           --name simprox \
           -p 7000:7000 \
           -e LISTEN_HOST=0.0.0.0:7000 \
           -e TARGET_HOST=http://localhost:9200 \
           --net host \
           cars10/simprox
```

b) Your service is accessible via `http://example.com`
```bash
docker run --rm \
           --name simprox \
           -p 7000:7000 \
           -e LISTEN_HOST=0.0.0.0:7000 \
           -e TARGET_HOST=http://example.com \
           cars10/simprox
```

c) Your service is running in another docker container named `test` on port `3000`
```bash
docker run --rm \
           --name simprox \
           -p 7000:7000 \
           -e LISTEN_HOST=0.0.0.0:7000 \
           -e TARGET_HOST=http://test:3000 \
           --link test \
           cars10/simprox
```


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


## License

MIT
