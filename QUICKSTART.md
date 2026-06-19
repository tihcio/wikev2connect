# WIKEv2 Connect — Quick Start

## Quick Install (Fedora)

```bash
# Prerequisites
sudo dnf install NetworkManager-strongswan strongswan ksshaskpass

# Clone and build
git clone https://github.com/YOUR_USERNAME/wikev2connect.git
cd wikev2connect
cargo build --release

# Or build an RPM package
make rpm
sudo dnf install ~/rpmbuild/RPMS/x86_64/wikev2connect-*.rpm
```

## Running

```bash
# Run directly
./target/release/wikev2connect

# In development (with logging)
cargo run
RUST_LOG=wikev2connect=debug cargo run
```

## Importing a WatchGuard VPN Configuration

1. Launch the app
2. Click **Import config...** in the toolbar
3. Select the `.zip` or `.ps1` file exported from WatchGuard
4. Review the parameters in the form (name, server, username)
5. Click **Save** — the CA certificate is installed automatically

## Connecting

- Click **Connect** on a connection card
- If the server uses MFA (AuthPoint), approve the push notification on your mobile device
- The status changes to **CONNECTED** when the handshake completes (~90 seconds maximum)

## Useful Commands During Development

| Command | Description |
|---|---|
| `cargo check` | Check compilation |
| `cargo run` | Run in debug mode |
| `cargo test` | Run tests |
| `cargo clippy` | Linter |
| `cargo fmt` | Format code |
| `make rpm` | Build RPM package |

## Debug

```bash
# App logs
RUST_LOG=wikev2connect=debug cargo run

# NetworkManager / strongSwan logs
journalctl -u NetworkManager -f | grep charon
journalctl -xe NM_CONNECTION=<uuid>
```

## Common Issues

**VPN fails to start**: make sure `NetworkManager-strongswan` and `strongswan` are installed.

**Certificate not installed**: make sure `ksshaskpass` is installed and your user has sudo privileges.

**Status not updating**: the status is refreshed every 5 seconds automatically. Use the **Refresh** button for an immediate update.
