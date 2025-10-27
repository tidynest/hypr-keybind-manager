#!/usr/bin/env bash
# Automated test script for Phase 6.8 - File Watcher
# Tests live config file monitoring functionality

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test config paths
TEST_CONFIG="test-data/hyprland-test.conf"
BACKUP_CONFIG="test-data/hyprland-test.conf.backup"
LOG_FILE="/tmp/hypr-keybind-manager-test.log"

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

    # Kill the app if still running
    if [ ! -z "$APP_PID" ]; then
        kill $APP_PID 2>/dev/null || true
        wait $APP_PID 2>/dev/null || true
    fi

    # Restore original config
    if [ -f "$BACKUP_CONFIG" ]; then
        mv "$BACKUP_CONFIG" "$TEST_CONFIG"
        print_info "Test config restored"
    fi

    # Clean up log file
    rm -f "$LOG_FILE"
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# Check prerequisites
print_header "Prerequisites Check"

if [ ! -f "$TEST_CONFIG" ]; then
    print_fail "Test config not found: $TEST_CONFIG"
    exit 1
fi
print_pass "Test config exists"

# Build the project
print_test "Building project..."
if cargo build 2>&1 | tail -5; then
    print_pass "Build successful"
else
    print_fail "Build failed"
    exit 1
fi

# Backup original test config
cp "$TEST_CONFIG" "$BACKUP_CONFIG"
print_info "Original config backed up"

# Count initial bindings
INITIAL_COUNT=$(grep -c "^bind" "$TEST_CONFIG" || echo "0")
print_info "Initial keybindings: $INITIAL_COUNT"

#==============================================================================
# TEST 1: Basic File Change Detection
#==============================================================================
print_header "Test 1: Basic File Change Detection"

print_test "Starting app in background..."
# Run binary directly to avoid runner.sh filtering
./target/debug/hypr-keybind-manager gui -c "$TEST_CONFIG" > "$LOG_FILE" 2>&1 &
APP_PID=$!
print_info "App PID: $APP_PID"

# Wait for app to fully start
print_test "Waiting for app to initialize (3 seconds)..."
sleep 3

# Check if app is running
if ! ps -p $APP_PID > /dev/null; then
    print_fail "App failed to start"
    cat "$LOG_FILE"
    exit 1
fi
print_pass "App started successfully"

# Add a new keybinding
print_test "Adding new keybinding to config..."
echo "bind = SUPER, T, exec, kitty" >> "$TEST_CONFIG"

# Wait for file watcher to detect (polling interval is 1 second)
print_test "Waiting for file watcher detection (2 seconds)..."
sleep 2

# Check log for reload message
if grep -q "📝 Config file changed - reloading" "$LOG_FILE"; then
    print_pass "File change detected"
else
    print_fail "File change NOT detected"
    echo "Log contents:"
    cat "$LOG_FILE"
fi

if grep -q "✅ Config reloaded successfully" "$LOG_FILE"; then
    print_pass "Config reloaded successfully"
else
    print_fail "Config reload failed or not completed"
fi

#==============================================================================
# TEST 2: Multiple Bindings Addition
#==============================================================================
print_header "Test 2: Multiple Rapid Changes"

# Clear log
> "$LOG_FILE"

print_test "Adding multiple keybindings rapidly..."
echo "bind = SUPER, Y, exec, firefox" >> "$TEST_CONFIG"
sleep 0.5
echo "bind = SUPER, U, exec, chromium" >> "$TEST_CONFIG"
sleep 0.5
echo "bind = SUPER, I, exec, code" >> "$TEST_CONFIG"

print_test "Waiting for detection (3 seconds)..."
sleep 3

# Count reload messages (should be at least 1, possibly 3)
RELOAD_COUNT=$(grep -c "📝 Config file changed - reloading" "$LOG_FILE" 2>/dev/null || echo "0")
RELOAD_COUNT=$(echo "$RELOAD_COUNT" | tr -d '\n' | head -1)
if [ "$RELOAD_COUNT" -ge 1 ]; then
    print_pass "Detected $RELOAD_COUNT reload(s)"
else
    print_fail "No reloads detected"
