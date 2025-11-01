#!/usr/bin/env bash
# Automated test script for Phase 7.5 - sync-version.sh
# Tests version synchronization across documentation files

set -e  # Exit on error

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Colour

# Test directory
TEST_DIR="/tmp/hypr-keybind-manager-version-test-$$"
SCRIPT_PATH="scripts/sync-version.sh"

# Counters
PASSED=0
FAILED=0

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_test() {
    echo -e "${YELLOW}▶ $1${NC}"
}

print_pass() {
    echo -e "${GREEN}✅ PASS: $1${NC}"
    PASSED=$((PASSED + 1))
}

print_fail() {
    echo -e "${RED}❌ FAIL: $1${NC}"
    FAILED=$((FAILED + 1))
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Cleanup function
cleanup() {
    print_info "Cleaning up..."
    if [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
        print_info "Test directory removed"
    fi
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# Check prerequisites
print_header "Prerequisites Check"

if [ ! -f "$SCRIPT_PATH" ]; then
    print_fail "Script not found: $SCRIPT_PATH"
    exit 1
fi
print_pass "sync-version.sh script exists"

# Create test directory
mkdir -p "$TEST_DIR/docs"
print_pass "Test directory created: $TEST_DIR"

# Create test Cargo.toml
cat > "$TEST_DIR/Cargo.toml" <<'EOF'
[package]
name = "hypr-keybind-manager"
version = "1.2.3"
edition = "2021"
EOF
print_pass "Test Cargo.toml created"

# Create test README.md
cat > "$TEST_DIR/README.md" <<'EOF'
# Hyprland Keybinding Manager

![Version](https://img.shields.io/badge/version-1.0.0-blue)

Some other content here.
More content that should not change.
EOF
print_pass "Test README.md created"

# Create test SECURITY.md
cat > "$TEST_DIR/SECURITY.md" <<'EOF'
# Security Policy

**Version**: 1.0.0

Some security content here.
EOF
print_pass "Test SECURITY.md created"

# Create test ARCHITECTURE.md
cat > "$TEST_DIR/docs/ARCHITECTURE.md" <<'EOF'
# Architecture

**Version**: 1.0.0

Some architecture content.
EOF
print_pass "Test docs/ARCHITECTURE.md created"

# Create test DESIGN_DECISIONS.md
cat > "$TEST_DIR/docs/DESIGN_DECISIONS.md" <<'EOF'
# Design Decisions

**Version**: 1.0.0

Some design decisions.
EOF
print_pass "Test docs/DESIGN_DECISIONS.md created"

# Create test PKGBUILD
cat > "$TEST_DIR/PKGBUILD" <<'EOF'
# Maintainer: Test User <test@example.com>

pkgname=test-package
pkgver=1.0.0
pkgrel=1
EOF
print_pass "Test PKGBUILD created."

#==============================================================================
# TEST 1: Basic Version Synchronization
#==============================================================================
print_header "Test 1: Basic Version Synchronization"

print_test "Running sync-version.sh in test directory..."
cd "$TEST_DIR"
bash "$OLDPWD/$SCRIPT_PATH" > /tmp/sync-version-output.log 2>&1
cd "$OLDPWD"

if [ $? -eq 0 ]; then
    print_pass "Script executed successfully"
else
    print_fail "Script execution failed"
    cat /tmp/sync-version-output.log
fi

#==============================================================================
# TEST 2: README.md Badge Update
#==============================================================================
print_header "Test 2: README.md Badge Update"

print_test "Checking README.md badge version..."
if grep -q "badge/version-1.2.3-blue" "$TEST_DIR/README.md"; then
    print_pass "README.md badge updated to 1.2.3"
else
    print_fail "README.md badge NOT updated"
    grep "badge/version" "$TEST_DIR/README.md" || echo "No badge found"
fi

# Verify unchanged content
if grep -q "Some other content here" "$TEST_DIR/README.md"; then
    print_pass "README.md other content unchanged"
else
    print_fail "README.md content was corrupted"
fi

#==============================================================================
# TEST 3: SECURITY.md Version Update
#==============================================================================
print_header "Test 3: SECURITY.md Version Update"

print_test "Checking SECURITY.md version..."
if grep -q "^\*\*Version\*\*: 1.2.3" "$TEST_DIR/SECURITY.md"; then
    print_pass "SECURITY.md version updated to 1.2.3"
else
    print_fail "SECURITY.md version NOT updated"
    grep "Version" "$TEST_DIR/SECURITY.md" || echo "No version found"
fi

# Verify unchanged content
if grep -q "Some security content here" "$TEST_DIR/SECURITY.md"; then
    print_pass "SECURITY.md other content unchanged"
else
    print_fail "SECURITY.md content was corrupted"
fi

#==============================================================================
# TEST 4: ARCHITECTURE.md Version Update
#==============================================================================
print_header "Test 4: ARCHITECTURE.md Version Update"

print_test "Checking docs/ARCHITECTURE.md version..."
if grep -q "^\*\*Version\*\*: 1.2.3" "$TEST_DIR/docs/ARCHITECTURE.md"; then
    print_pass "ARCHITECTURE.md version updated to 1.2.3"
else
    print_fail "ARCHITECTURE.md version NOT updated"
    grep "Version" "$TEST_DIR/docs/ARCHITECTURE.md" || echo "No version found"
fi

# Verify unchanged content
if grep -q "Some architecture content" "$TEST_DIR/docs/ARCHITECTURE.md"; then
    print_pass "ARCHITECTURE.md other content unchanged"
else
    print_fail "ARCHITECTURE.md content was corrupted"
fi

#==============================================================================
# TEST 5: DESIGN_DECISIONS.md Version Update
#==============================================================================
print_header "Test 5: DESIGN_DECISIONS.md Version Update"

print_test "Checking docs/DESIGN_DECISIONS.md version..."
if grep -q "^\*\*Version\*\*: 1.2.3" "$TEST_DIR/docs/DESIGN_DECISIONS.md"; then
    print_pass "DESIGN_DECISIONS.md version updated to 1.2.3"
else
    print_fail "DESIGN_DECISIONS.md version NOT updated"
    grep "Version" "$TEST_DIR/docs/DESIGN_DECISIONS.md" || echo "No version found"
fi

# Verify unchanged content
if grep -q "Some design decisions" "$TEST_DIR/docs/DESIGN_DECISIONS.md"; then
    print_pass "DESIGN_DECISIONS.md other content unchanged"
else
    print_fail "DESIGN_DECISIONS.md content was corrupted"
fi

#==============================================================================
# TEST 6: PKGBUILD Version Update
#==============================================================================
print_header "Test 6: PKGBUILD Version Update"

print_test "Checking PKGBUILD version..."
if grep -q "^pkgver=1.2.3" "$TEST_DIR/PKGBUILD"; then
    print_pass "PKGBUILD version updated to 1.2.3"
else
    print_fail "PKGBUILD version NOT updated"
    grep "pkgver" "$TEST_DIR/PKGBUILD" || echo "No pkgver found"
fi

# Verify pkgrel unchanged
if grep -q "^pkgrel=1" "$TEST_DIR/PKGBUILD"; then
    print_pass "PKGBUILD pkgrel unchanged"
else
    print_fail "PKGBUILD pkgrel was modified"
fi

#==============================================================================
# TEST 7: All Versions Consistent
#==============================================================================
print_header "Test 7: Version Consistency Check"

print_test "Verifying all files have same version (1.2.3)..."
README_VER=$(grep -oP 'badge/version-\K[0-9]+\.[0-9]+\.[0-9]+' "$TEST_DIR/README.md" | head -1)
SECURITY_VER=$(grep -oP '^\*\*Version\*\*: \K[0-9]+\.[0-9]+\.[0-9]+' "$TEST_DIR/SECURITY.md" | head -1)
ARCH_VER=$(grep -oP '^\*\*Version\*\*: \K[0-9]+\.[0-9]+\.[0-9]+' "$TEST_DIR/docs/ARCHITECTURE.md" | head -1)
DESIGN_VER=$(grep -oP '^\*\*Version\*\*: \K[0-9]+\.[0-9]+\.[0-9]+' "$TEST_DIR/docs/DESIGN_DECISIONS.md" | head -1)

PKGBUILD_VER=$(grep -oP '^pkgver=\K[0-9]+\.[0-9]+\.[0-9]+' "$TEST_DIR/PKGBUILD" | head -1)

if [ "$README_VER" = "1.2.3" ] && [ "$SECURITY_VER" = "1.2.3" ] && [ "$ARCH_VER" = "1.2.3" ] && [ "$DESIGN_VER" = "1.2.3" ] && [ "$PKGBUILD_VER" = "1.2.3" ]; then
    print_pass "All versions consistent (1.2.3)"
else
    print_fail "Version mismatch detected"
    print_info "README: $README_VER, SECURITY: $SECURITY_VER, ARCH: $ARCH_VER, DESIGN: $DESIGN_VER, PKGBUILD: $PKGBUILD_VER"
fi

#==============================================================================
# TEST 8: Edge Case - Invalid Cargo.toml (Missing Version)
#==============================================================================
print_header "Test 8: Edge Case - Missing Version in Cargo.toml"

# Create invalid Cargo.toml (no version field)
cat > "$TEST_DIR/Cargo.toml" <<'EOF'
[package]
name = "hypr-keybind-manager"
edition = "2021"
EOF

print_test "Running sync-version.sh with invalid Cargo.toml..."
cd "$TEST_DIR"
if bash "$OLDPWD/$SCRIPT_PATH" > /tmp/sync-version-error.log 2>&1; then
    print_fail "Script should have failed with invalid Cargo.toml"
else
    print_pass "Script correctly detected missing version"
    if grep -q "Error: Could not extract version" /tmp/sync-version-error.log; then
        print_pass "Error message correct"
    else
        print_fail "Error message incorrect or missing"
        cat /tmp/sync-version-error.log
    fi
fi
cd "$OLDPWD"

#==============================================================================
# TEST 9: Edge Case - Different Version Format
#==============================================================================
print_header "Test 9: Different Version Format"

# Create Cargo.toml with different version
cat > "$TEST_DIR/Cargo.toml" <<'EOF'
[package]
name = "hypr-keybind-manager"
version = "2.0.0"
edition = "2021"
EOF

print_test "Running sync-version.sh with version 2.0.0..."
cd "$TEST_DIR"
bash "$OLDPWD/$SCRIPT_PATH" > /tmp/sync-version-v2.log 2>&1
cd "$OLDPWD"

print_test "Verifying all files updated to 2.0.0..."
README_VER=$(grep -oP 'badge/version-\K[0-9]+\.[0-9]+\.[0-9]+' "$TEST_DIR/README.md" | head -1)
SECURITY_VER=$(grep -oP '^\*\*Version\*\*: \K[0-9]+\.[0-9]+\.[0-9]+' "$TEST_DIR/SECURITY.md" | head -1)

if [ "$README_VER" = "2.0.0" ] && [ "$SECURITY_VER" = "2.0.0" ]; then
    print_pass "All files correctly updated to 2.0.0"
else
    print_fail "Files not updated to 2.0.0"
    print_info "README: $README_VER, SECURITY: $SECURITY_VER"
fi

#==============================================================================
# SUMMARY
#==============================================================================
print_header "Test Summary"

TOTAL=$((PASSED + FAILED))
echo -e "${BLUE}Total Tests: $TOTAL${NC}"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed!${NC}\n"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Check output above.${NC}\n"
    exit 1
fi
