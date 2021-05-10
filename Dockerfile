FROM rust:1.52-slim-buster as builder

RUN rustup update
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup toolchain install stable

RUN apt-get update && \
    apt-get install -y libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/myapp
COPY . .

RUN cargo build --release --locked


FROM debian:buster-slim
RUN apt-get update && \
    apt-get install -y libssl-dev && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/myapp/target/release/simprox /usr/local/bin/simprox
CMD ["simprox"]
