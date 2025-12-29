#!/bin/bash

# This bash script uses /etc/cron.d to store the CRON job
CRON_DIR="/etc/cron.d"
CRON_FILE="game-sales-scrapper"

# Command States
CREATE_STATE="create"
DELETE_STATE="delete"
CMD=""

VALID_INPUTS="Valid arguments: [$CREATE_STATE, $DELETE_STATE]."

while getopts 'c:h' opt; do
  case $opt in
    c) CMD=$OPTARG;;
    h)
      echo "Usage: set_cron.sh [-c arg] "
      echo "$VALID_INPUTS"
      exit
      ;;
    *)
      echo "Please use flag -h for assistance."
      exit
      ;;
  esac
done

cron_file_exists() {
  cron_exists="0"
  if [ -f "$CRON_DIR/$CRON_FILE" ]; then
    cron_exists="1"
  fi
  echo $cron_exists
}

if [[ $CMD == $CREATE_STATE ]]; then
  cron_exists=$(cron_file_exists)

  # Create CRON file for automation if it does not exist
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

  if [[ $cron_exists == "0" ]]; then
    echo "'$CRON_DIR/$CRON_FILE' was successfully created."
  elif [[ $cron_exists == "1" ]]; then
    echo "'$CRON_DIR/$CRON_FILE' was successfully updated."
  fi

elif [[ $CMD == $DELETE_STATE ]]; then
  if [[ $(cron_file_exists) == "1" ]]; then
    cd $CRON_DIR
    rm -rf $CRON_DIR/$CRON_FILE
    echo "'$CRON_DIR/$CRON_FILE' was successfully deleted."
  else
    echo "'$CRON_DIR/$CRON_FILE' does not exist."
  fi
else
  echo "Unrecognized CMD_STATE: '$CMD_STATE'. $VALID_INPUTS"
fi