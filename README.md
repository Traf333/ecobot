# Ecobot

Telegram bot for eco-related information, built with Rust using [Teloxide](https://github.com/teloxide/teloxide) and [SurrealDB](https://surrealdb.com/).

## Features

- Telegram bot interface
- SurrealDB backend for data storage
- Docker containerization
- Automated CI/CD deployment via GitHub Actions

## Quick Start

### Prerequisites

- Rust 1.75+ (for local development)
- Docker (for containerized deployment)
- SurrealDB instance
- Telegram Bot Token from [@BotFather](https://t.me/botfather)

### Local Development

1. Clone the repository
2. Copy `.env.example` to `.env` and configure:
   ```bash
   cp .env.example .env
   nano .env  # Add your TELOXIDE_TOKEN and SurrealDB credentials
   ```
3. Run locally:
   ```bash
   cargo run
   ```

### Docker Deployment

See **[DEPLOYMENT.md](DEPLOYMENT.md)** for complete production deployment instructions.

For quick Docker commands, see **[README_DOCKER.md](README_DOCKER.md)**.

## Environment Variables

- `TELOXIDE_TOKEN` - Your Telegram bot token
- `URL` - SurrealDB host (e.g., `localhost`)
- `PORT` - SurrealDB port (default: `8000`)
- `DBNAME` - Database name
- `NAMESPACE` - Database namespace
- `USERNAME` - SurrealDB username
- `PASSWORD` - SurrealDB password
- `RUST_LOG` - Log level (default: `info`)

## Project Structure

```
src/
├── main.rs           # Entry point
├── db/               # Database operations
│   ├── mod.rs        # Connection setup
│   ├── user.rs       # User operations
│   └── bin_location.rs # Location operations
├── handlers/         # Bot command handlers
└── route.rs          # Message routing
```

## Deployment

Automated Docker deployment via GitHub Actions. Push to `main` branch to automatically build and deploy to your server.

- 📖 [DEPLOYMENT.md](DEPLOYMENT.md) - Complete deployment setup guide
- 🐳 [README_DOCKER.md](README_DOCKER.md) - Docker command reference
