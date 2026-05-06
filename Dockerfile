# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app

# 1. Cache dependency compilation by building a dummy project first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo '' > src/lib.rs
COPY build.rs ./
COPY sql/ sql/
RUN cargo build --release && rm -rf src

# 2. Copy actual source code; only this layer and beyond will recompile
COPY src/ src/

# Touch files to invalidate cargo cache for the real source
RUN touch src/main.rs src/lib.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/duckwire /usr/local/bin/duckwire

EXPOSE 5433

ENTRYPOINT ["duckwire"]
CMD ["--host", "0.0.0.0", "--port", "5433"]