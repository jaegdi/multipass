#!/bin/bash

# Test script for kpasscli with Bitwarden backend

echo "=== Testing kpasscli with Bitwarden Backend ==="
echo ""

# Check if bw is installed
echo "1. Checking Bitwarden CLI installation..."
if ! command -v bw &> /dev/null; then
    echo "   ❌ Bitwarden CLI (bw) is not installed"
    echo "   Install it from: https://bitwarden.com/help/cli/"
    exit 1
fi
echo "   ✓ Bitwarden CLI version: $(bw --version)"
echo ""

# Check Bitwarden status
echo "2. Checking Bitwarden vault status..."
BW_STATUS=$(bw status | jq -r '.status')
echo "   Current status: $BW_STATUS"
echo ""

# Unlock vault if needed
if [ "$BW_STATUS" != "unlocked" ]; then
    echo "3. Vault is locked. You need to unlock it first."
    echo "   Run: export BW_SESSION=\$(bw unlock --raw)"
    echo "   Or run: bw unlock"
    echo ""
    echo "   After unlocking, run this script again."
    exit 1
fi
echo "3. ✓ Vault is unlocked"
echo ""

# List some items to verify
echo "4. Listing first 3 items in your vault..."
bw list items | jq -r '.[0:3] | .[] | "   - \(.name)"'
echo ""

# Build kpasscli
echo "5. Building kpasscli..."
cargo build --bin kpasscli --quiet
if [ $? -ne 0 ]; then
    echo "   ❌ Build failed"
    exit 1
fi
echo "   ✓ Build successful"
echo ""

# Test configuration
echo "6. Testing configuration loading..."
./target/debug/kpasscli --config config-bitwarden.yaml --print-config
echo ""

# Test search (you'll need to replace 'test' with an actual item name)
echo "7. Testing search functionality..."
echo "   Enter an item name to search for (or press Enter to skip):"
read -r ITEM_NAME

if [ -n "$ITEM_NAME" ]; then
    echo "   Searching for: $ITEM_NAME"
    ./target/debug/kpasscli --config config-bitwarden.yaml --debug -i "$ITEM_NAME"
    echo ""
    
    if [ $? -eq 0 ]; then
        echo "   ✓ Search successful!"
    else
        echo "   ⚠ Search failed or no items found"
    fi
else
    echo "   Skipped search test"
fi
echo ""

# Test show-all
if [ -n "$ITEM_NAME" ]; then
    echo "8. Testing show-all functionality..."
    ./target/debug/kpasscli --config config-bitwarden.yaml -i "$ITEM_NAME" --show-all
    echo ""
fi

echo "=== Test Complete ==="
