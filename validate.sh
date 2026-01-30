#!/bin/bash
# Comprehensive validation script for amp refactoring
# Run this before merging to main

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔍 Running comprehensive validation...${NC}"
echo ""

# Get repository root
REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"

echo -e "${YELLOW}1. Format and lint code...${NC}"
if ./scripts/fmt_fix_clippy.sh; then
    echo -e "${GREEN}✓ Format and lint passed${NC}"
else
    echo -e "${RED}✗ Format and lint failed${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}2. Build Android release...${NC}"
if cd android && cargo build --release; then
    echo -e "${GREEN}✓ Build passed${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}3. Run tests...${NC}"
if cargo test --release; then
    echo -e "${GREEN}✓ Tests passed${NC}"
else
    echo -e "${RED}✗ Tests failed${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}4. Generate documentation...${NC}"
if cargo doc --no-deps; then
    echo -e "${GREEN}✓ Documentation generated${NC}"
else
    echo -e "${RED}✗ Documentation generation failed${NC}"
    exit 1
fi

cd "$REPO_ROOT"

echo ""
echo -e "${YELLOW}5. Verify file structure...${NC}"

# Check old files are deleted
if [ -f android/src/ui/adresser.rs ]; then
    echo -e "${RED}✗ adresser.rs should be deleted${NC}"
    exit 1
fi
if [ -f android/src/ui/paneler.rs ]; then
    echo -e "${RED}✗ paneler.rs should be deleted${NC}"
    exit 1
fi
if [ -f android/src/ui/topbar.rs ]; then
    echo -e "${RED}✗ topbar.rs should be deleted${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Old files deleted${NC}"

# Check new files exist
if [ ! -f android/src/ui/addresses.rs ]; then
    echo -e "${RED}✗ addresses.rs missing${NC}"
    exit 1
fi
if [ ! -f android/src/ui/panels.rs ]; then
    echo -e "${RED}✗ panels.rs missing${NC}"
    exit 1
fi
if [ ! -f android/src/ui/top_bar.rs ]; then
    echo -e "${RED}✗ top_bar.rs missing${NC}"
    exit 1
fi
echo -e "${GREEN}✓ New files exist${NC}"

# Check scripts directory
if [ ! -f scripts/build.sh ] || [ ! -f scripts/serve.sh ] || [ ! -f scripts/adb-install.sh ]; then
    echo -e "${RED}✗ Scripts missing from scripts/ directory${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Scripts in place${NC}"

echo ""
echo -e "${YELLOW}6. Check for Swedish variable names...${NC}"
cd android/src

# These should NOT be found in code
if grep -r "gatunummer" . --include="*.rs" 2>/dev/null; then
    echo -e "${RED}✗ Found 'gatunummer' in code${NC}"
    exit 1
fi
if grep -r "postnummer" . --include="*.rs" 2>/dev/null; then
    echo -e "${RED}✗ Found 'postnummer' in code${NC}"
    exit 1
fi
echo -e "${GREEN}✓ No Swedish variable names found${NC}"

# Swedish UI text SHOULD be present
if ! grep -r "Adresser" . --include="*.rs" 2>/dev/null | grep -q "Adresser"; then
    echo -e "${RED}✗ Swedish UI text missing (should be preserved)${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Swedish UI text preserved${NC}"

cd "$REPO_ROOT"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All validation checks passed!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}📋 Summary:${NC}"
echo "  ✓ Code formatted and linted"
echo "  ✓ Android release build successful"
echo "  ✓ All tests passing"
echo "  ✓ Documentation generated"
echo "  ✓ File structure correct"
echo "  ✓ Swedish variables translated"
echo "  ✓ Swedish UI text preserved"
echo ""
echo -e "${GREEN}🚀 Ready to merge to main!${NC}"
echo ""
echo "To merge:"
echo "  git checkout main"
echo "  git merge refactor/comprehensive-2026-01"
echo "  git push origin main"
echo ""
