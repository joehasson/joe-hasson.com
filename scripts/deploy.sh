##!/bin/bash

################################################################################
# This script is run via SSH to set up our EC2 instance as a server
################################################################################

set -euo pipefail

sudo yum update -y
sudo amazon-linux-extras install nginx1 -y

# Check nginx configuration and then start
sudo nginx -t
sudo systemctl restart nginx
sudo systemctl enable nginx
