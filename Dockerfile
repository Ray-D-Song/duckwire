# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY sql/ sql/
COPY build.rs ./

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/duckwire /usr/local/bin/duckwire

EXPOSE 5433

ENTRYPOINT ["duckwire"]
CMD ["--host", "0.0.0.0", "--port", "5433"]
