#!/bin/bash

# Command line arguments
while getopts "p:" flag; do
  case ${flag} in
    p) SCRIPT_DIR=${OPTARG};;
    *) echo "Invalid flag: ${OPTARG}";;
  esac
done

# Go to script directory
cd $SCRIPT_DIR || exit

# Create log directory
LOG_DIR="$SCRIPT_DIR/../logs"
if [ ! -d $LOG_DIR ]; then
    mkdir -p $LOG_DIR
fi

# Check if target directory created
if [ ! -d "$SCRIPT_DIR/../target/release" ]; then
  cargo build --release
fi

# Run script to get game sales
DATE_TIME=$(date +"%Y_%m_%d_%H_%M")
LOG_NAME=$LOG_DIR/$DATE_TIME"_email.html"
cd ..
./target/release/gss-cli --send-email > $LOG_NAME
if [ -f $LOG_NAME ]; then
    echo "$LOG_NAME created"
fi