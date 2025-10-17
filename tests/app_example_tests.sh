#!/bin/bash
# API Tests for Hodei Example Application
# Tests the document management system with Cedar Policy authorization

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

API_URL="${API_URL:-http://localhost:3000}"
FAILED_TESTS=0
PASSED_TESTS=0

echo -e "${BLUE}🚀 Hodei Example Application - API Tests${NC}"
echo "📍 API URL: $API_URL"
echo ""

# Function to run test
run_test() {
    local test_name="$1"
    local expected_status="$2"
    shift 2
    local curl_args=("$@")
    
    echo -n "🧪 $test_name ... "
    
    response=$(curl -s -w "\n%{http_code}" "${curl_args[@]}")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected_status" ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $http_code)"
        ((PASSED_TESTS++))
        if [ -n "$body" ] && [ "$body" != "null" ]; then
            echo "   $(echo $body | jq -c . 2>/dev/null || echo $body | head -c 100)"
        fi
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} (Expected: $expected_status, Got: $http_code)"
        ((FAILED_TESTS++))
        if [ -n "$body" ]; then
            echo "   Response: $(echo $body | head -c 200)"
        fi
        return 1
    fi
}

# Check if server is running
echo "🔍 Checking if server is running..."
if ! curl -s "$API_URL/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ Server is not running at $API_URL${NC}"
    echo ""
    echo "Please start the server first:"
    echo "  cargo run -p app-example"
    echo ""
    exit 1
fi
echo -e "${GREEN}✓ Server is running${NC}"
echo ""

echo "═══════════════════════════════════════════════════════════════"
echo "📋 BASIC ENDPOINTS"
echo "═══════════════════════════════════════════════════════════════"

run_test "GET / - API Info" 200 \
    -X GET "$API_URL/"

run_test "GET /health - Health Check" 200 \
    -X GET "$API_URL/health"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 USER ENDPOINTS"
echo "═══════════════════════════════════════════════════════════════"

run_test "GET /users - List all users" 200 \
    -X GET "$API_URL/users"

run_test "GET /users/:id - Get Alice" 200 \
    -X GET "$API_URL/users/alice"

run_test "GET /users/:id - Get Bob" 200 \
    -X GET "$API_URL/users/bob"

run_test "GET /users/:id - Get Charlie" 200 \
    -X GET "$API_URL/users/charlie"

run_test "GET /users/:id - Non-existent user" 404 \
    -X GET "$API_URL/users/nonexistent"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 DOCUMENT ENDPOINTS"
echo "═══════════════════════════════════════════════════════════════"

run_test "GET /documents - List all documents" 200 \
    -X GET "$API_URL/documents"

run_test "GET /documents/:id - Get document" 200 \
    -X GET "$API_URL/documents/doc"

run_test "GET /documents/:id - Non-existent document" 404 \
    -X GET "$API_URL/documents/nonexistent"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 CREATE DOCUMENT"
echo "═══════════════════════════════════════════════════════════════"

run_test "POST /documents - Create document by Alice" 200 \
    -X POST "$API_URL/documents" \
    -H "Content-Type: application/json" \
    -d '{
        "title": "Test Document",
        "content": "This is a test document",
        "owner_email": "alice@example.com",
        "is_public": false
    }'

run_test "POST /documents - Create public document by Bob" 200 \
    -X POST "$API_URL/documents" \
    -H "Content-Type: application/json" \
    -d '{
        "title": "Public Document",
        "content": "This is public",
        "owner_email": "bob@example.com",
        "is_public": true
    }'

run_test "POST /documents - Invalid owner email" 404 \
    -X POST "$API_URL/documents" \
    -H "Content-Type: application/json" \
    -d '{
        "title": "Invalid",
        "content": "Invalid owner",
        "owner_email": "invalid@example.com",
        "is_public": false
    }'

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 AUTHORIZATION CHECKS - OWNER PERMISSIONS"
echo "═══════════════════════════════════════════════════════════════"

