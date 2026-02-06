FROM rust:1.87-bookworm as builder

WORKDIR /app
COPY tickit-sync/ .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tickit-sync /usr/local/bin/

EXPOSE 3030
VOLUME /data

ENV RUST_LOG=tickit_sync=info

CMD ["tickit-sync", "serve", "--config", "/data/config.toml"]
