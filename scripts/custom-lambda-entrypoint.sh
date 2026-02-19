#!/bin/sh
# custom entrypoint script to allow selection of multiple rust binaries for dockerized tests
set -e

HANDLER=${1:-basic-lambda}

if [ -f "/var/task/$HANDLER" ]; then
  ln -sf "/var/task/$HANDLER" "${LAMBDA_RUNTIME_DIR}/bootstrap"
  exec /usr/local/bin/aws-lambda-rie "${LAMBDA_RUNTIME_DIR}/bootstrap"
else
  echo "Error: Handler '$HANDLER' not found in /var/task"
  echo "Available handlers:"
  ls -la /var/task
  exit 1
fi
