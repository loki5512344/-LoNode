# Dockerfile — LoNode
#
# Build: docker build -t lonode .
# Run:   docker run -p 2333:2333 -v $(pwd)/config.toml:/app/config.toml lonode

FROM rust:1.80-bookworm AS builder

WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libopus0 libsodium23 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/lonode /app/lonode
COPY --from=builder /build/config.example.toml /app/config.toml

EXPOSE 2333
VOLUME ["/app/plugins"]

ENTRYPOINT ["/app/lonode"]
