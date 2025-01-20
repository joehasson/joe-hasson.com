#!/bin/bash

# Work in project root
cd $(dirname $0)/..

ln -sf .hooks/sqlx-prepare-check.sh .git/hooks/pre-commit

