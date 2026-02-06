# tickit-sync

Self-hosted sync server for [Tickit](https://github.com/ricardodantas/tickit) task manager.

## Features

- 🔒 **Self-hosted**: Run on your own server, keep your data private
- 🚀 **Single binary**: No dependencies, just download and run
- 💾 **SQLite storage**: Simple, reliable, no database server needed
- 🔑 **Token-based auth**: Simple API tokens for multiple devices
- 🔄 **Conflict resolution**: Last-write-wins with conflict reporting

## Quick Start

### 1. Initialize config

```bash
tickit-sync init
```

This creates `config.toml` with default settings.

### 2. Generate an API token

```bash
tickit-sync token --name "my-laptop"
```

Add the token to your `config.toml`:

```toml
[[tokens]]
name = "my-laptop"
token = "your-generated-token-here"
```

### 3. Start the server

```bash
tickit-sync serve
```

Server runs on `http://0.0.0.0:3030` by default.

### 4. Configure Tickit client

Edit `~/.config/tickit/config.toml`:

```toml
[sync]
enabled = true
server = "http://your-server:3030"
token = "your-generated-token-here"
interval_secs = 300  # sync every 5 minutes
```

## Configuration

### config.toml

```toml
[server]
bind = "0.0.0.0"  # Listen address
port = 3030       # Port

[database]
path = "tickit-sync.sqlite"  # SQLite database file

# Add multiple tokens for different devices
[[tokens]]
name = "macbook"
token = "token-1"

[[tokens]]
name = "desktop"
token = "token-2"
```

## Docker

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p tickit-sync

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tickit-sync /usr/local/bin/
EXPOSE 3030
VOLUME /data
CMD ["tickit-sync", "serve", "--config", "/data/config.toml"]
```

```bash
docker build -t tickit-sync .
docker run -d -p 3030:3030 -v ./data:/data tickit-sync
```

## API

### Health Check

```bash
curl http://localhost:3030/health
```

### Sync (POST /api/v1/sync)

```bash
curl -X POST http://localhost:3030/api/v1/sync \
  -H "Authorization: Bearer your-token" \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "uuid",
    "last_sync": "2024-01-01T00:00:00Z",
    "changes": []
  }'
```

## Security Notes

- Always use HTTPS in production (put behind nginx/caddy with TLS)
- Tokens are stored in plain text in config - protect the config file
- Each device should have its own token for revocation capability

## License

MIT
