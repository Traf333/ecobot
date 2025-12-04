# Quick Docker Reference

## Local Testing

```bash
# Build the image
docker build -t ecobot .

# Run with .env file (foreground)
docker run --rm --network host --env-file .env ecobot

# Run in detached mode (background)
docker run -d --name ecobot-app --network host --env-file .env ecobot
```

## Managing the Container

```bash
# View live logs (current container)
docker logs -f ecobot-app

# View persistent logs (survives deployments)
tail -f /opt/ecobot/logs/ecobot.log

# Stop
docker stop ecobot-app

# Start
docker start ecobot-app

# Restart
docker restart ecobot-app

# Remove
docker rm ecobot-app
```

## Production Deployment

Deployment is automatic via GitHub Actions when you push to `main`. The workflow:

1. Builds Docker image
2. Pushes to Docker Hub
3. SSHs to server
4. Pulls and runs the new image

See `DEPLOYMENT.md` for full setup instructions.
