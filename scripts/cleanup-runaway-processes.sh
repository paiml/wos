#!/bin/bash
# Cleanup script to kill runaway test and build processes
# Run this if the system is running low on memory or if tests are stuck

set -uo pipefail

echo "Checking for runaway processes..."

# Kill all cargo mutants processes
MUTANTS_COUNT=$(ps aux | grep -E "(cargo.mutants|rustc.*mutants)" | grep -v grep | wc -l)
if [ "$MUTANTS_COUNT" -gt 0 ]; then
    echo "Found $MUTANTS_COUNT cargo mutants processes. Killing..."
    ps aux | grep -E "(cargo.mutants|rustc.*mutants)" | grep -v grep | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true
fi

# Kill all playwright test processes
PLAYWRIGHT_COUNT=$(ps aux | grep -E "playwright test" | grep -v grep | wc -l)
if [ "$PLAYWRIGHT_COUNT" -gt 0 ]; then
    echo "Found $PLAYWRIGHT_COUNT playwright processes. Killing..."
    ps aux | grep -E "playwright test" | grep -v grep | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true
fi

# Kill any orphaned node processes from playwright
NODE_PLAYWRIGHT_COUNT=$(ps aux | grep -E "node.*playwright" | grep -v grep | wc -l)
if [ "$NODE_PLAYWRIGHT_COUNT" -gt 0 ]; then
    echo "Found $NODE_PLAYWRIGHT_COUNT node playwright processes. Killing..."
    ps aux | grep -E "node.*playwright" | grep -v grep | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true
fi

# Wait for processes to die
sleep 2

# Show current memory usage
echo ""
echo "Current memory usage after cleanup:"
free -h

# Check if swap is heavily used and suggest clearing it
SWAP_USED=$(free | grep Swap | awk '{if ($2 > 0) printf "%.0f", ($3/$2)*100; else print "0"}')
if [ "$SWAP_USED" -gt 80 ]; then
    echo ""
    echo "⚠️  WARNING: Swap usage is at ${SWAP_USED}%"
    echo "   Consider running: sudo swapoff -a && sudo swapon -a"
    echo "   This will clear swap and potentially free memory"
fi

echo ""
echo "Cleanup complete!"
