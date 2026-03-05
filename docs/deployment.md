# Deployment Guide / 部署指南

## Systemd Service (Linux / WSL2)

### Service File

Location: `~/.config/systemd/user/ai-sound-notify.service`

```ini
[Unit]
Description=AI Sound Notify Server
After=network.target

[Service]
Type=simple
WorkingDirectory=/mnt/d/AI/ai-sound-notify/server
ExecStart=/home/km/.nvm/versions/node/v22.17.1/bin/node index.js
Restart=on-failure
RestartSec=5
Environment=PORT=9800

[Install]
WantedBy=default.target
```

> Adjust `WorkingDirectory` and `ExecStart` paths to match your environment.

### Setup Commands

```bash
# Create service directory
mkdir -p ~/.config/systemd/user

# Copy or create the service file
cp ai-sound-notify.service ~/.config/systemd/user/

# Reload systemd, enable and start
systemctl --user daemon-reload
systemctl --user enable ai-sound-notify
systemctl --user start ai-sound-notify

# Enable linger so service runs without active login session
loginctl enable-linger $USER
```

### Management Commands

```bash
# Check status
systemctl --user status ai-sound-notify

# Restart (after code changes)
systemctl --user restart ai-sound-notify

# Stop
systemctl --user stop ai-sound-notify

# Disable auto-start
systemctl --user disable ai-sound-notify

# View logs (live)
journalctl --user -u ai-sound-notify -f

# View recent logs
journalctl --user -u ai-sound-notify --since "1 hour ago"
```

## Reverse Proxy (Traefik)

If using Traefik as reverse proxy, add to your dynamic config:

### Router

```yaml
http:
  routers:
    ainotify:
      rule: Host(`ainotify.yourdomain.com`)
      entryPoints:
        - websecure
      service: ainotify-service
      tls:
        certResolver: letsencrypt
```

### Service

```yaml
http:
  services:
    ainotify-service:
      loadBalancer:
        servers:
          - url: "http://host.docker.internal:9800"
```

> **Important:** Traefik must support WebSocket upgrades. This works by default with Traefik v2+.

### Nginx Alternative

```nginx
server {
    listen 443 ssl;
    server_name ainotify.yourdomain.com;

    location / {
        proxy_pass http://localhost:9800;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Hook Configuration URLs

After deployment, update the hook configs to use your server URL:

| Scenario | URL |
|----------|-----|
| Local | `http://localhost:9800/notify` |
| LAN | `http://<server-ip>:9800/notify` |
| Reverse proxy (HTTPS) | `https://ainotify.yourdomain.com/notify` |

## Troubleshooting

### Service won't start
```bash
# Check logs for error details
journalctl --user -u ai-sound-notify -e

# Verify node path is correct
which node
```

### Port already in use
```bash
# Find process on port 9800
lsof -i :9800
# Or
ss -tlnp | grep 9800
```

### WebSocket not connecting through reverse proxy
- Ensure reverse proxy supports WebSocket upgrade headers
- Check browser console for connection errors
- Verify the proxy target URL matches the server port
