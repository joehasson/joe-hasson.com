#!/bin/bash

set -e

# Work in project root
cd $(dirname $0)/..

docker build . -t web
docker build . -f Dockerfile.migrations -t migrations
docker compose up
