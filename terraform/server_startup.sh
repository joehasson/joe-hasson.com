#!/bin/bash
# Update system packages
sudo yum update -y

# Install Python and development tools
sudo yum install -y python3-pip python3-devel gcc git

# Install nginx
sudo amazon-linux-extras install nginx1 -y

# Start nginx
sudo systemctl start nginx
sudo systemctl enable nginx

# Create a directory for your app
sudo mkdir -p /var/www/fastapi
sudo chown ec2-user:ec2-user /var/www/fastapi

# Create a systemd service file for your FastAPI app
sudo tee /etc/systemd/system/fastapi.service << EOF
[Unit]
Description=FastAPI application
After=network.target

[Service]
User=ec2-user
Group=ec2-user
WorkingDirectory=/var/www/fastapi/src
ExecStart=gunicorn -w 4 -k uvicorn.workers.UvicornWorker app:app --bind 0.0.0.0:8000

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

# Test nginx configuration
sudo nginx -t

# Restart nginx
sudo systemctl restart nginx

# Start FastAPI service
sudo systemctl start fastapi
sudo systemctl enable fastapi
