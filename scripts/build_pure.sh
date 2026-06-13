#!/bin/bash
# Build NELAIA programs without libc (Linux x86_64)
# Usage: ./build_pure.sh program.ll

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SYSCALL_LAYER="$SCRIPT_DIR/../src/syscalls/linux_x86_64.ll"

if [ -z "$1" ]; then
    echo "Usage: $0 <program.ll> [output_name]"
    exit 1
fi

INPUT="$1"
OUTPUT="${2:-$(basename "$INPUT" .ll)}"

echo "[NELAIA] Building $INPUT -> $OUTPUT (pure, no libc)"

# Combine syscall layer with program
cat "$SYSCALL_LAYER" "$INPUT" > "/tmp/nelaia_combined.ll"

# Compile without libc
clang -nostdlib -static -O2 -o "$OUTPUT" "/tmp/nelaia_combined.ll"

# Show result
echo "[NELAIA] Build complete:"
file "$OUTPUT"
ls -la "$OUTPUT"

echo ""
echo "[NELAIA] Verifying no libc dependency:"
ldd "$OUTPUT" 2>&1 || echo "  (statically linked - no dynamic dependencies)"
