#!/bin/bash

set -e

# Work in project root
cd $(dirname $0)/..

docker compose up db -d --wait
sqlx migrate run
cargo sqlx prepare

docker build . -t web
docker build . -f Dockerfile.migrations -t migrations

docker compose up web migrations
