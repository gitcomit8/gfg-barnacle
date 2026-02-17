#!/bin/bash

# Test script to demonstrate the infinite redirect bug
# This script assumes the server is running on localhost:8080

echo "=========================================="
echo "Testing Module 14 - Auth Guard Bug"
echo "=========================================="
echo ""

# Check if server is running
if ! curl -s http://localhost:8080 > /dev/null 2>&1; then
    echo "❌ Error: Server is not running!"
    echo "Please start the server with: cargo run"
    exit 1
fi

echo "✓ Server is running"
echo ""

# Test 1: Home page redirect
echo "Test 1: Home page (should redirect to /login)"
echo "---"
RESPONSE=$(curl -s -I http://localhost:8080/ 2>&1)
if echo "$RESPONSE" | grep -q "location: /login"; then
    echo "✓ Home page redirects to /login"
else
    echo "✗ Unexpected response"
fi
echo ""

# Test 2: Login page infinite redirect bug
echo "Test 2: Login page (should show infinite redirect loop)"
echo "---"
RESPONSE=$(curl -s -I -L --max-redirs 5 http://localhost:8080/login 2>&1)
REDIRECT_COUNT=$(echo "$RESPONSE" | grep -c "HTTP")
if [ "$REDIRECT_COUNT" -gt 3 ]; then
    echo "✓ Bug confirmed! Login page causes infinite redirects"
    echo "  Detected $REDIRECT_COUNT redirect responses"
else
    echo "✗ Bug not reproduced"
fi
echo ""

# Test 3: Dashboard redirect
echo "Test 3: Dashboard (should redirect to /login)"
echo "---"
RESPONSE=$(curl -s -I http://localhost:8080/dashboard 2>&1)
if echo "$RESPONSE" | grep -q "location: /login"; then
    echo "✓ Dashboard correctly redirects unauthenticated users"
else
    echo "✗ Unexpected response"
fi
echo ""

echo "=========================================="
echo "Summary:"
echo "=========================================="
echo "The bug is in src/auth_middleware.rs"
echo ""
echo "Problem: The middleware checks authentication for ALL routes,"
echo "including /login itself, causing an infinite redirect loop."
echo ""
echo "To fix:"
echo "1. Add route whitelisting in auth_middleware.rs"
echo "2. Allow public access to /login, /, and /auth/* routes"
echo "3. See SOLUTION.md for detailed fix instructions"
echo ""
