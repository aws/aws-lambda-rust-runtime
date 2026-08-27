#!/bin/bash
set -e

OUTPUT_DIR="${OUTPUT_DIR:-/tmp/var-task}"
HANDLERS_TO_BUILD="${HANDLERS_TO_BUILD:-}"

mkdir -p "$OUTPUT_DIR"

echo "Building handlers: ${HANDLERS_TO_BUILD}"


for handler in ${HANDLERS_TO_BUILD}; do
    dir="examples/$handler"
    if [ ! -f "$dir/Cargo.toml" ]; then
        echo "✗ $handler not found"
        continue
    fi

    echo "Building $handler..."
    if ! (cd "$dir" && cargo build --release); then
        continue
    fi

    if [ ! -f "$dir/target/release/$handler" ]; then
        echo "✗ $handler artifact not found"
        continue
    fi

    cp "$dir/target/release/$handler" "$OUTPUT_DIR/"
    echo "✓ $handler"
done

echo ""
ls -lh "$OUTPUT_DIR/" 2>/dev/null || echo "No binaries built"
exit 0
