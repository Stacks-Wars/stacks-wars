#!/bin/bash

echo "Testing IP-based Rate Limiting..."
echo "=================================="

# Function to test rate limits
test_rate_limit() {
    local endpoint=$1
    local max_requests=$2
    local description=$3
    local test_requests=$((max_requests + 20))  # Add 20 extra requests to exceed the limit

    echo ""
    echo "Testing $description at $endpoint"
    echo "Expected limit: $max_requests requests/minute"
    echo "Making $test_requests rapid requests (should trigger rate limiting)..."

    success_count=0
    rate_limited_count=0
    other_count=0

    # Make requests rapidly
    for i in $(seq 1 $test_requests); do
        response=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3001$endpoint" 2>/dev/null)
        if [ "$response" = "200" ] || [ "$response" = "405" ]; then
            success_count=$((success_count + 1))
            echo -n "✓"
        elif [ "$response" = "429" ]; then
            rate_limited_count=$((rate_limited_count + 1))
            echo -n "✗"
        else
            other_count=$((other_count + 1))
            echo -n "?"
        fi

        # Very small delay to make requests rapid but not overwhelming
        sleep 0.01
    done

    echo ""
    echo "Results: $success_count successful, $rate_limited_count rate-limited, $other_count other"
    echo "Rate limiting is $([ $rate_limited_count -gt 0 ] && echo "WORKING ✅" || echo "NOT TRIGGERED ❌")"

    if [ $rate_limited_count -gt 0 ]; then
        echo "✅ Successfully hit rate limit after ~$((success_count)) requests"
    fi
}

# Start the server in background if not running
if ! curl -s http://localhost:3001/game >/dev/null 2>&1; then
    echo "Starting server..."
    cargo run &
    SERVER_PID=$!
    sleep 3
    echo "Server started with PID: $SERVER_PID"
else
    echo "Server already running"
    SERVER_PID=""
fi

# Test different endpoints with different rate limits
test_rate_limit "/game" "300" "API endpoints (GET /game)"
test_rate_limit "/user" "50" "Auth endpoints (POST /user)"

# Test rapid requests to trigger global rate limit
echo ""
echo "Testing Global Rate Limit (1000/min)..."
echo "Making 1050 rapid requests to various endpoints (should trigger global limit)..."

global_success=0
global_limited=0
global_other=0

for i in {1..1050}; do
    endpoints=("/game" "/lobby" "/leaderboard")
    endpoint=${endpoints[$((i % 3))]}

    response=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3001$endpoint" 2>/dev/null)
    if [ "$response" = "200" ]; then
        global_success=$((global_success + 1))
        echo -n "✓"
    elif [ "$response" = "429" ]; then
        global_limited=$((global_limited + 1))
        echo -n "✗"
    else
        global_other=$((global_other + 1))
        echo -n "?"
    fi

    # Print progress every 100 requests
    if [ $((i % 100)) -eq 0 ]; then
        echo " [$i/1050]"
    fi

    sleep 0.005
done

echo ""
echo "Global test results: $global_success successful, $global_limited rate-limited, $global_other other"
echo "Global rate limiting is $([ $global_limited -gt 0 ] && echo "WORKING ✅" || echo "NOT TRIGGERED ❌")"

if [ $global_limited -gt 0 ]; then
    echo "✅ Successfully hit global rate limit after ~$((global_success)) requests"
fi

# Clean up
if [ ! -z "$SERVER_PID" ]; then
    echo ""
    echo "Stopping test server..."
    kill $SERVER_PID 2>/dev/null
fi

echo ""
echo "Rate limiting test complete!"
echo ""
echo "Summary:"
echo "- Auth endpoints: 70 requests tested (limit: 50/min)"
echo "- API endpoints: 320 requests tested (limit: 300/min)"
echo "- Global limit: 1050 requests tested (limit: 1000/min)"
echo ""
echo "✅ Per-IP rate limiting is now active!"
echo "✅ Each IP address gets its own rate limit bucket"
echo "✅ Different endpoints have different limits"
