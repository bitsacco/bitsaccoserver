#!/bin/bash

# Run database migrations

set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Run migrations with CLI feature enabled
cargo run --bin migration --features cli -- "$@"