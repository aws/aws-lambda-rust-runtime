#!/bin/bash
set -euo pipefail

EXAMPLE=${1:-basic-lambda}
# Optional: set RIE_MAX_CONCURRENCY to enable LMI mode (emulates AWS_LAMBDA_MAX_CONCURRENCY)
RIE_MAX_CONCURRENCY=${RIE_MAX_CONCURRENCY:-}

echo "Building Docker image with RIE for example: $EXAMPLE..."
docker build -f Dockerfile.rie --build-arg EXAMPLE=$EXAMPLE -t rust-lambda-rie-test .

echo "Starting RIE container on port 9000..."
if [ -n "$RIE_MAX_CONCURRENCY" ]; then
    echo "Enabling LMI mode with AWS_LAMBDA_MAX_CONCURRENCY=$RIE_MAX_CONCURRENCY"
    docker run -p 9000:8080 -e AWS_LAMBDA_MAX_CONCURRENCY="$RIE_MAX_CONCURRENCY" rust-lambda-rie-test &
else
    docker run -p 9000:8080 rust-lambda-rie-test &
fi
CONTAINER_PID=$!

echo "Container started. Test with:"
if [ "$EXAMPLE" = "basic-lambda" ] || [ "$EXAMPLE" = "basic-lambda-concurrent" ]; then
    echo "curl -XPOST 'http://localhost:9000/2015-03-31/functions/function/invocations' -d '{\"command\": \"test from RIE\"}' -H 'Content-Type: application/json'"
else
    echo "For example '$EXAMPLE', check examples/$EXAMPLE/src/main.rs for the expected payload format."
fi
echo ""
echo "Press Ctrl+C to stop the container."

wait $CONTAINER_PID