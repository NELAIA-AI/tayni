#!/bin/bash
# TAYNI Version Bump Script
# Usage: ./scripts/bump-version.sh 0.25.0
#
# This script updates the version in all required files:
# - Cargo.toml
# - src/main.rs
# - README.md
# - tests/compiler_tests.rs

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.25.0"
    exit 1
fi

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format. Use semantic versioning: X.Y.Z (e.g., 0.25.0)"
    exit 1
fi

MAJOR_MINOR=$(echo "$VERSION" | sed 's/\.[0-9]*$//')

echo "Bumping TAYNI version to $VERSION"
echo ""

# 1. Update Cargo.toml
if [ -f "Cargo.toml" ]; then
    sed -i.bak "s/version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$VERSION\"/" Cargo.toml
    rm -f Cargo.toml.bak
    echo "[OK] Cargo.toml"
fi

# 2. Update src/main.rs
if [ -f "src/main.rs" ]; then
    sed -i.bak "s/const VERSION: \&str = \"[0-9]*\.[0-9]*\.[0-9]*\"/const VERSION: \&str = \"$VERSION\"/" src/main.rs
    rm -f src/main.rs.bak
    echo "[OK] src/main.rs"
fi

# 3. Update README.md
if [ -f "README.md" ]; then
    sed -i.bak "s/TAYNI Compiler v[0-9]*\.[0-9]*/TAYNI Compiler v$MAJOR_MINOR/" README.md
    sed -i.bak "s/tayni-c [0-9]*\.[0-9]*\.[0-9]*/tayni-c $VERSION/" README.md
    rm -f README.md.bak
    echo "[OK] README.md"
fi

# 4. Update tests/compiler_tests.rs
if [ -f "tests/compiler_tests.rs" ]; then
    sed -i.bak "s/contains(\"[0-9]*\.[0-9]*\")/contains(\"$MAJOR_MINOR\")/" tests/compiler_tests.rs
    rm -f tests/compiler_tests.rs.bak
    echo "[OK] tests/compiler_tests.rs"
fi

echo ""
echo "Version bumped to $VERSION"
echo ""
echo "Next steps:"
echo "  1. cargo build --release"
echo "  2. git add -A"
echo "  3. git commit -m 'Release v$VERSION'"
echo "  4. git tag -a v$VERSION -m 'TAYNI v$VERSION'"
echo "  5. git push origin main --tags"
