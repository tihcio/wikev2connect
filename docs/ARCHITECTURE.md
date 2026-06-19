# WIKEv2 Connect — Architecture

## Overview

```
┌─────────────────────────────────────────────────────────┐
│                    GUI Layer (egui/eframe)               │
│  ┌──────────────────────────────────────────────────┐   │
│  │ Wikev2App | theme | operations                   │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
             ↓
┌─────────────────────────────────────────────────────────┐
│                  Business Logic Layer                   │
│  ┌──────────────────────────────────────────────────┐   │
│  │ VPN Manager | Config Parser | Cert Manager       │   │
│  │ System Prerequisites | Password Storage           │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
             ↓
┌─────────────────────────────────────────────────────────┐
│                  System Integration Layer               │
│  ┌──────────────────────────────────────────────────┐   │
│  │ nmcli (NetworkManager) | System Commands         │   │
│  │ File System | Certificates | D-Bus / KWallet     │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
             ↓
┌─────────────────────────────────────────────────────────┐
│                     Fedora System                       │
│  NetworkManager | strongSwan | Certificates | KDE      │
└─────────────────────────────────────────────────────────┘
```

## Main Modules

### 1. GUI Module (`src/gui/`)

**Responsibilities:**
- Rendering the user interface
- Handling user interactions
- Updating UI state

**Components:**
- `mod.rs`: Window launch, ViewportBuilder, Wayland app_id
- `app.rs`: `Wikev2App` — main state struct, event loop, card rendering
- `theme.rs`: reads `~/.config/kdeglobals`, builds `Palette`, applies egui `Visuals`
- `operations.rs`: bridge between GUI events and VPN backend calls

**Pattern:**
- egui immediate-mode: full redraw every frame
- Background tasks communicate via `mpsc::sync_channel` (`BgEvent`)
- Optimistic UI updates with a `pending: HashSet<String>` guard

### 2. VPN Module (`src/vpn/`)

**Responsibilities:**
- Managing VPN connections
- Communication with NetworkManager
- VPN data modelling

**Components:**
- `nmcli.rs`: `nmcli` command wrappers (output parsing)
- `models.rs`: `VpnConnection`, `VpnProposal`, `ConnectionStatus`

**Main API:**
```rust
pub struct VpnManager;

impl VpnManager {
    pub async fn list_connections() -> Result<Vec<VpnConnection>>
    pub async fn create_connection(...) -> Result<VpnConnection>
    pub async fn modify_connection(...) -> Result<()>
    pub async fn delete_connection(...) -> Result<()>
    pub async fn connect(...) -> Result<()>
    pub async fn disconnect(...) -> Result<()>
}
```

**Connection Creation Flow:**
```
create_connection()
    ↓
nmcli connection add type=vpn vpn-type=org.freedesktop.NetworkManager.strongswan
    ↓
Store in NetworkManager database
    ↓
Return VpnConnection struct
```

### 3. Config Module (`src/config.rs`)

**Responsibilities:**
- Parsing WatchGuard configuration files
- Extracting IKE/ESP parameters
- Data validation

**Main Functions:**
```rust
pub async fn parse_powershell_config(content: &str) -> Result<VpnConfig>
pub async fn parse_config_from_file<P: AsRef<Path>>(path: P) -> Result<VpnConfig>
```

**Supported Formats:**
- PowerShell (.ps1) ✓
- ZIP archive containing PS1 + certificate ✓

### 4. Certificate Module (`src/cert.rs`)

**Responsibilities:**
- Installing CA certificates
- Updating the CA trust database
- Certificate lifecycle (install, verify)

**Main Functions:**
```rust
pub async fn install_certificate<P: AsRef<Path>>(cert_path: P, cert_name: &str) -> Result<PathBuf>
pub fn get_installed_cert_path(cert_name: &str) -> PathBuf
```

**Certificate Paths:**
- Installed to: `/etc/pki/ca-trust/source/anchors/`
- Naming convention: `{ClientName}-WatchGuard.pem`
- SELinux context: `cert_t` (required by `charon-nm`)

### 5. System Module (`src/system/`)

**Responsibilities:**
- Checking system prerequisites
- Installing missing packages
- Configuring strongSwan

**Checked Prerequisites:**
- NetworkManager ✓
- NetworkManager-strongswan ✓
- strongSwan ✓
- openssl ✓
- Crypto Policy (SHA1 enabled) ✓
- charon-nm daemon ✓

### 6. Storage Module (`src/storage.rs`)

**Responsibilities:**
- Secure password storage
- KWallet integration (KDE keyring)
- NetworkManager fallback

**Supported Backends:**
```rust
pub enum StorageBackend {
    KWallet,  // recommended — encrypted, user-approved access
    NmCli,    // NetworkManager — restricted file permissions, not encrypted
}
```

### 7. Utils Module (`src/utils.rs`)

**Responsibilities:**
- ZIP extraction
- File reading with encoding detection (UTF-16 LE/BE, UTF-8 BOM, Latin-1)
- Certificate and filename utilities

## Flow Diagrams

### Application Startup

