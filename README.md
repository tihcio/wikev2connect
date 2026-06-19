# WIKEv2 Connect

WatchGuard IKEv2 VPN manager for Fedora Linux with a native GUI.

Lets you import, create, edit and manage IKEv2 VPN connections to WatchGuard Firebox appliances,
integrating with NetworkManager and the strongSwan plugin.

---

## Features

- **Configuration import** — load `.zip` or `.ps1` files exported from WatchGuard;
  automatically extracts IKE/ESP parameters, server address and CA certificate
- **Certificate installation** — copies the CA certificate to `/etc/pki/ca-trust/source/anchors/`
  via the KDE graphical sudo dialog (ksshaskpass), no terminal required
- **Connection management** — create, edit and delete connections via NetworkManager
- **Connect / Disconnect** — immediate visual feedback; status updated in real time
- **MFA AuthPoint support** — visual hint while waiting for a WatchGuard AuthPoint push notification
- **Adaptive KDE theme** — colours read from `~/.config/kdeglobals` at runtime
- **Search** — real-time filter by name, server and username

## Screenshots

> *(to be added once the project reaches a stable release)*

## System Requirements

| Component | Fedora package |
|---|---|
| NetworkManager | `NetworkManager` |
| strongSwan NM plugin | `NetworkManager-strongswan` |
| strongSwan | `strongswan` |
| ksshaskpass (GUI sudo) | `ksshaskpass` |

```bash
sudo dnf install NetworkManager-strongswan strongswan ksshaskpass
```

## Building

Requires Rust 1.75+ (recommended via [rustup](https://rustup.rs)).

```bash
git clone https://github.com/YOUR_USERNAME/wikev2connect.git
cd wikev2connect
cargo build --release
```

The compiled binary is at `target/release/wikev2connect`.

### System build dependencies

```bash
sudo dnf install gcc-c++ \
    libX11-devel libXcursor-devel libXi-devel libXrandr-devel \
    wayland-devel libxkbcommon-devel \
    mesa-libGL-devel mesa-libEGL-devel \
    zlib-devel openssl-devel dbus-devel
```

## Development

```bash
# Run in debug mode with logging
cargo run
RUST_LOG=wikev2connect=debug cargo run

# Check compilation
cargo check

# Linter
cargo clippy
```

## RPM Installation (Fedora)

```bash
make rpm
sudo dnf install ~/rpmbuild/RPMS/x86_64/wikev2connect-*.rpm
```

## Usage

On startup the app shows all VPN connections configured in NetworkManager.

### Importing a WatchGuard configuration

1. Click **Import config...** in the toolbar
2. Select the `.zip` file (contains PS1 + certificate) or a `.ps1` file directly
3. Parameters are extracted automatically
4. Review the data in the form and click **Save**
5. If a CA certificate is present, it is installed automatically (requires your admin password)

### Connecting

1. Click **Connect** on the connection card
2. If the VPN uses MFA (AuthPoint), approve the push notification on your device
3. The status will update to **CONNECTED** once the handshake completes

### Notes on MFA AuthPoint

After clicking Connect, the WatchGuard server sends a push notification to the device
registered in AuthPoint. The connection completes within ~90 seconds of approval.
The app shows a yellow banner with instructions while waiting.

## Project Structure

```
src/
├── main.rs                  # Entry point, tokio runtime
├── errors.rs                # Error types
├── config.rs                # WatchGuard PowerShell config parsing
├── cert.rs                  # CA certificate installation
├── storage.rs               # Credentials (KWallet / NetworkManager)
├── utils.rs                 # ZIP and file utilities
├── gui/
│   ├── mod.rs               # eframe window launch
│   ├── app.rs               # App state, event loop, card rendering
│   ├── theme.rs             # kdeglobals reader, colour palette
│   └── operations.rs        # GUI → VPN backend bridge
├── vpn/
│   ├── mod.rs               # VpnManager (create/modify/delete/connect)
│   ├── nmcli.rs             # nmcli commands
│   └── models.rs            # VpnConnection, ConnectionStatus, VpnProposal
└── system/
    ├── mod.rs
    └── prerequisites.rs     # System prerequisite check and install
```

## Tech Stack

- **[Rust](https://rust-lang.org)** — primary language
- **[egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)** — immediate-mode GUI, OpenGL rendering (glow), native Wayland
- **[NetworkManager](https://networkmanager.dev)** — connection management via `nmcli`
- **[strongSwan](https://strongswan.org)** — IKEv2 backend (`charon-nm` plugin)
- **[rfd](https://github.com/PolyMeilex/rfd)** — native KDE file dialog (XDG portal)
- **[tokio](https://tokio.rs)** — async runtime for network operations

## License

[MIT](LICENSE)
