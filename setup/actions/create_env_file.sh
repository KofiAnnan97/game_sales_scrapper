#!/bin/bash

# Read environment variables
STEAM_API_KEY=$(echo "$STEAM_API_KEY")
PROJECT_PATH=$(echo "$PROJECT_PATH")
TEST_PATH="$(echo "$PROJECT_PATH")/src/tests"

# Create file contents
touch .env
ENV_STR="STEAM_API_KEY=\"$STEAM_API_KEY\"
PROJECT_PATH=\"$PROJECT_PATH\"
TEST_PATH=\"$TEST_PATH\"
RECIPIENT_EMAIL=\"\"
SMTP_HOST=\"\"
SMTP_PORT=0
SMTP_EMAIL=\"\"
SMTP_USERNAME=\"\"
SMTP_PWD=\"\""

# Write to .env file
echo "$ENV_STR" > .env