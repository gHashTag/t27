#!/bin/bash
# Test Agent Bridge CLI & Backend
#
# This script:
# 1. Starts the Trinity Core backend
# 2. Tests the agent-say.ts CLI utility
# 3. Displays results

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Agent Bridge Test Script ===${NC}"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Error: cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Check if node/tsx is available
if ! command -v node &> /dev/null; then
    echo -e "${YELLOW}Error: node not found. Please install Node.js.${NC}"
    exit 1
fi

# Check if pnpm is available
if ! command -v pnpm &> /dev/null; then
    echo -e "${YELLOW}Warning: pnpm not found. Using npm instead.${NC}"
    PKG_CMD="npm"
else
    PKG_CMD="pnpm"
fi

# Step 1: Build backend
echo -e "${BLUE}[1/4] Building Trinity Core backend...${NC}"
cd backend/trinity-core
cargo build --release 2>&1 || {
    echo -e "${YELLOW}Failed to build backend. Run manually first: cd backend/trinity-core && cargo build --release${NC}"
    exit 1
}
cd ../..
echo -e "${GREEN}✓ Backend built${NC}"

# Step 2: Start backend in background
echo ""
echo -e "${BLUE}[2/4] Starting backend on port 8082...${NC}"
cd backend/trinity-core
cargo run --release &
BACKEND_PID=$!
cd ../..

# Wait for backend to be ready
echo "Waiting for backend to start..."
sleep 3

# Check if backend is responding
if curl -s http://localhost:8082/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Backend is running${NC}"
else
    echo -e "${YELLOW}✗ Backend failed to start. Check logs above.${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# Step 3: Install dependencies for desktop app
echo ""
echo -e "${BLUE}[3/4] Installing desktop dependencies...${NC}"
cd apps/desktop
$PKG_CMD install --silent 2>&1 || {
    echo -e "${YELLOW}Note: Some packages may need manual installation${NC}"
}
cd ../..

# Step 4: Test agent-say CLI
echo ""
echo -e "${BLUE}[4/4] Testing agent-say CLI...${NC}"

# Test 1: Agent A message
echo ""
echo -e "${YELLOW}Test 1: Agent A sending a message...${NC}"
npx tsx scripts/agent-say.ts A message "Hello from Agent A" 💬 || {
    echo -e "${YELLOW}✗ Failed to send message${NC}"
}
echo ""

# Test 2: Queen (Q) sending test result
echo -e "${YELLOW}Test 2: Queen (Q) sending test result...${NC}"
npx tsx scripts/agent-say.ts Q test_result "All agents operational" 🧪 || {
    echo -e "${YELLOW}✗ Failed to send test result${NC}"
}
echo ""

# Test 3: 27th agent sending status
echo -e "${YELLOW}Test 3: 27th agent sending status...${NC}"
npx tsx scripts/agent-say.ts 27 status "Building spec..." 📡 || {
    echo -e "${YELLOW}✗ Failed to send status${NC}"
}
echo ""

# Test 4: Agent B sending error
echo -e "${YELLOW}Test 4: Agent B sending error...${NC}"
npx tsx scripts/agent-say.ts B error "Build failed" ❌ || {
    echo -e "${YELLOW}✗ Failed to send error${NC}"
}
echo ""

# Cleanup
echo -e "${BLUE}=== Tests Complete ===${NC}"
echo ""
echo -e "${GREEN}Backend PID: $BACKEND_PID${NC}"
echo "Press Ctrl+C to stop the backend"
echo ""
echo "To view agent sessions:"
echo "  curl http://localhost:8082/session?directory=/tmp/agent-a"
echo "  curl http://localhost:8082/session?directory=/tmp/agent-q"
echo ""

# Wait for user to stop
trap "echo -e '${YELLOW}Stopping backend (PID: $BACKEND_PID)...${NC}'; kill $BACKEND_PID 2>/dev/null; echo -e '${GREEN}✓ Backend stopped${NC}'; exit 0" INT TERM

# Keep script running
wait $BACKEND_PID
