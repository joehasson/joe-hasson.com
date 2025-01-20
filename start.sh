#!/bin/bash
# Script used by the Dockerfile.migrations image to automatically retry
# running migrations at a regular interval to avoid timing issues
# with database startup.


MAX_RETRIES=10
RETRY_INTERVAL=20
count=0

until sqlx migrate run; do
    count=$((count + 1))
    if [ $count -eq $MAX_RETRIES ]; then
        echo "Max retries ($MAX_RETRIES) reached, giving up"
        exit 1
    fi
    echo "Migration failed, retry $count of $MAX_RETRIES in $RETRY_INTERVAL seconds..."
    sleep $RETRY_INTERVAL
done