```
main()
  ↓
Build tokio multi-thread runtime
rt.enter()  ← makes tokio::spawn() work inside eframe
  ↓
gui::launch()
  ├─ Load icon (embedded PNG)
  ├─ Build ViewportBuilder (.with_active(true) for Wayland)
  ├─ run_native("wikev2connect", ...)
  └─ Wikev2App::new(cc)
       ├─ Spawn background refresh loop (every 5 s)
       └─ Request first-frame focus (Wayland fix)
```

### Import ZIP and Create Connection

```
User clicks "Import config..."
  ↓
rfd::FileDialog (XDG portal)
  ↓
operations::parse_config_file(path)
  ├─ Detect ZIP or PS1
  ├─ Extract ZIP to temp dir
  ├─ Find PS1 + certificate files
  └─ config::parse_powershell_config()
       ├─ Extract ServerAddress, Name, DnsSuffix
       ├─ Extract IKE parameters
       └─ Extract ESP parameters
  ↓
Show edit form pre-filled with parsed data
  ↓
User confirms → operations::create_connection()
  ├─ cert::install_certificate()  (ksshaskpass)
  └─ VpnManager::create_connection()  (nmcli)
  ↓
BgEvent::Done → UI adds card to list
```

### Connect to VPN

```
User clicks "Connect"
  ↓
Optimistic update: status → Connecting
pending.insert(conn_name)
  ↓
tokio::spawn → VpnManager::connect(name)
  ↓
nmcli connection up {name}
  ├─ charon-nm: IKE Phase 1 (key exchange)
  ├─ IKE Phase 2 (SA negotiation)
  ├─ IPSec tunnel setup
  └─ IP address assignment
  ↓
BgEvent::Done { conn_name }
  ├─ pending.remove(conn_name)
  └─ UI shows final status from next refresh
```

## Data Models

### VpnConnection

```rust
pub struct VpnConnection {
    pub name: String,                    // e.g. "MyVPN"
    pub server_address: String,          // e.g. "vpn.example.com"
    pub username: String,                // e.g. "Firebox-DB\mario.rossi"
    pub certificate_path: String,        // /etc/pki/ca-trust/source/anchors/...
    pub ike_proposal: String,            // e.g. "aes256-sha256-modp2048"
    pub esp_proposal: String,            // e.g. "aes256-sha1"
    pub status: ConnectionStatus,        // Connected / Disconnected / Connecting / Error
    pub dns_suffix: Option<String>,      // e.g. "example.local"
    pub encap: bool,                     // UDP encapsulation
    pub ipcomp: bool,                    // IP compression
}
```

### VpnConfig (from PowerShell parsing)

```rust
pub struct VpnConfig {
    pub name: String,                    // "MyVPN"
    pub server_address: String,          // "vpn.example.com"
    pub dh_group: String,                // "Group14" → "modp2048"
    pub encryption_method: String,       // "AES256"
    pub integrity_check: String,         // "SHA256"
    pub cipher_transform: String,        // "AES256"
    pub auth_transform: String,          // "SHA196" → "sha1"
}
```

### VpnProposal

```rust
pub struct VpnProposal {
    pub ike: String,                     // "aes256-sha256-modp2048"
    pub esp: String,                     // "aes256-sha1"
}
```

## Error Handling

### Error Hierarchy

```
WIKEv2ConnectError
├─ VpnError(String)            → VPN connection issues
├─ ConfigError(String)         → configuration parsing failed
├─ CertError(String)           → certificate management
├─ SystemError(String)         → system commands
├─ StorageError(String)        → password storage
├─ FileError(io::Error)        → file operations
├─ ZipError(ZipError)          → ZIP extraction
├─ CommandError(String)        → command execution failed
└─ MissingPrerequisite(String) → missing packages / configuration
```

All errors are logged via `tracing` / `log`; set `RUST_LOG=wikev2connect=debug` for verbose output.

## Async Architecture

```
                    ┌─ File Operations
                    ├─ Command Execution (nmcli, sudo)
Tokio Runtime ──────┤─ Certificate Installation
                    ├─ Connection Status Polling
                    └─ Background Refresh Loop
```

**Thread model:**
- eframe runs on the main thread (OpenGL context)
- Background tasks run on the tokio thread pool via `tokio::spawn`
- Communication: `mpsc::sync_channel` (bounded, non-blocking send)
- `ctx.request_repaint()` wakes the eframe loop after each background event

## Security Notes

### Password Storage

- **KWallet** (recommended): encrypted on disk, user approves access via D-Bus
- **NetworkManager**: stored in `/etc/NetworkManager/system-connections/` with root-only file permissions; not encrypted

### Certificate Installation

Certificates must land in `/etc/pki/ca-trust/source/anchors/` to get the `cert_t`
SELinux context readable by `charon-nm`. Files in `/tmp/` have `user_tmp_t` and are
denied. Installation uses `ksshaskpass` for graphical privilege escalation.

### sudo Usage

Operations requiring elevated privileges:
- Certificate installation and `update-ca-trust`
- strongSwan configuration changes
- Crypto policy updates
