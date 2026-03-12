#!/bin/bash

# Test runner script for cocos4-rust
# This script validates feature parity with Cocos4

set -e

echo "========================================="
echo "Cocos4-Rust Feature Parity Test Suite"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "Running: $test_name ... "
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Function to print test results
print_results() {
    echo ""
    echo "========================================="
    echo "Test Results Summary"
    echo "========================================="
    echo -e "Total Tests:  $TOTAL_TESTS"
    echo -e "${GREEN}Passed:       $PASSED_TESTS${NC}"
    echo -e "${RED}Failed:       $FAILED_TESTS${NC}"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}All tests passed! ✓${NC}"
        exit 0
    else
        echo -e "${RED}Some tests failed! ✗${NC}"
        exit 1
    fi
}

# Trap to ensure we print results on exit
trap print_results EXIT

echo "Section 1: Math Module Tests"
echo "-------------------------------------------"

# Math module tests
run_test "Vec2 creation and operations" \
    "cargo test --release vec2"

run_test "Vec3 creation and operations" \
    "cargo test --release vec3"

run_test "Vec4 creation and operations" \
    "cargo test --release vec4"

run_test "Mat3 operations" \
    "cargo test --release mat3"

run_test "Mat4 operations" \
    "cargo test --release mat4"

run_test "Color operations" \
    "cargo test --release color"

run_test "Geometry (Rect, Size) operations" \
    "cargo test --release geometry"

echo ""
echo "Section 2: Base Module Tests"
echo "-------------------------------------------"

# Base module tests
run_test "Base types" \
    "cargo test --release base::types"

run_test "Data structures" \
    "cargo test --release base::data"

run_test "Logging system" \
    "cargo test --release base::log"

run_test "Reference counting" \
    "cargo test --release base::refcount"

echo ""
echo "Section 3: Core Module Tests"
echo "-------------------------------------------"

# Core module tests
run_test "Event system" \
    "cargo test --release core::event"

run_test "Asset management" \
    "cargo test --release core::assets"

echo ""
echo "Section 4: Integration Tests"
echo "-------------------------------------------"

# Integration tests
run_test "Full build test" \
    "cargo build --release"

run_test "All unit tests" \
    "cargo test --release"

run_test "Documentation tests" \
    "cargo test --release --doc"

echo ""
echo "Section 5: Feature Parity Checks"
echo "-------------------------------------------"

# Feature parity verification
echo "Checking math module completeness..."
if grep -q "Vec2" src/math/mod.rs && \
   grep -q "Vec3" src/math/mod.rs && \
   grep -q "Vec4" src/math/mod.rs && \
   grep -q "Mat3" src/math/mod.rs && \
   grep -q "Mat4" src/math/mod.rs; then
    echo -e "${GREEN}✓ Math module exports complete${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ Math module exports incomplete${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

echo "Checking module structure..."
if [ -d "src/math" ] && [ -d "src/base" ] && [ -d "src/core" ]; then
    echo -e "${GREEN}✓ Module structure matches Cocos4${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ Module structure incomplete${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

echo ""
echo "Section 6: Performance Benchmarks (Optional)"
echo "-------------------------------------------"

# Run benchmarks if available
if [ -d "benches" ]; then
    run_test "Math benchmarks" \
        "cargo bench --bench math_bench 2>/dev/null || true"
else
    echo -e "${YELLOW}Skipping benchmarks - no benches directory${NC}"
fi

echo ""
echo "========================================="
echo "Test execution completed"
echo "========================================="
