#!/bin/bash

# Exit on error
set -xe

# Ensure sqlx ready for offline check stage of build
cargo sqlx prepare -- --lib

# Check if there are any unstaged changes in .sqlx files
if [ -n "$(git diff -- '.sqlx')" ] || [ -n "$(git ls-files --others -- '.sqlx')" ]; then
    echo "Error: .sqlx files have unstaged changes after running 'sqlx prepare'"
    echo "Please stage these changes before committing:"
    git diff -- '*.sqlx'
    exit 1
fi

exit 0
