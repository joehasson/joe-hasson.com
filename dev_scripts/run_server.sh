#!/bin/bash

# Work in project root
cd "$(dirname $0)/.."

# Render static content
python3 static_site/render.py

# Run nginx
docker run \
    -v "$(pwd)/nginx.conf:/etc/nginx/nginx.conf" \
    -v "$(pwd)/static_site/build:/static" \
    -w '/' \
    -p 8000:80 \
    nginx nginx -g 'daemon off;'
