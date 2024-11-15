##!/bin/bash

set -euo pipefail

# Update system packages
sudo yum update -y

# Install nginx
sudo amazon-linux-extras install nginx1 -y

# Install Python and development tools
sudo yum install -y python3-pip python3-devel gcc git

# Install python packages
pip3 install --no-cache-dir fastapi uvicorn gunicorn jinja2 webassets cssmin

# Create a directory for the app
APP_DIR="/var/www/fastapi"
sudo rm -rf "$APP_DIR"
sudo mkdir -p "$APP_DIR"
sudo chown ec2-user:ec2-user "$APP_DIR"
timeout 60 git clone https://github.com/${REPO}.git "$APP_DIR"
cd "$APP_DIR"

# Create a systemd service file for FastAPI
sudo tee /etc/systemd/system/fastapi.service << EOF
[Unit]
Description=FastAPI application
After=network.target

[Service]
User=ec2-user
Group=ec2-user
WorkingDirectory=/var/www/fastapi/src
ExecStart=/home/ec2-user/.local/bin/gunicorn -w 4 -k uvicorn.workers.UvicornWorker app:app --bind 0.0.0.0:8000

[Install]
WantedBy=multi-user.target
EOF

# Configure nginx as reverse proxy
sudo tee /etc/nginx/conf.d/fastapi.conf << EOF
server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
    }
}
EOF

# Remove default nginx configuration
sudo rm /etc/nginx/conf.d/default.conf

# Test nginx configuration and then start
sudo nginx -t
sudo systemctl restart nginx
sudo systemctl enable nginx

# Start FastAPI service
sudo systemctl restart fastapi
sudo systemctl enable fastapi
