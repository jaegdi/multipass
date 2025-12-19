# Testing kpasscli with Bitwarden Backend

## Prerequisites

1. **Bitwarden CLI installed**: You have version 2025.11.0 ✓
2. **Logged in to Bitwarden**: You are logged in as dirk_jaeger@email.de ✓
3. **Vault unlocked**: Your vault is currently **locked** ❌

## Quick Start

### 1. Unlock your Bitwarden vault

```bash
# Option A: Unlock and export session (recommended)
export BW_SESSION=$(bw unlock --raw)

# Option B: Just unlock (you'll need to enter password each time)
bw unlock
```

### 2. Run the automated test script

```bash
./test-bitwarden.sh
```

This script will:
- Check Bitwarden CLI installation
- Verify vault status
- Build kpasscli
- Test configuration loading
- Allow you to search for items interactively

### 3. Manual testing

Once your vault is unlocked, you can test manually:

```bash
# Test 1: Print configuration
cargo run --bin kpasscli -- --config config-bitwarden.yaml --print-config

# Test 2: Search for an item (replace "github" with an actual item name)
cargo run --bin kpasscli -- --config config-bitwarden.yaml -i "github"

# Test 3: Search with debug output
cargo run --bin kpasscli -- --config config-bitwarden.yaml --debug -i "github"

# Test 4: Show all fields of an item
cargo run --bin kpasscli -- --config config-bitwarden.yaml -i "github" --show-all

# Test 5: Get username instead of password
cargo run --bin kpasscli -- --config config-bitwarden.yaml -i "github" -f username

# Test 6: Copy password to clipboard
cargo run --bin kpasscli -- --config config-bitwarden.yaml -i "github" -C
```

## Configuration

The `config-bitwarden.yaml` file is configured as follows:

```yaml
database_path: bitwarden  # This triggers the Bitwarden backend
default_output: stdout
password_file: ~/pwpasswd  # Not used for Bitwarden
password_executable:       # Not used for Bitwarden
```

## How it Works

1. **Backend Detection**: When `database_path` is set to `"bitwarden"`, kpasscli automatically selects the Bitwarden backend
2. **Authentication**: The backend uses the Bitwarden CLI (`bw`) which must be unlocked separately
3. **Session Management**: If you have `BW_SESSION` environment variable set, it will be used automatically
4. **Search**: The backend uses `bw list items --search <query>` to find items

## Test Results

### Build Status
✓ Project builds successfully (16.04s)

### Backend Initialization
✓ Bitwarden backend initializes successfully (~3.4s)
✓ Detects backend type correctly from config
✓ Checks for `bw` CLI availability

### Known Issues
- Vault must be unlocked before using kpasscli
- The current implementation uses basic string parsing for JSON (should use serde_json in production)
- No automatic vault unlocking (by design - security feature)

## Troubleshooting

### "no items found"
- Make sure your vault is unlocked
- Verify the item name exists in your vault: `bw list items | jq -r '.[].name'`
- Try case-insensitive search (default behavior)

### "Failed to execute 'bw' command"
- Install Bitwarden CLI: https://bitwarden.com/help/cli/
- Make sure `bw` is in your PATH

### "Bitwarden command failed: You are not logged in"
- Run: `bw login`

### "Bitwarden command failed: Vault is locked"
- Run: `export BW_SESSION=$(bw unlock --raw)`

## Next Steps

To fully test the Bitwarden backend:

1. Unlock your vault: `export BW_SESSION=$(bw unlock --raw)`
2. List your items to find a test item: `bw list items | jq -r '.[].name' | head -10`
3. Run the test script: `./test-bitwarden.sh`
4. Try searching for one of your actual items
