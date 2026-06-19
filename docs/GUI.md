# WIKEv2 Connect — GUI Reference

## Overview

The GUI is built with **egui / eframe** (immediate-mode, OpenGL via `glow`).
It runs natively on Wayland and on X11 (via XWayland) under KDE Plasma.

The colour palette is read at startup from `~/.config/kdeglobals`, so the app
automatically follows the active KDE colour scheme.

---

## Main Window

```
┌──────────────────────────────────────────────────────┐
│  WIKEv2 Connect — VPN Manager                        │
├──────────────────────────────────────────────────────┤
│  [Import config...]  [New connection]  [🔄 Refresh]  │
│  🔍 Search...                                        │
├──────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  MyVPN                    ● CONNECTED        │   │
│  │  vpn.example.com · mario.rossi               │   │
│  │  [Disconnect]  [Edit]  [Delete]              │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  OfficeVPN                ○ DISCONNECTED     │   │
│  │  vpn2.example.com · john.doe                 │   │
│  │  [Connect]  [Edit]  [Delete]                 │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### Toolbar

| Button | Action |
|---|---|
| **Import config...** | Opens a file dialog (XDG portal); accepts `.zip` or `.ps1` |
| **New connection** | Opens an empty connection form |
| **Refresh** | Forces an immediate status poll from NetworkManager |

### Search Field

Real-time filter: type to narrow the list by connection name, server address or username.
The filter is case-insensitive and matches substrings.

---

## Connection Cards

Each VPN connection is shown as a card with:

- **Coloured status dot** — painted directly with egui `Painter`
- **Status badge** — `CONNECTED` / `DISCONNECTED` / `CONNECTING` / `ERROR`
- **Name** (large), **server** and **username** (small)
- **Action buttons** — `Connect` / `Disconnect`, `Edit`, `Delete`

### Status Colours

| Status | Colour |
|---|---|
| Connected | Green |
| Disconnected | Grey |
| Connecting | Yellow / amber |
| Error | Red |

### MFA Banner

When a connection is in `Connecting` state, a yellow banner appears below the card:

```
⚠  Waiting for AuthPoint push notification.
   Approve the push on your device, then wait up to 90 seconds.
```

---

## Import Configuration Flow

```
Click "Import config..."
  ↓
File dialog (XDG portal — native KDE)
  ↓
ZIP selected                    PS1 selected
  ↓                               ↓
Extract to temp dir          Read file directly
Find PS1 + certificate
  ↓
Parse PowerShell parameters
  ↓
┌─────────────────────────────────┐
│  Extracted parameters:          │
│  Name:    MyVPN                 │
│  Server:  vpn.example.com       │
│  IKE:     aes256-sha256-modp2048│
│  ESP:     aes256-sha1           │
└─────────────────────────────────┘
  ↓
Connection form (pre-filled)
  ↓
User reviews and saves
  ↓
Certificate installed (ksshaskpass)
nmcli creates connection
Card appears in list
```

---

## Connection Form

Used for both creating and editing connections.

```
┌─────────────────────────────────────────────────┐
│  New Connection                                  │
├─────────────────────────────────────────────────┤
│  Name:          [________________________]      │
│  Server:        [________________________]      │
│  Username:      [________________________]      │
│  Password:      [________________________]      │
│  Certificate:   [Browse...] path/to/cert.pem    │
│                                                  │
│  IKE proposal:  [aes256-sha256-modp2048______]  │
│  ESP proposal:  [aes256-sha1_________________]  │
│                                                  │
│  [ ] UDP encapsulation                          │
│  [ ] IP compression                             │
│  DNS suffix:    [________________________]      │
│                                                  │
│  [Save]                          [Cancel]       │
└─────────────────────────────────────────────────┘
```

### Fields

| Field | Description |
|---|---|
| Name | Connection name (used as `nmcli` connection ID) |
| Server | VPN server address or IP |
| Username | Format `DOMAIN\user` (e.g. `Firebox-DB\mario.rossi`) |
| Password | EAP-MSCHAPv2 password |
| Certificate | Path to the CA certificate (PEM or DER) |
| IKE proposal | strongSwan IKE algorithm string |
| ESP proposal | strongSwan ESP algorithm string |
| UDP encapsulation | Useful behind NAT (`encap=yes`) |
| IP compression | Enable IPComp |
| DNS suffix | Optional DNS search domain |

---

## Delete Confirmation Dialog

```
┌──────────────────────────────────────────┐
│  Delete "MyVPN"?                         │
│  This will remove the connection from    │
│  NetworkManager. The certificate file    │
│  in /etc/pki/... will not be removed.   │
│                                          │
│  [Delete]                   [Cancel]    │
└──────────────────────────────────────────┘
```

---

## Keyboard Navigation

The app responds to standard keyboard input in text fields. There is no global
keyboard shortcut scheme at this time.

---

## Wayland Notes

- The Wayland `app_id` is `wikev2connect` (matches the `.desktop` filename)
- `with_active(true)` and `ViewportCommand::Focus` are used on first frame to prevent the "first click ignored" issue caused by KWin consuming the activation event
- The file dialog uses the XDG Desktop Portal for native Wayland integration

---

## Theme

`src/gui/theme.rs` reads KDE colour values from `~/.config/kdeglobals`:

- `Colors:Window` → panel background
- `Colors:Button` → button background
- `Colors:Selection` → accent / highlight colour
- Text colours, border colours

If the file is not found or a value is missing, egui default colours are used.
