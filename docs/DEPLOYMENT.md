# Deployment Guide

This guide covers deploying LiteTerm-Web to production environments on various platforms.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build for Production](#build-for-production)
- [Configuration](#configuration)
- [Deployment Options](#deployment-options)
  - [Standalone Server](#standalone-server)
  - [Systemd Service](#systemd-service)
  - [Docker Deployment](#docker-deployment)
  - [Cloud Deployment](#cloud-deployment)
- [Reverse Proxy Setup](#reverse-proxy-setup)
- [Security Considerations](#security-considerations)
- [SSL/HTTPS Setup](#sslhttps-setup)
- [Monitoring](#monitoring)
- [Updates](#updates)

---

## Prerequisites

### System Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| **CPU** | 1 core | 2+ cores |
| **Memory** | 512 MB | 2 GB+ |
| **Storage** | 1 GB | 10 GB+ |
| **OS** | Linux/macOS/Windows | Linux (Ubuntu/CentOS) |
| **Network** | 100 Mbps | 1 Gbps |

### Dependencies

- **Rust** 1.75+ (for building)
- **Node.js** 18+ (for building frontend)
- **Operating System** with PTY support (Linux/macOS)

---

## Build for Production

### Option 1: Local Build

```bash
# Clone repository
git clone https://github.com/your-username/liteterm-web.git
cd liteterm-web

# Build release versions
make build

# Output:
# - Backend: server/target/release/liteterm-web
# - Frontend: client/dist/ (served by backend)
```

### Option 2: Cross-Compilation

```bash
# Build for different target
cd server

# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# macOS
cargo build --release --target x86_64-apple-darwin

# ARM64 (Raspberry Pi)
cargo build --release --target aarch64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

### Build Artifacts

After successful build:

```
server/
├── target/release/
│   └── liteterm-web          # Main server binary
└── config/
    └── config.toml           # Configuration file

client/
└── dist/                     # Static frontend files
    ├── index.html
    ├── assets/
    └── ...
```

---

## Configuration

### Create Production Config

Create `server/config/config.toml`:

```toml
# Server Configuration
[server]
bind_addr = "0.0.0.0"          # Listen on all interfaces
port = 3000                    # Port to listen on
root_dir = "/home/user/workspace"  # Root directory to serve
session_timeout_minutes = 60   # Session timeout
max_connections = 100          # Maximum concurrent connections

# Authentication
[auth]
# Generate with: argon2-cli generate salt
password_hash = "$argon2id$v=19$m=65536,t=3,p=4$YOUR_HASH_HERE"

# Feature Flags
[features]
enable_hdf5 = true             # Enable HDF5 file viewer
enable_watch = true            # Enable file watching
enable_debug = false           # Disable debug mode in production

# Logging
[logging]
level = "info"                 # log level: trace, debug, info, warn, error
file = "/var/log/liteterm.log" # Log file path
```

### Generate Password Hash

#### Method 1: Using argon2-cli

```bash
# Install argon2-cli
# Ubuntu/Debian:
sudo apt install argon2

# macOS:
brew install argon2

# Generate hash
echo "your-secure-password" | argon2 $(openssl rand -base64 32) -e -l 16 -k 65536 -p 4 -t 3

# Output:
# $argon2id$v=19$m=65536,t=3,p=4$...
```

#### Method 2: Using Online Tool

Visit: https://argon2.online/

Enter your password and use these settings:
- **Algorithm**: Argon2id
- **Memory**: 65536 KB
- **Iterations**: 3
- **Parallelism**: 4
- **Output**: 32 bytes

#### Method 3: Generate with Rust

```bash
cd server
cargo run --example generate_password

# This will prompt for password and print the hash
```

---

## Deployment Options

### Standalone Server

Simple deployment for single server instance.

#### Steps

1. **Create user account**

   ```bash
   sudo useradd -r -s /bin/false liteterm
   sudo mkdir -p /opt/liteterm
   sudo chown liteterm:liteterm /opt/liteterm
   ```

2. **Copy build artifacts**

   ```bash
   sudo cp server/target/release/liteterm-web /opt/liteterm/
   sudo mkdir -p /opt/liteterm/config
   sudo cp server/config/config.toml /opt/liteterm/config/
   sudo cp -r client/dist /opt/liteterm/static/
   sudo chown -R liteterm:liteterm /opt/liteterm
   ```

3. **Run server**

   ```bash
   sudo -u liteterm /opt/liteterm/liteterm-web --config /opt/liteterm/config/config.toml
   ```

4. **Access application**

   ```
   http://your-server-ip:3000
   ```

---

### Systemd Service

Recommended for production deployments on Linux.

#### Steps

1. **Create service file**

   ```bash
   sudo tee /etc/systemd/system/liteterm.service > /dev/null <<EOF
   [Unit]
   Description=LiteTerm-Web Server
   After=network.target

   [Service]
   Type=simple
   User=liteterm
   Group=liteterm
   WorkingDirectory=/opt/liteterm
   ExecStart=/opt/liteterm/liteterm-web --config /opt/liteterm/config/config.toml
   Restart=always
   RestartSec=5

   # Security settings
   NoNewPrivileges=yes
   PrivateTmp=yes
   ProtectSystem=strict
   ReadWritePaths=/var/log/liteterm
   ReadWritePaths=/tmp

   [Install]
   WantedBy=multi-user.target
   EOF
   ```

2. **Update config for systemd paths**

   ```toml
   [logging]
   level = "info"
   file = "/var/log/liteterm/liteterm.log"  # Ensure writable by liteterm user
   ```

3. **Reload systemd and start service**

   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable liteterm
   sudo systemctl start liteterm
   sudo systemctl status liteterm
   ```

4. **Check logs**

   ```bash
   sudo journalctl -u liteterm -f          # Follow logs
   sudo journalctl -u liteterm --since "1 hour ago"  # Recent logs
   ```

5. **Update service**

   ```bash
   # After updating binary
   sudo systemctl restart liteterm
   ```

---

### Docker Deployment

Containerized deployment using Docker.

#### Create Dockerfile

Create `Dockerfile` in project root:

```dockerfile
# Multi-stage build

# Build stage
FROM node:18-alpine AS builder

WORKDIR /app
COPY client/package*.json ./
RUN npm ci

COPY client/ .
RUN npm run build

# Runtime stage
FROM rust:1.75 AS server-builder

WORKDIR /build
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false liteterm

WORKDIR /opt/liteterm

# Copy server binary
COPY --from=server-builder /build/target/release/liteterm-web ./
COPY --from=builder /app/dist ./static
COPY server/config/config.toml ./config/

# Set permissions
RUN chown -R liteterm:liteterm /opt/liteterm
USER liteterm

EXPOSE 3000

CMD ["./liteterm-web", "--config", "config/config.toml"]
```

#### Build and Run

```bash
# Build image
docker build -t liteterm-web:latest .

# Run container
docker run -d \
  --name liteterm-web \
  -p 3000:3000 \
  -v /path/to/workspace:/workspace \
  -v /var/log/liteterm:/var/log/liteterm \
  liteterm-web:latest

# Check logs
docker logs liteterm-web

# Stop/remove
docker stop liteterm-web
docker rm liteterm-web
```

#### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  liteterm:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./config:/opt/liteterm/config:ro
      - /path/to/workspace:/workspace:ro
      - liteterm-logs:/var/log/liteterm
    restart: unless-stopped
    security_opt:
      - no-new-privileges:true
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m

volumes:
  liteterm-logs:
```

Run with:

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

---

### Cloud Deployment

#### AWS EC2

1. **Launch EC2 Instance**
   - AMI: Ubuntu Server 22.04 LTS
   - Instance Type: t3.small or larger
   - Security Group: Open port 3000 (or 80/443 with load balancer)

2. **Deploy**

   ```bash
   # SSH into instance
   ssh -i your-key.pem ubuntu@your-ec2-ip

   # Install dependencies
   sudo apt update
   sudo apt install -y git make

   # Clone and build
   git clone https://github.com/your-username/liteterm-web.git
   cd liteterm-web
   make install
   make build

   # Configure and run with systemd (see above)
   ```

3. **Elastic IP**
   - Assign Elastic IP to instance
   - Update DNS records if applicable

#### Google Cloud Platform (GCP)

1. **Create Compute Engine VM**
   - OS: Ubuntu 22.04 LTS
   - Machine Type: e2-micro or larger

2. **Deploy**

   ```bash
   # Same steps as AWS EC2
   # Create firewall rule to allow port 3000
   gcloud compute firewall-rules create allow-liteterm \
     --allow tcp:3000 \
     --source-ranges 0.0.0.0/0
   ```

#### DigitalOcean Droplet

1. **Create Droplet**
   - Image: Ubuntu 22.04
   - Size: Basic $6/mo (1GB RAM)

2. **Deploy**

   ```bash
   # Same deployment steps
   # Create Firewall in dashboard to allow port 3000
   ```

#### Railway

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and deploy
railway login
railway init
railway up
```

#### Render

```yaml
# render.yaml
services:
  - type: web
    name: liteterm-web
    env: rust
    buildCommand: make install && make build
    startCommand: ./server/target/release/liteterm-web --config config/config.toml
    envVars:
      - key: ROCKET_ADDRESS
        value: 0.0.0.0
      - key: ROCKET_PORT
        value: 10000
```

---

## Reverse Proxy Setup

### Nginx

Recommended for production to handle SSL, load balancing, etc.

#### Install Nginx

```bash
# Ubuntu/Debian
sudo apt install nginx

# CentOS/RHEL
sudo yum install nginx
```

#### Nginx Configuration

Create `/etc/nginx/sites-available/liteterm`:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # Redirect HTTP to HTTPS (optional)
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    # SSL Configuration (see SSL setup below)
    ssl_certificate /path/to/certificate.crt;
    ssl_certificate_key /path/to/private.key;

    # Proxy to LiteTerm
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 86400;
    }

    # Static files caching
    location ~* \.(js|css|png|jpg|jpeg|gif|svg|ico|woff|woff2)$ {
        proxy_pass http://127.0.0.1:3000;
        expires 30d;
        add_header Cache-Control "public, no-transform";
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
}
```

Enable site:

```bash
sudo ln -s /etc/nginx/sites-available/liteterm /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

---

## Security Considerations

### 1. Authentication

- **Strong passwords**: Use long, complex passwords (16+ characters)
- **Argon2 hashing**: Already implemented (resistant to brute force)
- **Session timeout**: Configure appropriate timeout (30-60 min)
- **Regular password rotation**: Change passwords periodically

```toml
[auth]
# Use strong, unique password
password_hash = "..."
# Shorter timeout for production
session_timeout_minutes = 30
```

### 2. Network Security

- **Firewall rules**: Only allow necessary ports
- **VPN/SSH tunnel**: Use for remote access
- **IP whitelisting**: Restrict access to specific IPs (if applicable)

```bash
# UFW (Ubuntu)
sudo ufw allow from 192.168.1.0/24 to any port 3000
sudo ufw enable
```

### 3. System Security

- **Run as non-root**: Use dedicated user account
- **Limit file access**: Restrict `root_dir` to necessary directories
- **Regular updates**: Keep system packages updated

```bash
# Automatic security updates
sudo apt install unattended-upgrades
sudo dpkg-reconfigure unattended-upgrades
```

### 4. Logging and Monitoring

```toml
[logging]
level = "info"  # Don't use debug in production
file = "/var/log/liteterm.log"
```

Monitor logs:

```bash
# Check for failed authentication attempts
grep "AuthFailed" /var/log/liteterm.log

# Monitor connection counts
grep "New connection" /var/log/liteterm.log | wc -l
```

---

## SSL/HTTPS Setup

### Using Let's Encrypt (Free SSL)

1. **Install Certbot**

   ```bash
   # Ubuntu/Debian
   sudo apt install certbot python3-certbot-nginx

   # CentOS/RHEL
   sudo yum install certbot python3-nginx
   ```

2. **Obtain certificate**

   ```bash
   sudo certbot --nginx -d your-domain.com -d www.your-domain.com
   ```

3. **Auto-renewal**

   ```bash
   # Test renewal
   sudo certbot renew --dry-run

   # Auto-renewal is set up automatically
   ```

### Using Custom Certificate

```bash
# Generate self-signed certificate (for testing)
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/liteterm.key \
  -out /etc/ssl/certs/liteterm.crt
```

Update Nginx config with certificate paths:

```nginx
server {
    listen 443 ssl;
    ssl_certificate /etc/ssl/certs/liteterm.crt;
    ssl_certificate_key /etc/ssl/private/liteterm.key;
    # ... rest of config
}
```

---

## Monitoring

### System Metrics

```bash
# Check server status
systemctl status liteterm

# Monitor resource usage
top -p $(pgrep liteterm)

# Disk usage
df -h

# Memory usage
free -h
```

### Application Logs

```bash
# View logs
sudo tail -f /var/log/liteterm.log

# Error messages
grep "ERROR" /var/log/liteterm.log

# Recent errors
journalctl -u liteterm --since "1 hour ago"
```

### Health Check Endpoint

Create simple health check:

```bash
#!/bin/bash
# check-health.sh

STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/health)

if [ $STATUS -eq 200 ]; then
  echo "OK"
  exit 0
else
  echo "FAILED (HTTP $STATUS)"
  exit 1
fi
```

Add to crontab:

```bash
# Check every 5 minutes
*/5 * * * * /path/to/check-health.sh || systemctl restart liteterm
```

---

## Updates

### Update Server Binary

```bash
# Stop service
sudo systemctl stop liteterm

# Backup current version
sudo cp /opt/liteterm/liteterm-web /opt/liteterm/liteterm-web.bak

# Pull latest changes
git pull origin main

# Rebuild
make build

# Deploy new binary
sudo cp server/target/release/liteterm-web /opt/liteterm/

# Start service
sudo systemctl start liteterm

# Check status
sudo systemctl status liteterm
```

### Rollback

```bash
# If issues occur
sudo systemctl stop liteterm
sudo cp /opt/liteterm/liteterm-web.bak /opt/liteterm/liteterm-web
sudo systemctl start liteterm
```

### Zero-Downtime Update (Advanced)

```bash
# Start new version on different port
sudo ./liteterm-web --config config-prod.toml --port 3001

# Update nginx upstream (see nginx config below)
# Test new version
# Switch nginx to new port
# Stop old version

# nginx upstream config
upstream liteterm_backend {
    server 127.0.0.1:3000;
    server 127.0.0.1:3001 backup;
}
```

---

## Troubleshooting

See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) for common issues and solutions.

---

## Performance Tuning

### System Settings

```bash
# Increase file descriptor limits
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# TCP settings for high load
echo "net.core.somaxconn = 1024" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Application Settings

```toml
[server]
max_connections = 200           # Adjust based on load
session_timeout_minutes = 30    # Shorter timeout
worker_threads = 4              # Match CPU cores
```

---

## Backup and Recovery

### Backup Configuration

```bash
#!/bin/bash
# backup-config.sh

BACKUP_DIR="/backup/liteterm/$(date +%Y%m%d)"
mkdir -p $BACKUP_DIR

# Backup config
cp -r /opt/liteterm/config $BACKUP_DIR/

# Backup logs
cp /var/log/liteterm.log $BACKUP_DIR/

# Backup custom files
tar -czf $BACKUP_DIR/custom-files.tar.gz /path/to/custom/dir

echo "Backup completed: $BACKUP_DIR"
```

### Restore from Backup

```bash
#!/bin/bash
# restore-config.sh

BACKUP_DIR=$1

if [ -z "$BACKUP_DIR" ]; then
  echo "Usage: $0 <backup-dir>"
  exit 1
fi

# Stop service
sudo systemctl stop liteterm

# Restore config
sudo cp -r $BACKUP_DIR/config /opt/liteterm/

# Restore logs
sudo cp $BACKUP_DIR/liteterm.log /var/log/

# Restart service
sudo systemctl start liteterm

echo "Restore completed"
```

---

## Conclusion

You have successfully deployed LiteTerm-Web! For support, see:
- [Troubleshooting Guide](./TROUBLESHOOTING.md)
- [GitHub Issues](https://github.com/your-repo/liteterm-web/issues)

Happy remote coding! 🚀
