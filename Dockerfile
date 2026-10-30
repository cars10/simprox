FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/myapp
COPY . .

RUN cargo build --release --locked


FROM alpine:3
COPY --from=builder /usr/src/myapp/target/release/simprox /usr/local/bin/simprox
CMD ["simprox"]
