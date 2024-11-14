##!/bin/bash

set -e

# Store private key
echo "$PRIVATE_KEY" > private_key.pem
chmod 600 private_key.pem

# Deploy using SSH with fresh clone
echo "ec2-user@$EC2_IP"
ssh -o StrictHostKeyChecking=no -i private_key.pem "ec2-user@$EC2_IP" "\
  sudo rm -rf /var/www/fastapi &&\
  sudo git clone https://github.com/${REPO}.git /var/www/fastapi &&\
  cd /var/www/fastapi &&\
  pip3 install fastapi uvicorn gunicorn jinja2 webassets cssmin &&\
  sudo systemctl restart fastapi
"
