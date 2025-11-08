#!/bin/sh

# This bash script uses /etc/cron.d to store the CRON job
CRON_DIR="/etc/cron.d"
CRON_FILE="game-sales-scrapper"

# CRON file for automation if it does not exist
if [ ! -f "$CRON_DIR/$CRON_FILE" ]; then
    touch "$CRON_DIR/$CRON_FILE"
fi

# Run every Tuesday and Thursday at 1 pm
SCHEDULE="0 13 * * 2,4"
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Configure as a system wide cron job
chmod +x "$SCRIPT_DIR/send_email.sh"
echo "$SCHEDULE root $SCRIPT_DIR/send_email.sh -p $SCRIPT_DIR >> $SCRIPT_DIR/../log/email_history.log 2>&1" > "$CRON_DIR/$CRON_FILE"
chmod 600 "$CRON_DIR/$CRON_FILE"