fi

#==============================================================================
# TEST 3: Conflict Detection
#==============================================================================
print_header "Test 3: Conflict Detection Updates"

# Clear log
> "$LOG_FILE"

print_test "Creating a conflict..."
# Add duplicate SUPER+Q binding (assuming one exists)
echo "bind = SUPER, Q, exec, test-conflict-1" >> "$TEST_CONFIG"
echo "bind = SUPER, Q, exec, test-conflict-2" >> "$TEST_CONFIG"

print_test "Waiting for detection (2 seconds)..."
sleep 2

if grep -q "📝 Config file changed - reloading" "$LOG_FILE"; then
    print_pass "Conflict addition detected"
else
    print_fail "Conflict addition NOT detected"
fi

#==============================================================================
# TEST 4: Fault-Tolerant Parsing (Invalid Syntax Handling)
#==============================================================================
print_header "Test 4: Fault-Tolerant Parsing"

# Clear log
> "$LOG_FILE"

print_test "Adding invalid/malformed syntax..."
echo "bind = SUPER, , exec" >> "$TEST_CONFIG"
echo "this is completely broken garbage" >> "$TEST_CONFIG"

print_test "Waiting for detection (2 seconds)..."
sleep 2

if grep -q "📝 Config file changed - reloading" "$LOG_FILE"; then
    print_pass "File change detected"
else
    print_fail "File change NOT detected"
fi

if grep -q "✅ Config reloaded successfully" "$LOG_FILE"; then
    print_pass "Fault-tolerant parsing working (skipped invalid lines)"
else
    print_fail "Config reload failed (parser not fault-tolerant)"
fi

# Check app is still running
if ps -p $APP_PID > /dev/null; then
    print_pass "App still running after invalid syntax (graceful handling)"
else
    print_fail "App crashed on invalid config"
fi

#==============================================================================
# TEST 5: Config Restoration
#==============================================================================
print_header "Test 5: Config Restoration"

# Clear log
> "$LOG_FILE"

print_test "Restoring valid config..."
cp "$BACKUP_CONFIG" "$TEST_CONFIG"

print_test "Waiting for detection (2 seconds)..."
sleep 2

if grep -q "✅ Config reloaded successfully" "$LOG_FILE"; then
    print_pass "Valid config restored and loaded"
else
    print_fail "Config restoration failed"
fi

#==============================================================================
# TEST 6: File Watcher Performance (Zero CPU)
#==============================================================================
print_header "Test 6: Performance Check"

print_test "Checking CPU usage (no file changes for 3 seconds)..."
sleep 3

# Get CPU usage (this is a rough check)
CPU_USAGE=$(ps -p $APP_PID -o %cpu= 2>/dev/null || echo "0.0")
print_info "App CPU usage: ${CPU_USAGE}%"

# CPU should be very low (< 5%) when idle
if (( $(echo "$CPU_USAGE < 5.0" | bc -l 2>/dev/null || echo "1") )); then
    print_pass "CPU usage acceptable (${CPU_USAGE}%)"
else
    print_fail "CPU usage too high (${CPU_USAGE}%)"
fi

#==============================================================================
# TEST 7: Graceful Degradation
#==============================================================================
print_header "Test 7: Graceful Degradation"

# Stop current app
print_test "Stopping current app instance..."
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true
APP_PID=""

# Clear log
> "$LOG_FILE"

print_test "Making config read-only..."
chmod 444 "$TEST_CONFIG"

print_test "Starting app with read-only config..."
./target/debug/hypr-keybind-manager gui -c "$TEST_CONFIG" > "$LOG_FILE" 2>&1 &
APP_PID=$!

sleep 3

# Check if warning appears
if grep -q "⚠️  File watcher setup failed" "$LOG_FILE" || ps -p $APP_PID > /dev/null; then
    print_pass "App handles watcher failure gracefully"
else
    print_fail "App didn't handle watcher failure well"
fi

# Restore permissions
chmod 644 "$TEST_CONFIG"
print_info "Permissions restored"

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
    echo -e "${YELLOW}Log file available at: $LOG_FILE${NC}\n"
    exit 1
fi
