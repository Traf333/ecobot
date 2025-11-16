#!/bin/bash
# Quick setup script for remote server
# Run this on your remote server to prepare for deployment

set -e

echo "🚀 Setting up Ecobot deployment environment..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "📦 Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
    echo "✅ Docker installed. Please log out and back in for group changes to take effect."
else
    echo "✅ Docker is already installed"
fi

# Create deployment directory
echo "📁 Creating deployment directory..."
sudo mkdir -p /opt/ecobot

# Prompt for configuration
read -p "Enter your Telegram bot token: " TELOXIDE_TOKEN
read -p "Enter SurrealDB username [root]: " DB_USERNAME
DB_USERNAME=${DB_USERNAME:-root}
read -sp "Enter SurrealDB password: " DB_PASSWORD
echo ""

# Create .env file
echo "📝 Creating environment file..."
sudo tee /opt/ecobot/.env > /dev/null <<EOF
RUST_LOG=info
TELOXIDE_TOKEN=$TELOXIDE_TOKEN
URL=localhost
PORT=8000
DBNAME=ecobot
NAMESPACE=prod
USERNAME=$DB_USERNAME
PASSWORD=$DB_PASSWORD
EOF

# Set permissions (readable by root, needed for Docker)
sudo chmod 644 /opt/ecobot/.env

echo "✅ .env file created at /opt/ecobot/.env"
echo ""
echo "Next steps:"
echo "1. Generate SSH key for GitHub Actions (on your local machine):"
echo "   ssh-keygen -t ed25519 -C 'github-actions-ecobot' -f ~/.ssh/ecobot-deploy"
echo "2. Copy public key to this server:"
echo "   ssh-copy-id -i ~/.ssh/ecobot-deploy.pub $(whoami)@$(hostname -I | awk '{print $1}')"
echo "3. Add GitHub Secrets (DOCKER_USERNAME, DOCKER_PASSWORD, SERVER_HOST, SERVER_USER, SSH_PRIVATE_KEY)"
echo "4. Push to main branch to deploy!"
echo ""
echo "Useful commands:"
echo "  docker logs -f ecobot-app    # View bot logs"
echo "  docker ps                     # List running containers"
echo "  docker restart ecobot-app    # Restart the bot"
