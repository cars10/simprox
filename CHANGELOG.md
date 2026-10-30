# Changelog

## 0.5.0

* update all dependencies, rust edition 2024
* replace warp/openssl with hyper and rustls (no system SSL library needed anymore)
* reuse connections to the proxy target
* send the target's `Host` header instead of the proxy's
* add `--skip-cors` to answer CORS preflight requests in simprox
* fix the macOS release binary name (`simprox-macos-arm64`)

## 0.4.0

* update dependencies

Breaking change: `--listen-host` was renamed to `--listen`

## 0.3.0

* fix clippy warnings
* update to rust `1.52`

## 0.2.0

* Improve `Dockerfile` to reduce image size

## 0.1.0

Intial release
