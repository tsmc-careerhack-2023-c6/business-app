FROM rust:slim AS builder

WORKDIR /usr/src/business-app

RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config
RUN apt-get install -y wrk

COPY Cargo.toml .
COPY Cargo.lock .
COPY src src

RUN rustup component add llvm-tools-preview
RUN cargo install cargo-pgo

EXPOSE 8100

RUN RUSTFLAGS="-C target-cpu=native -C target-feature=+sse3,+avx2,+fma" cargo pgo instrument build -- --release -j16
RUN cargo pgo run &
RUN sleep 300
RUN wrk -t2 -c64 -d300s --latency http://localhost:8100/api/health
RUN killall business-app

RUN RUSTFLAGS="-C target-cpu=native -C target-feature=+sse3,+avx2,+fma" cargo pgo optimize

FROM debian:bullseye-slim

WORKDIR /usr/src/business-app

RUN apt-get update && apt-get install -y libpq-dev

COPY --from=builder /usr/src/business-app/target/release/business-app .
# COPY .env .

EXPOSE 8100

CMD ["./business-app"]