FROM rust:slim AS builder

WORKDIR /usr/src/business-app

RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config

COPY Cargo.toml .
COPY Cargo.lock .
COPY src src

RUN cargo build --release -j16

FROM debian:bullseye-slim

WORKDIR /usr/src/business-app

RUN apt-get update && apt-get install -y libpq-dev

COPY --from=builder /usr/src/business-app/target/release/business-app .
# COPY .env .

EXPOSE 8100

CMD ["./business-app"]