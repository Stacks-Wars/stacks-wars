#!/bin/bash

echo "🧪 Testing Auth Route Rate Limiting (50/min limit)"
echo "================================================="

# Make 70 requests to auth endpoint (50 limit + 20 extra)
echo "Making 70 rapid requests to /user (POST) - should hit 50/min limit..."
echo ""

success_count=0
rate_limited_count=0
other_count=0

for i in {1..70}; do
    response=$(curl -s -o /dev/null -w "%{http_code}" -X POST "http://localhost:3001/user" 2>/dev/null)

    if [ "$response" = "405" ] || [ "$response" = "400" ] || [ "$response" = "422" ]; then
        # These are expected for POST /user without proper data, but not rate limited
        success_count=$((success_count + 1))
        echo -n "✓"
    elif [ "$response" = "429" ]; then
        # Rate limited!
        rate_limited_count=$((rate_limited_count + 1))
        echo -n "✗"
    else
        other_count=$((other_count + 1))
        echo -n "?"
    fi

    # Print progress every 10 requests
    if [ $((i % 10)) -eq 0 ]; then
        echo " [$i/70]"
    fi

    sleep 0.01
done

echo ""
echo ""
echo "📊 Results:"
echo "   ✅ Successful requests: $success_count"
echo "   🚫 Rate limited (429): $rate_limited_count"
echo "   ❓ Other responses: $other_count"
echo ""

if [ $rate_limited_count -gt 0 ]; then
    echo "🎉 SUCCESS! Rate limiting is working!"
    echo "   └─ Hit rate limit after ~$success_count requests"
    echo "   └─ Blocked $rate_limited_count additional requests with 429"
else
    echo "⚠️  Rate limiting may not be working as expected"
fi

echo ""
echo "Expected: ~50 successful requests, then 429 responses"
echo "Limit: 50 requests per minute per IP address"
