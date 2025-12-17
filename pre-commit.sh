#!/bin/sh
#
# Pre-commit hook for Solidus
# Ensures code is properly formatted before allowing commits
#

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo "Running pre-commit checks..."

# Check if cargo is available
if ! command -v cargo >/dev/null 2>&1; then
    echo "${RED}Error: cargo is not installed or not in PATH${NC}"
    exit 1
fi

# Run cargo fmt --check to see if any files need formatting
echo "Checking Rust code formatting..."
if ! cargo fmt --all -- --check >/dev/null 2>&1; then
    echo "${RED}Error: Code is not properly formatted!${NC}"
    echo ""
    echo "The following files have formatting issues:"
    cargo fmt --all -- --check 2>&1
    echo ""
    echo "${YELLOW}To fix formatting issues, run:${NC}"
    echo "  ${GREEN}cargo fmt --all${NC}"
    echo ""
    echo "Then stage your changes and commit again."
    exit 1
fi

echo "${GREEN}✓ Code formatting is correct${NC}"
exit 0
