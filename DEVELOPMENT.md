# WIKEv2 Connect — Development Guide

## Architecture

### Tech Stack

| Layer | Technology |
|---|---|
| GUI | [egui](https://github.com/emilk/egui) 0.28 + [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) |
| Rendering | OpenGL via `glow` (native Wayland, X11 via XWayland) |
| Async runtime | [tokio](https://tokio.rs) multi-thread |
| VPN backend | `nmcli` (NetworkManager CLI) + strongSwan plugin |
| Credentials | KWallet via `secret-service` / NetworkManager |
| File dialog | [rfd](https://github.com/PolyMeilex/rfd) (XDG portal, native KDE) |

### Module Structure

```
src/
├── main.rs              # Entry point: initialises tokio, calls gui::launch()
├── errors.rs            # WIKEv2ConnectError, Result<T> alias
├── config.rs            # WatchGuard PowerShell parsing (regex on PS1 text)
├── cert.rs              # CA cert install via ksshaskpass + update-ca-trust
├── storage.rs           # Credentials (KWallet / NetworkManager)
├── utils.rs             # ZIP extraction, file reading with encoding detection
├── gui/
│   ├── mod.rs           # launch(): ViewportBuilder, run_native("wikev2connect", ...)
│   ├── app.rs           # Wikev2App: state, event loop, card rendering
│   ├── theme.rs         # kdeglobals reader, Palette, apply() on egui Visuals
│   └── operations.rs    # GUI → VPN bridge (parse_config_file, create/update/delete/connect)
├── vpn/
│   ├── mod.rs           # VpnManager: CRUD + connect/disconnect via NM
│   ├── nmcli.rs         # nmcli commands (output parsing)
│   └── models.rs        # VpnConnection, ConnectionStatus, VpnProposal
└── system/
    ├── mod.rs
    └── prerequisites.rs # System prerequisite check and install
```

### GUI Event Loop

egui is immediate-mode: every frame redraws everything. The app uses a producer/consumer pattern:

```
tokio::spawn(async {
    // VPN operation (nmcli, NM...)
    tx.send(BgEvent::Done { ... })
}) ─────────────────────────────► mpsc::sync_channel
                                        │
                                        ▼
                               app.poll() every frame
                               updates self.connections
                               ctx.request_repaint()
```

`BgEvent` has three variants:
- `Connections(Vec<VpnConnection>)` — periodic refresh (every 5 s)
- `Error(String)` — operation failed
- `Done { msg, conn_name }` — operation completed (connect/disconnect/save)

### Optimistic UI State

When the user clicks **Connect**, the status is updated immediately in the UI
without waiting for NM to confirm. To prevent the periodic refresh from overwriting
this in-flight state, a `pending: HashSet<String>` is used:

```rust
// on click
self.connections[i].status = ConnectionStatus::Connecting;
self.pending.insert(conn_name.clone());

// in poll(), when BgEvent::Connections arrives
if self.pending.contains(&conn.name) {
    // don't overwrite the optimistic state
} else {
    conn.status = fresh_status;
}

// when BgEvent::Done { conn_name: Some(name) } arrives
self.pending.remove(&name);
```

### Wayland: First Click Ignored

KWin on Wayland consumes the first input event to activate the window.
Fix applied in `src/gui/mod.rs`:

```rust
.with_active(true)  // ViewportBuilder
```

And in `src/gui/app.rs` on the first frame:

```rust
if self.need_focus {
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    self.need_focus = false;
}
```

### Wayland app_id

The first argument to `run_native` becomes the Wayland `app_id`.
It must match **exactly** the `.desktop` filename (without extension)
so KDE can associate the titlebar icon with the window:

```rust
eframe::run_native("wikev2connect", ...)  // ← must match wikev2connect.desktop
```

### SELinux and Certificates

strongSwan (`charon-nm`) runs under the `charon_t` SELinux context and cannot
read files in `/tmp/` (which have `user_tmp_t` context). CA certificates must be
installed in `/etc/pki/ca-trust/source/anchors/` (which gets `cert_t` context).

Installation is done via `ksshaskpass` (KDE GUI sudo):

```bash
sudo --askpass cp cert.pem /etc/pki/ca-trust/source/anchors/
sudo --askpass update-ca-trust
```

Falls back to plain `sudo` if `ksshaskpass` is not available.

---

## Local Development

```bash
# Clone
git clone https://github.com/YOUR_USERNAME/wikev2connect.git
cd wikev2connect

# Debug with logging
cargo run
RUST_LOG=wikev2connect=debug cargo run

# Check and test
cargo check
cargo test
cargo clippy
cargo fmt
```

### Build RPM

```bash
make rpm
# Package is created in ~/rpmbuild/RPMS/x86_64/
```

### System Build Dependencies

```bash
sudo dnf install gcc-c++ \
    libX11-devel libXcursor-devel libXi-devel libXrandr-devel \
    wayland-devel libxkbcommon-devel \
    mesa-libGL-devel mesa-libEGL-devel \
    zlib-devel openssl-devel dbus-devel
```

---

## Known Extension Points

- **Tray icon**: eframe does not natively support a system tray; would require
  integration with `libappindicator` or D-Bus `StatusNotifierItem`.
- **Desktop notifications**: `notify-rust` can be used to send KDE notifications
  when a connection completes.
- **VPN statistics**: `nmcli` exposes RX/TX bytes for active connections;
  these could be displayed in the connection card.
- **Automatic reconnection**: monitor connection state and reconnect if it drops
  unexpectedly.
