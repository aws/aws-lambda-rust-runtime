#!/bin/bash
set -e

OUTPUT_DIR="${OUTPUT_DIR:-/tmp/var-task}"
EXAMPLES="${EXAMPLES:-}"

mkdir -p "$OUTPUT_DIR"

echo "Building examples: $(EXAMPLES)"


for example in ${EXAMPLES}; do
    dir="examples/$example"
    [ ! -f "$dir/Cargo.toml" ] && echo "✗ $example not found" && continue
    
    echo "Building $example..."
    (cd "$dir" && cargo build --release) || continue
    
    [ -f "$dir/target/release/$example" ] && cp "$dir/target/release/$example" "$OUTPUT_DIR/" && echo "✓ $example"
done

echo ""
ls -lh "$OUTPUT_DIR/" 2>/dev/null || echo "No binaries built"
