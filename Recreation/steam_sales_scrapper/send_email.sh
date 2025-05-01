#!/bin/bash

# Go to script directory
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

# Create log directory
LOG_DIR="$SCRIPT_DIR/log"
if [ ! -d $LOG_DIR ]; then
    mkdir -p $LOG_DIR
fi

# Run script to get game sales
DATE_TIME=$(date +"%Y_%m_%d_%H_%M")
cargo run -- --send-email > $LOG_DIR/$DATE_TIME"_email.log"