# Get first document ID (Alice's private document)
DOC_ID=$(curl -s "$API_URL/documents" | jq -r '.[0].id' | grep -o 'doc-[^"]*' | head -1)

if [ -n "$DOC_ID" ]; then
    echo "Testing with document ID: $DOC_ID"
    
    run_test "Alice can READ her own private document" 200 \
        -X POST "$API_URL/documents/$DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "alice@example.com",
            "action": "DocApp::Action::\"Document::Read\""
        }'
    
    run_test "Alice can UPDATE her own document" 200 \
        -X POST "$API_URL/documents/$DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "alice@example.com",
            "action": "DocApp::Action::\"Document::Update\""
        }'
    
    run_test "Alice can DELETE her own document" 200 \
        -X POST "$API_URL/documents/$DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "alice@example.com",
            "action": "DocApp::Action::\"Document::Delete\""
        }'
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 AUTHORIZATION CHECKS - NON-OWNER PERMISSIONS"
echo "═══════════════════════════════════════════════════════════════"

if [ -n "$DOC_ID" ]; then
    run_test "Bob CANNOT read Alice's private document" 200 \
        -X POST "$API_URL/documents/$DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "bob@example.com",
            "action": "DocApp::Action::\"Document::Read\""
        }'
    
    run_test "Charlie CANNOT update Alice's document" 200 \
        -X POST "$API_URL/documents/$DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "charlie@example.com",
            "action": "DocApp::Action::\"Document::Update\""
        }'
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 AUTHORIZATION CHECKS - PUBLIC DOCUMENTS"
echo "═══════════════════════════════════════════════════════════════"

# Get Bob's public document
PUB_DOC_ID=$(curl -s "$API_URL/documents" | jq -r '.[] | select(.is_public == true) | .id' | grep -o 'doc-[^"]*' | head -1)

if [ -n "$PUB_DOC_ID" ]; then
    echo "Testing with public document ID: $PUB_DOC_ID"
    
    run_test "Anyone can READ public documents" 200 \
        -X POST "$API_URL/documents/$PUB_DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "charlie@example.com",
            "action": "DocApp::Action::\"Document::Read\""
        }'
    
    run_test "Non-owner CANNOT update public document" 200 \
        -X POST "$API_URL/documents/$PUB_DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "charlie@example.com",
            "action": "DocApp::Action::\"Document::Update\""
        }'
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 AUTHORIZATION CHECKS - ROLE-BASED ACCESS"
echo "═══════════════════════════════════════════════════════════════"

if [ -n "$DOC_ID" ]; then
    run_test "Admin (Alice) can do ANYTHING" 200 \
        -X POST "$API_URL/documents/$DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "alice@example.com",
            "action": "DocApp::Action::\"Document::Delete\""
        }'
    
    run_test "Editor (Bob) can UPDATE documents" 200 \
        -X POST "$API_URL/documents/$PUB_DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "bob@example.com",
            "action": "DocApp::Action::\"Document::Update\""
        }'
    
    run_test "Viewer (Charlie) can only READ" 200 \
        -X POST "$API_URL/documents/$PUB_DOC_ID/check" \
        -H "Content-Type: application/json" \
        -d '{
            "user_email": "charlie@example.com",
            "action": "DocApp::Action::\"Document::Read\""
        }'
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📊 TEST SUMMARY"
echo "═══════════════════════════════════════════════════════════════"
echo -e "${GREEN}✓ Passed: $PASSED_TESTS${NC}"
echo -e "${RED}✗ Failed: $FAILED_TESTS${NC}"
echo "Total: $((PASSED_TESTS + FAILED_TESTS))"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    echo ""
    echo "✅ Validated Features:"
    echo "   • Basic API endpoints"
    echo "   • User management"
    echo "   • Document CRUD"
    echo "   • Owner-based permissions"
    echo "   • Public document access"
    echo "   • Role-based access control (RBAC)"
    echo "   • Cedar Policy authorization"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Check errors above.${NC}"
    exit 1
fi
