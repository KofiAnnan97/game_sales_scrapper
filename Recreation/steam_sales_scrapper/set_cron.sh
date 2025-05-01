#!/bin/bash

# This bash script uses /etc/cron.d to store the CRON job

CRON_DIR="/etc/cron.d"
CRON_FILE="steam-sales-scrapper"

# CRON file for automation if it does not exist
if [ ! -f "$CRON_DIR/$CRON_FILE" ]; then
    sudo touch "$CRON_DIR/$CRON_FILE"
fi

# Run every Tuesday and Thursday at 1 pm
SCHEDULE="0 13 * * 2,4"
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Configure as a system wide cron job
sudo echo "$SCHEDULE root $SCRIPT_DIR/send_email.sh" > "$CRON_DIR/$CRON_FILE"
sudo chmod 600 "$CRON_DIR/$CRON_FILE"