#!/bin/bash

echo "=== Testing Request ID Middleware ==="
echo

# Test 1: Auto-generated request ID
echo "1. Testing auto-generated request ID:"
curl -s -D - -o /dev/null http://localhost:8080/api/users 2>/dev/null | grep -iE "(x-request-id|HTTP/)" | head -2
echo

# Test 2: Client-provided request ID
echo "2. Testing client-provided request ID:"
curl -s -H "X-Request-ID: client-12345" -D - -o /dev/null http://localhost:8080/api/users 2>/dev/null | grep -iE "(x-request-id|HTTP/)" | head -2
echo

# Test 3: Invalid request ID (should be replaced)
echo "3. Testing invalid request ID:"
curl -s -H "X-Request-ID: invalid@id!" -D - -o /dev/null http://localhost:8080/api/users 2>/dev/null | grep -iE "(x-request-id|HTTP/)" | head -2
echo

# Test 4: Request correlation across multiple requests
echo "4. Testing request correlation:"
REQUEST_ID="correlation-test-$(date +%s)"
echo "Using request ID: $REQUEST_ID"
curl -s -H "X-Request-ID: $REQUEST_ID" http://localhost:8080/api/users > /dev/null
curl -s -H "X-Request-ID: $REQUEST_ID" http://localhost:8080/health > /dev/null
echo "Check server logs for correlation with ID: $REQUEST_ID"
echo

# Test 5: Error scenario with tracing
echo "5. Testing error scenario with request ID:"
curl -s -H "X-Request-ID: error-trace-123" -D - http://localhost:8080/api/error 2>/dev/null | grep -iE "(x-request-id|HTTP/)" | head -2

echo
echo "=== Request ID Test Complete ==="
