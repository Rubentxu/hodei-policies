#!/bin/bash
# Script to run the Hodei Example Application and execute tests

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   Hodei Example Application - Test Runner${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if services are running
echo "🔍 Checking services..."

# Check PostgreSQL
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  PostgreSQL is not running${NC}"
    echo "Starting services with docker compose (dev)..."
    docker compose -f docker-compose.dev.yml up -d postgres redis
    echo "Waiting for services to be ready..."
    sleep 5
fi

# Check Redis
if ! redis-cli -h localhost -p 6379 ping > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Redis is not running${NC}"
    echo "Starting services with docker compose (dev)..."
    docker compose -f docker-compose.dev.yml up -d postgres redis
    echo "Waiting for services to be ready..."
    sleep 5
fi

echo -e "${GREEN}✓ Services are running${NC}"
echo ""

# Set environment variables
export DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/hodei_policies}"
export REDIS_URL="${REDIS_URL:-redis://localhost:6379}"

echo "📝 Configuration:"
echo "   DATABASE_URL: $DATABASE_URL"
echo "   REDIS_URL: $REDIS_URL"
echo ""

# Build the application
echo "🔨 Building application..."
cargo build -p app-example --release
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Start the server in background
echo "🚀 Starting server..."
cargo run -p app-example --release > /tmp/hodei-server.log 2>&1 &
SERVER_PID=$!

# Wait for server to be ready
echo "⏳ Waiting for server to be ready..."
MAX_WAIT=30
WAITED=0
while ! curl -s http://localhost:3000/health > /dev/null 2>&1; do
    sleep 1
    WAITED=$((WAITED + 1))
    if [ $WAITED -ge $MAX_WAIT ]; then
        echo -e "${RED}❌ Server failed to start within ${MAX_WAIT}s${NC}"
        echo "Server log:"
        cat /tmp/hodei-server.log
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
    echo -n "."
done
echo ""
echo -e "${GREEN}✓ Server is ready${NC}"
echo ""

# Run tests
echo "🧪 Running tests..."
echo ""
bash tests/app_example_tests.sh
TEST_EXIT_CODE=$?

# Cleanup
echo ""
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true
echo -e "${GREEN}✓ Server stopped${NC}"

# Show server log if tests failed
if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo ""
    echo "📋 Server log (last 50 lines):"
    tail -50 /tmp/hodei-server.log
fi

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}   ✅ ALL TESTS PASSED!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
else
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}   ❌ TESTS FAILED${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
fi

exit $TEST_EXIT_CODE
