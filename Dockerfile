FROM rust:1.50-slim-buster as builder

RUN rustup update
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup toolchain install stable

RUN apt-get update && \
    apt-get install -y libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .
CMD ["simprox"]
