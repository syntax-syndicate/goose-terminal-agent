#!/bin/bash

echo "Testing OpenRouter OAuth flow..."
echo "Current directory: $(pwd)"

# Clean up any existing config
rm -f ~/.config/goose/config.json

# Run the auth command
cd /Users/micn/Development/goose
cargo run --bin goose -- auth openrouter
