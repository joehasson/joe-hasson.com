##!/bin/bash

set -e

# Store private key
echo "$PRIVATE_KEY" > private_key.pem
chmod 600 private_key.pem

# Deploy using SSH with fresh clone
ssh -o StrictHostKeyChecking=no -i private_key.pem "ec2-user@$EC2_IP" '\
  git clone https://github.com/${REPO}.git /var/www/fastapi &&\
  cd /var/www/fastapi &&\
  source venv/bin/activate &&\
  sudo systemctl restart fastapi
'
