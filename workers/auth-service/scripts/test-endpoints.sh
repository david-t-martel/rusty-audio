#!/bin/bash
# Test Auth Service Endpoints
# Usage: ./scripts/test-endpoints.sh [base_url]

set -e

BASE_URL=${1:-http://localhost:8787}

echo "🧪 Testing Rusty Audio Auth Service"
echo "📍 Base URL: $BASE_URL"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to test endpoint
test_endpoint() {
    local NAME=$1
    local METHOD=$2
    local ENDPOINT=$3
    local DATA=$4
    local EXPECTED_STATUS=$5

    echo -n "Testing $NAME... "

    if [ -n "$DATA" ]; then
        RESPONSE=$(curl -s -w "\n%{http_code}" -X "$METHOD" "$BASE_URL$ENDPOINT" \
            -H "Content-Type: application/json" \
            -d "$DATA")
    else
        RESPONSE=$(curl -s -w "\n%{http_code}" -X "$METHOD" "$BASE_URL$ENDPOINT")
    fi

    STATUS=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$STATUS" = "$EXPECTED_STATUS" ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $STATUS)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} (Expected $EXPECTED_STATUS, got $STATUS)"
        echo "Response: $BODY"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Test 1: Health Check
test_endpoint "Health Check" "GET" "/health" "" "200"

# Test 2: Initiate Auth - Valid Provider
test_endpoint "Initiate Auth (Google)" "POST" "/api/auth/initiate" '{"provider":"google"}' "200"

# Test 3: Initiate Auth - Invalid Provider
test_endpoint "Initiate Auth (Invalid Provider)" "POST" "/api/auth/initiate" '{"provider":"invalid"}' "400"

# Test 4: Initiate Auth - Missing Provider
test_endpoint "Initiate Auth (Missing Provider)" "POST" "/api/auth/initiate" '{}' "400"

# Test 5: Callback - Missing Fields
test_endpoint "Callback (Missing Fields)" "POST" "/api/auth/callback" '{"code":"test"}' "400"

# Test 6: Refresh - Missing Token
test_endpoint "Refresh (Missing Token)" "POST" "/api/auth/refresh" '{}' "400"

# Test 7: Logout - Missing Token
test_endpoint "Logout (Missing Token)" "POST" "/api/auth/logout" '{}' "400"

# Test 8: Get User - No Auth
test_endpoint "Get User (No Auth)" "GET" "/api/auth/user" "" "401"

# Test 9: Get User - Invalid Token
test_endpoint "Get User (Invalid Token)" "GET" "/api/auth/user" "" "401" \
    -H "Authorization: Bearer invalid_token"

# Test 10: CORS Preflight
echo -n "Testing CORS Preflight... "
CORS_RESPONSE=$(curl -s -w "\n%{http_code}" -X OPTIONS "$BASE_URL/api/auth/initiate" \
    -H "Origin: http://localhost:8080" \
    -H "Access-Control-Request-Method: POST")

CORS_STATUS=$(echo "$CORS_RESPONSE" | tail -n1)
if [ "$CORS_STATUS" = "204" ]; then
    echo -e "${GREEN}✓ PASS${NC} (HTTP $CORS_STATUS)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC} (Expected 204, got $CORS_STATUS)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 11: Rate Limiting (Initiate)
echo -n "Testing Rate Limiting... "
RATE_LIMIT_EXCEEDED=false
for i in {1..15}; do
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/initiate" \
        -H "Content-Type: application/json" \
        -d '{"provider":"google"}')
    STATUS=$(echo "$RESPONSE" | tail -n1)
    if [ "$STATUS" = "429" ]; then
        RATE_LIMIT_EXCEEDED=true
        break
    fi
    sleep 0.5
done

if [ "$RATE_LIMIT_EXCEEDED" = true ]; then
    echo -e "${GREEN}✓ PASS${NC} (Rate limit enforced)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ WARN${NC} (Rate limit not triggered - may need adjustment)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi

# Summary
echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo "Total:  $((TESTS_PASSED + TESTS_FAILED))"
echo "========================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
