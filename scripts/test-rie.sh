#!/bin/bash
set -euo pipefail

# Optional: set RIE_MAX_CONCURRENCY to enable LMI mode (emulates AWS_LAMBDA_MAX_CONCURRENCY)
RIE_MAX_CONCURRENCY=${RIE_MAX_CONCURRENCY:-}

echo "Building Docker image with RIE"
docker build -f Dockerfile.rie -t rust-lambda-rie-test .

echo "Starting RIE container on port 9000..."
if [ -n "$RIE_MAX_CONCURRENCY" ]; then
    echo "Enabling LMI mode with AWS_LAMBDA_MAX_CONCURRENCY=$RIE_MAX_CONCURRENCY"
    docker run -p 9000:8080 -e AWS_LAMBDA_MAX_CONCURRENCY="$RIE_MAX_CONCURRENCY" rust-lambda-rie-test &
else
    docker run -p 9000:8080 rust-lambda-rie-test &
fi
CONTAINER_PID=$!

echo "Container started. Test with:"
echo "curl -XPOST 'http://localhost:9000/2015-03-31/functions/function/invocations' -d '{\"command\": \"test from RIE\"}' -H 'Content-Type: application/json'"
echo "or for a specific example check under examples/  for the expected payload format."

echo ""
echo "Press Ctrl+C to stop the container."

wait $CONTAINER_PID
