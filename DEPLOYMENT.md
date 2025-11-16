# Deployment Guide

This guide will help you deploy the Ecobot Telegram bot to your remote server using Docker and automated CI/CD with SSH.

## Prerequisites

- Docker installed on your remote server
- Docker Hub account (or other container registry)
- GitHub repository with your code
- SSH access to your remote server

## Setup Steps

### 1. Prepare Your Remote Server

SSH into your remote server and install Docker:

```bash
# Install Docker (Ubuntu/Debian)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Log out and back in for group changes to take effect
```

### 2. Verify SurrealDB Connection

Ensure your existing SurrealDB instance is running and accessible. Note down:

- SurrealDB host/URL (e.g., `localhost` or IP address)
- Port (default: `8000`)
- Username and password
- Database name and namespace

Test the connection:

```bash
# Replace with your actual SurrealDB details
curl http://localhost:8000/health
```

### 3. Set Up Environment on Server

SSH into your server and create the environment file:

```bash
# Create directory for deployment files
sudo mkdir -p /opt/ecobot

# Create .env file with your bot configuration
sudo tee /opt/ecobot/.env > /dev/null <<EOF
RUST_LOG=info
TELOXIDE_TOKEN=your_telegram_bot_token
URL=localhost
PORT=8000
DBNAME=ecobot
NAMESPACE=prod
USERNAME=your_db_username
PASSWORD=your_db_password
EOF

# Set permissions
sudo chmod 600 /opt/ecobot/.env
```

### 4. Generate SSH Key for GitHub Actions

Create an SSH key for GitHub Actions to use:

```bash
# On your local machine
ssh-keygen -t ed25519 -C "github-actions-ecobot" -f ~/.ssh/ecobot-deploy

# Copy the public key to your server
ssh-copy-id -i ~/.ssh/ecobot-deploy.pub user@your-server

# Display the private key to copy
cat ~/.ssh/ecobot-deploy
# Copy the entire output (including BEGIN and END lines)
```

### 5. Configure GitHub Secrets

Go to your GitHub repository → Settings → Secrets and variables → Actions → New repository secret

Add the following secrets:

| Secret Name       | Description                              | Example                                 |
| ----------------- | ---------------------------------------- | --------------------------------------- |
| `DOCKER_USERNAME` | Your Docker Hub username                 | `johndoe`                               |
| `DOCKER_PASSWORD` | Your Docker Hub password or access token | `dckr_pat_xxxxx`                        |
| `SERVER_HOST`     | Your server IP or hostname               | `192.168.1.100` or `server.example.com` |
| `SERVER_USER`     | SSH username for your server             | `ubuntu`                                |
| `SSH_PRIVATE_KEY` | SSH private key from step 4              | Contents of `~/.ssh/ecobot-deploy`      |
| `SERVER_PORT`     | SSH port (optional, defaults to 22)      | `22`                                    |

### 6. Test the Setup Locally (Optional)

Before pushing to main, test locally (ensure your SurrealDB is accessible):

```bash
# Create .env file from example
cp .env.example .env

# Edit .env with your actual SurrealDB connection details
nano .env

# Build the Docker image
docker build -t ecobot .

# Run with env file
docker run --rm --network host --env-file .env ecobot
```

**Note:** The `URL` in `.env` should point to your existing SurrealDB instance (e.g., `localhost` or the IP where it's running).

### 7. Deploy

Simply push your code to the `main` branch:

```bash
git add .
git commit -m "Add Docker and CI/CD configuration"
git push origin main
```

**How it works:**

1. GitHub Actions builds the Docker image
2. Pushes it to Docker Hub
3. SSHs into your server
4. Pulls the latest image
5. Stops old container and starts new one

### 8. Monitor the Deployment

- **GitHub Actions**: Check the Actions tab in your GitHub repository
- **Bot logs**: SSH into your server and run `docker logs -f ecobot-app`

## Troubleshooting

### Container won't start

```bash
# Check container status
docker ps -a

# View logs
docker logs ecobot-app
```

### SurrealDB connection issues

```bash
# Test if SurrealDB is accessible from the bot container
docker exec -it ecobot-app curl http://${DB_URL}:${DB_PORT}/health

# Or test from the host
curl http://localhost:8000/health
```

### Update environment variables

Edit `/opt/ecobot/.env` on your server, then restart the bot:

```bash
sudo nano /opt/ecobot/.env
docker restart ecobot-app
```

## Maintenance

### Update the bot

Push changes to main branch - automatic deployment will handle it.

### Common Docker Commands

```bash
# View logs (live tail)
docker logs -f ecobot-app

# View logs (last 100 lines)
docker logs --tail 100 ecobot-app

# View all running containers
docker ps

# Stop the bot
docker stop ecobot-app

# Start the bot
docker start ecobot-app

# Restart the bot
docker restart ecobot-app

# Check bot status
docker ps -a | grep ecobot-app

# Remove stopped container
docker rm ecobot-app
```

### Backup Database

Refer to your SurrealDB instance documentation for backup procedures. The bot doesn't manage the database, so backups should be handled by your database administrator.

## Security Best Practices

1. **Use strong passwords** for SurrealDB
2. **Keep secrets secure** - never commit them to git
3. **Use SSH keys** instead of passwords
4. **Regularly update** your Docker images
5. **Monitor logs** for suspicious activity
6. **Firewall rules** - only open necessary ports
7. **Regular backups** of your database

## Architecture

```
GitHub Push (main)
    ↓
GitHub Actions
    ↓ (build & push)
Docker Hub
    ↓ (SSH deploy)
Remote Server
    ├── Pull latest image
    ├── Stop old container
    └── Start new container
        └── ecobot-app (Telegram Bot) ──→ Existing SurrealDB Instance
```

**Deployment Flow:**

1. You push to main branch
2. GitHub Actions builds Docker image
3. Image pushed to Docker Hub
4. GitHub Actions SSHs to server
5. Server pulls latest image and restarts container
