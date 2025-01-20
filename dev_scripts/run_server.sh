#!/bin/bash

# Work in project root
cd $(dirname $0)/..

docker build . -t joe-hasson-personal-site
docker build . -f Dockerfile.migrations -t joe-hasson-personal-site-migrations
docker compose up
