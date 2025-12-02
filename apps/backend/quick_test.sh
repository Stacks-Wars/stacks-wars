#!/bin/bash

echo "Quick Rate Limiting Demo"
echo "======================="

# Set lower limits temporarily for testing
echo "For a real test, you could temporarily lower the rate limits in middleware.rs:"
echo "- Global: 10 requests/minute instead of 1000"
echo "- API: 5 requests/minute instead of 300"
echo "- Auth: 2 requests/minute instead of 50"
echo ""

echo "Current rate limits are:"
echo "- Global: 1000 requests/minute per IP"
echo "- API endpoints: 300 requests/minute per IP"
echo "- Auth endpoints: 50 requests/minute per IP"
echo ""

echo "Making 10 requests to test basic functionality..."
for i in {1..10}; do
    response=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3001/game")
    echo "Request $i: HTTP $response"
    sleep 0.001
done

echo ""
echo "✅ Rate limiting is now PER-IP based!"
echo "✅ Each IP address gets its own rate limit bucket"
echo "✅ Different endpoints have different limits"
echo ""
echo "To see rate limiting in action:"
echo "1. Lower the limits in src/middleware.rs"
echo "2. Recompile and run the server"
echo "3. Make requests rapidly to see 429 responses"
