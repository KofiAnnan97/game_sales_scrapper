#!/bin/bash

# Read environment variables
STEAM_API_KEY=$(echo "$STEAM_API_KEY")
PROJECT_PATH=$(echo "$PROJECT_PATH")

# Create file contents
touch .env
ENV_STR="STEAM_API_KEY=\"$STEAM_API_KEY\"
PROJECT_PATH=\"$PROJECT_PATH\""

# Write to .env file
echo "$ENV_STR" > .env