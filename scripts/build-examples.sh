#!/bin/bash
set -e

OUTPUT_DIR="${OUTPUT_DIR:-/tmp/var-task}"
HANDLERS_TO_BUILD="${HANDLERS_TO_BUILD:-}"

mkdir -p "$OUTPUT_DIR"

echo "Building handlers: ${HANDLERS_TO_BUILD}"


for handler in ${HANDLERS_TO_BUILD}; do
    dir="examples/$handler"
    [ ! -f "$dir/Cargo.toml" ] && echo "✗ $handler not found" && continue
    
    echo "Building $handler..."
    (cd "$dir" && cargo build --release) || continue
    
    [ -f "$dir/target/release/$handler" ] && cp "$dir/target/release/$handler" "$OUTPUT_DIR/" && echo "✓ $handler"
done

echo ""
ls -lh "$OUTPUT_DIR/" 2>/dev/null || echo "No binaries built"
