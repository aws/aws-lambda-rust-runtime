#!/bin/bash
set -euo pipefail

# Optional: set RIE_MAX_CONCURRENCY to enable LMI mode (emulates AWS_LAMBDA_MAX_CONCURRENCY)
RIE_MAX_CONCURRENCY=${RIE_MAX_CONCURRENCY:-}
# Optional: specify which handler to run (defaults to first handler)
HANDLER=${HANDLER:-basic-lambda}

echo "Building Docker image with RIE (handlers: $HANDLER)"
docker build -f Dockerfile.rie --build-arg HANDLERS_TO_BUILD="$HANDLER" -t rust-lambda-rie-test .

echo "Starting RIE container on port 9000 with handler: $HANDLER"
if [ -n "$RIE_MAX_CONCURRENCY" ]; then
    echo "Enabling LMI mode with AWS_LAMBDA_MAX_CONCURRENCY=$RIE_MAX_CONCURRENCY"
    docker run -p 9000:8080 -e AWS_LAMBDA_MAX_CONCURRENCY="$RIE_MAX_CONCURRENCY" rust-lambda-rie-test "$HANDLER" &
else
    docker run -p 9000:8080 rust-lambda-rie-test "$HANDLER" &
fi
CONTAINER_PID=$!

echo "Container started. Test with:"
echo "curl -XPOST 'http://localhost:9000/2015-03-31/functions/function/invocations' -d '{\"command\": \"test from RIE\"}' -H 'Content-Type: application/json'"
echo "or for a specific example check under examples/ for the expected payload format."

echo ""
echo "Press Ctrl+C to stop the container."

wait $CONTAINER_PID
