# WIKEv2 Connect — Testing Guide

## Overview

WIKEv2 Connect uses three levels of testing:

1. **Unit Tests** — test individual functions in isolation
2. **Integration Tests** — test complete workflows
3. **Manual Tests** — real-environment testing

---

## Unit Tests

### Running

```bash
cargo test --lib
```

### Locations

```
src/
├── config.rs      ← parsing tests
├── vpn/models.rs  ← data model tests
└── utils.rs       ← utility function tests
```

### Example

```rust
#[test]
fn test_parse_powershell_config() {
    let ps_script = r#"
    param([string]$ServerAddress = 'vpn.example.com')
    $DHGroup = 'Group14'
    "#;

    let config = parse_powershell_config(ps_script);
    assert!(config.is_ok());
}
```

---

## Integration Tests

### Running

```bash
cargo test --test '*'
```

### Location

```
tests/
└── integration_test.rs
```

### Example

```rust
#[tokio::test]
async fn test_vpn_workflow() {
    let config = parse_config(...).await.unwrap();
    let conn = VpnManager::create_connection(...).await.unwrap();
    assert_eq!(conn.name, "TestVPN");
}
```

---

## Manual Testing Scenarios

### Scenario 1: First Run

**Prerequisites:**
- Fedora with NetworkManager installed
- Test credentials (never use real credentials)

**Steps:**
1. [ ] Launch the app: `cargo run`
2. [ ] App checks prerequisites
3. [ ] App shows the connection list (empty on first run)
4. [ ] Check logs: `RUST_LOG=wikev2connect=debug cargo run`

**Expected:**
- No critical errors
- GUI renders and responds to input

---

### Scenario 2: Import ZIP Configuration

**Setup:**
```bash
mkdir -p /tmp/test-vpn/Windows/ps
echo "param([string]\$ServerAddress = 'vpn.example.com')" \
    > /tmp/test-vpn/Windows/ps/AddVPN.ps1
echo "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----" \
    > /tmp/test-vpn/TestCert.pem
cd /tmp && zip -r test-vpn.zip test-vpn/
```

**Steps:**
1. [ ] Click "Import config..."
2. [ ] Select the ZIP file
3. [ ] App extracts and parses automatically
4. [ ] Review extracted parameters in the form
5. [ ] Allow manual edits

**Expected:**
- All parameters extracted correctly
- No parsing errors
- Parameter preview is accurate

---

### Scenario 3: Create VPN Connection

**Setup:**
```bash
mkdir -p /tmp/test-cert
openssl req -new -x509 -nodes \
    -out /tmp/test-cert/test.pem -keyout /tmp/test-cert/test.key
```

**Steps:**
1. [ ] Form shows all fields
2. [ ] Enter parameters:
   - Name: "TestVPN"
   - Server: "vpn.example.com"
   - Username: "Firebox-DB\\testuser"
   - Certificate: selected
3. [ ] Click Save

**Expected:**
- Certificate copied to `/etc/pki/ca-trust/source/anchors/`
- nmcli creates the connection
- No permission errors
- Connection visible in the list

**Verify:**
```bash
nmcli connection show TestVPN
```

---

### Scenario 4: Edit Connection

**Setup:** create a connection (Scenario 3)

**Steps:**
1. [ ] Click **Edit** on the connection card
2. [ ] Change the username
3. [ ] Click Save

**Expected:**
- Changes saved to NetworkManager
- No forced reconnection

---

### Scenario 5: Delete Connection

**Setup:** create a connection (Scenario 3)

**Steps:**
1. [ ] Click **Delete** on the connection card
2. [ ] Confirm deletion

**Expected:**
- Connection removed from nmcli
- List updated

**Verify:**
```bash
nmcli connection show TestVPN  # should fail
```

---

### Scenario 6: Real VPN Connection (Optional)

⚠️ **Use test credentials only — never real credentials**

**Steps:**
1. [ ] Create a connection (Scenario 3)
2. [ ] Click **Connect**
3. [ ] If MFA is enabled, approve the push notification
4. [ ] Verify connection established
5. [ ] Click **Disconnect**

**Verify:**
```bash
ip addr show      # assigned IP
ip route          # VPN route
```

---

## Test Automation

```bash
# Specific test
cargo test test_parse_powershell_config

# Test a module
cargo test config::

# With stdout output
cargo test -- --nocapture

# Single-threaded
cargo test -- --test-threads=1
```

### Coverage (optional)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

---

## Release Checklist

Before tagging a release:

- [ ] All tests pass: `cargo test`
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt -- --check`
- [ ] Docs build: `cargo doc --no-deps`
- [ ] Manual Scenarios 1–5 completed
- [ ] No compilation warnings
- [ ] `Cargo.toml` version bumped
- [ ] `wikev2connect.spec` changelog updated

---

## Debugging Failing Tests

```bash
# Enable logging in tests
RUST_LOG=wikev2connect=debug cargo test -- --nocapture

# Full backtrace
RUST_BACKTRACE=full cargo test
```

### "Permission denied" in tests

Some tests require elevated privileges (certificate install):

```bash
cargo test --lib     # unit tests only (no sudo required)
```

---

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
