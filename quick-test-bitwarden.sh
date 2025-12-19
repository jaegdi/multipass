#!/bin/bash

# Quick unlock and test script for Bitwarden backend

echo "=== Quick Bitwarden Test ==="
echo ""
echo "This script will help you unlock your vault and test kpasscli"
echo ""

# Check status
BW_STATUS=$(bw status 2>/dev/null | jq -r '.status' 2>/dev/null)

if [ "$BW_STATUS" != "unlocked" ]; then
    echo "Your vault is currently: $BW_STATUS"
    echo ""
    echo "Please unlock your vault. Enter your master password:"
    
    # Unlock and get session
    SESSION=$(bw unlock --raw)
    
    if [ $? -ne 0 ]; then
        echo "Failed to unlock vault"
        exit 1
    fi
    
    export BW_SESSION="$SESSION"
    echo ""
    echo "✓ Vault unlocked successfully!"
    echo "✓ Session exported to BW_SESSION"
else
    echo "✓ Vault is already unlocked"
fi

echo ""
echo "=== Available items in your vault ==="
bw list items --session "$BW_SESSION" 2>/dev/null | jq -r '.[0:10] | .[] | "  - \(.name)"' 2>/dev/null

echo ""
echo "=== Testing kpasscli ==="
echo ""

# Get first item name for testing
FIRST_ITEM=$(bw list items --session "$BW_SESSION" 2>/dev/null | jq -r '.[0].name' 2>/dev/null)

if [ -n "$FIRST_ITEM" ]; then
    echo "Testing with item: $FIRST_ITEM"
    echo ""
    
    echo "1. Searching for item..."
    cargo run --bin kpasscli -- --config config-bitwarden.yaml -i "$FIRST_ITEM" 2>&1
    
    echo ""
    echo "2. Showing all fields..."
    cargo run --bin kpasscli -- --config config-bitwarden.yaml -i "$FIRST_ITEM" --show-all 2>&1
else
    echo "No items found in vault. Please add some items first."
fi

echo ""
echo "=== Test Complete ==="
echo ""
echo "Note: Your BW_SESSION is set for this terminal session."
echo "To use it in other terminals, run:"
echo "  export BW_SESSION=\"$BW_SESSION\""
