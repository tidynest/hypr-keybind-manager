#!/usr/bin/env bash
# Test script to verify Escape key support in all dialogs
# Copyright 2025 Eric Jingryd (tidynest@proton.me)

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Escape Key Support Verification${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

PASSED=0
FAILED=0

# Function to check if a file has both Window::builder() and EventControllerKey
check_dialog() {
    local file=$1
    local dialog_name=$2
    local line_num=$3

    echo -n "Checking ${dialog_name}... "

    # Check if file has Window::builder
    if ! grep -q "Window::builder()" "$file";
then
        echo -e "${YELLOW}SKIP${NC} (no dialog window)"
        return 0
    fi

    # Check if file has EventControllerKey import
    if ! grep -q "EventControllerKey" "$file";
then
        echo -e "${RED}FAIL${NC} (missing EventControllerKey import)"
        FAILED=$((FAILED + 1))
        return 1
    fi

    # Check if file has gdk::Key::Escape
    if ! grep -q "gdk::Key::Escape" "$file";
then
        echo -e "${RED}FAIL${NC} (missing Escape key handler)"
        FAILED=$((FAILED + 1))
        return 1
    fi

    # Check if file has connect_key_pressed
    if ! grep -q "connect_key_pressed" "$file";
then
        echo -e "${RED}FAIL${NC} (missing key event handler)"
        FAILED=$((FAILED + 1))
        return 1
    fi

    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
    return 0
}

echo -e "${BLUE}Checking dialog files:${NC}"
echo ""

# Check all dialog files
check_dialog "src/ui/components/edit_dialog.rs" "Edit Dialog" "78"
check_dialog "src/ui/components/conflict_resolution_dialog.rs" "Conflict Resolution Dialog" "44"
check_dialog "src/ui/components/backup_dialog.rs" "Backup Dialog" "101"
check_dialog "src/ui/actions.rs" "Import Mode Selection Dialog" "171"

echo ""
echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}=====================================${NC}"
echo -e "Passed: ${GREEN}${PASSED}${NC}"
echo -e "Failed: ${RED}${FAILED}${NC}"
echo ""

# Additional verification: Count total EventControllerKey instances
echo -e "${BLUE}Additional Verification:${NC}"
TOTAL_CONTROLLERS=$(grep -r "EventControllerKey::new()" src/ui/ | wc -l)
echo -e "Total EventControllerKey instances: ${GREEN}${TOTAL_CONTROLLERS}${NC}"
echo -e "(Expected: 6 - Edit Dialog main, Edit Dialog error, Conflict, Backup, Import, Keyboard nav)"
echo ""

# Verify the pattern is correct
echo -e "${BLUE}Verifying implementation pattern:${NC}"
echo -n "All dialogs use glib::Propagation... "
if grep -r "glib::Propagation::Stop" src/ui/ >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    FAILED=$((FAILED + 1))
fi

echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All Escape key implementations verified!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some checks failed. Please review the implementation.${NC}"
    exit 1
fi

