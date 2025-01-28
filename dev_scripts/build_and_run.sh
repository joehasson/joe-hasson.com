#!/bin/bash

set -xe

# Work in project root
cd $(dirname $0)/..

docker compose up db -d --wait
sqlx migrate run
cargo sqlx prepare

docker build . --target reverse-proxy -t reverse-proxy
docker build . --target blog-post-dispatcher -t blog-post-dispatcher
docker build . --target backend -t backend
docker build . --target migrations -f Dockerfile.migrations -t migrations

docker compose up
