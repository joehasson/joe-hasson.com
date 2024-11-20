#!/bin/bash

# Work in project root
cd $(dirname $0)/..

docker build . -t site
docker run -p 8000:80 site
