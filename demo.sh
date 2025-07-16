#!/bin/bash

# Example script showing how to use the New Relic API Keys CLI
# Make sure to set your API key first:
# export NEW_RELIC_API_KEY="NRAK-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

# Build the CLI
echo "Building the CLI..."
cargo build --release

# Binary location
CLI="./target/release/newrelic_apikeys_cli"

# Check if API key is set
if [ -z "$NEW_RELIC_API_KEY" ]; then
    echo "Please set your NEW_RELIC_API_KEY environment variable"
    echo "export NEW_RELIC_API_KEY=\"NRAK-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\""
    exit 1
fi

# Replace with your actual account ID
ACCOUNT_ID="123456"

echo "==== New Relic API Keys CLI Demo ===="
echo

# Query existing API keys
echo "1. Querying existing API keys..."
$CLI query --account-id $ACCOUNT_ID
echo

# Create a new API key
echo "2. Creating a new API key..."
$CLI create --account-id $ACCOUNT_ID --key-type USER --name "Demo Key" --notes "Created by CLI demo"
echo

# Update an API key (you'll need to replace KEY_ID with actual ID from create response)
# echo "3. Updating an API key..."
# $CLI update --key-id "your-key-id-here" --name "Updated Demo Key"
# echo

# Delete an API key (you'll need to replace KEY_ID with actual ID)
# echo "4. Deleting an API key..."
# $CLI delete --key-id "your-key-id-here"
# echo

echo "Demo complete!"
