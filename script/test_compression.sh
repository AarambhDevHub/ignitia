#!/bin/bash
echo "=== Testing Compression Middleware ==="
echo

# Wait for server to start
sleep 2

# Test 1: No compression (no Accept-Encoding header)
echo "1. Testing without Accept-Encoding header:"
curl -s -D - -o /dev/null http://localhost:8080/api/data 2>/dev/null | grep -iE "(content-encoding|content-length|HTTP/)" | head -3
echo

# Test 2: Gzip compression
echo "2. Testing with gzip compression:"
curl -s -H "Accept-Encoding: gzip" -D - -o /dev/null http://localhost:8080/api/data 2>/dev/null | grep -iE "(content-encoding|content-length|HTTP/)" | head -3
echo

# Test 3: Brotli compression
echo "3. Testing with brotli compression:"
curl -s -H "Accept-Encoding: br" -D - -o /dev/null http://localhost:8080/api/data 2>/dev/null | grep -iE "(content-encoding|content-length|HTTP/)" | head -3
echo

# Test 4: Quality preferences (brotli preferred)
echo "4. Testing with quality preferences (br preferred):"
curl -s -H "Accept-Encoding: gzip;q=0.8, br;q=1.0" -D - -o /dev/null http://localhost:8080/api/data 2>/dev/null | grep -iE "(content-encoding|content-length|HTTP/)" | head -3
echo

# Test 5: Small response (below threshold)
echo "5. Testing small response (should not be compressed):"
curl -s -H "Accept-Encoding: gzip" -D - -o /dev/null http://localhost:8080/small 2>/dev/null | grep -iE "(content-encoding|content-length|HTTP/)" | head -3
echo

# Test 6: Large text response
echo "6. Testing large text response:"
curl -s -H "Accept-Encoding: gzip" -D - -o /dev/null http://localhost:8080/large-text 2>/dev/null | grep -iE "(content-encoding|content-length|HTTP/)" | head -3
echo

echo "=== Compression Test Complete ==="
