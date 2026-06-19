# WatchGuard IKEv2 VPN on Fedora Linux — Manual Setup Guide

Guide for configuring IKEv2 client connections on Fedora with a WatchGuard Firebox appliance.
Authentication via username/password (EAP-MSCHAPv2) + CA certificate.

> This is a reference for manual setup via the command line. WIKEv2 Connect automates
> all of these steps through its GUI.

---

## One-Time System Setup

These steps are performed **once** per machine.

### 1. Install Required Packages

```bash
rpm -q NetworkManager-strongswan strongswan
```

If not installed:

```bash
sudo dnf install NetworkManager-strongswan strongswan
```

### 2. System Crypto Policy

WatchGuard uses RSA signatures in the pre-RFC7427 format. Fedora's DEFAULT policy blocks
SHA1 and the old format. Apply once:

```bash
sudo update-crypto-policies --set DEFAULT:SHA1
```

### 3. Configure strongSwan (charon-nm.conf)

Add the following two lines inside the `charon-nm { }` block in
`/etc/strongswan/strongswan.d/charon-nm.conf`, right after `load_modular = yes`:

```bash
sudo sed -i 's/load_modular = yes/load_modular = yes\n    signature_authentication_constraints = no\n    signature_authentication = no/' \
    /etc/strongswan/strongswan.d/charon-nm.conf
```

Verify:

```bash
grep -A3 "load_modular" /etc/strongswan/strongswan.d/charon-nm.conf
```

Expected output:
```
load_modular = yes
signature_authentication_constraints = no
signature_authentication = no
```

**Why:** WatchGuard does not implement RFC 7427 (IKEv2 Signature Authentication). strongSwan 6.x
requires it by default; disabling it makes strongSwan accept the older RSA format.

---

## Per-Connection Setup

The client provides a ZIP archive with WatchGuard configuration files, typically containing:

```
ClientName.pem        ← CA certificate (PEM format)
ClientName.crt        ← same certificate in DER format
Windows/
  ClientName.bat      ← Windows script (contains server IP and connection name)
  ps/AddVPN.ps1       ← IKE/ESP parameters
Android/
  ClientName.sswan    ← strongSwan profile (contains the embedded CA cert)
```

### Step 1 — Extract Parameters from PS1

From `Windows/ps/AddVPN.ps1`:

| PowerShell parameter | Linux value |
|---|---|
| `-ServerAddress` | VPN server address |
| `-Name` | connection name |
| `-DnsSuffix` | DNS search domain (optional) |
| `DHGroup` | Group14 → `modp2048`, Group19 → `ecp256`, Group20 → `ecp384` |
| `EncryptionMethod` | AES256 → `aes256`, AES128 → `aes128` |
| `IntegrityCheckMethod` | SHA256 → `sha256`, SHA384 → `sha384` |
| `CipherTransformConstants` | AES256 → `aes256`, AES128 → `aes128` |
| `AuthenticationTransformConstants` | SHA196 → `sha1`, SHA256128 → `sha256` |

**Example:**
```powershell
-ServerAddress 'vpn.example.com'
-Name 'MyVPN'
-DnsSuffix 'example.local'
DHGroup = 'Group14'                          → modp2048
EncryptionMethod = 'AES256'                  → aes256
IntegrityCheckMethod = 'SHA256'              → sha256   (IKE integrity)
CipherTransformConstants = 'AES256'          → aes256   (ESP cipher)
AuthenticationTransformConstants = 'SHA196'  → sha1     (ESP auth)
```

Result: `ike=aes256-sha256-modp2048` and `esp=aes256-sha1`

### Step 2 — Install the CA Certificate

```bash
sudo cp /path/to/ClientName.pem /etc/pki/ca-trust/source/anchors/ClientName-WatchGuard.pem
sudo update-ca-trust
```

> **SELinux note:** the certificate must be in `/etc/pki/ca-trust/source/anchors/`
> to receive the `cert_t` SELinux context. `charon-nm` cannot read files in `/tmp/`
> (which have `user_tmp_t`).

### Step 3 — Create the VPN Connection

```bash
nmcli connection add \
  type vpn \
  con-name "ConnectionName" \
  vpn-type org.freedesktop.NetworkManager.strongswan \
  vpn.data "address=VPN_SERVER, method=eap, user=DOMAIN\\USER, \
            certificate=/etc/pki/ca-trust/source/anchors/ClientName-WatchGuard.pem, \
            virtual=yes, encap=no, ipcomp=no, proposal=yes, \
            ike=IKE_PROPOSAL, esp=ESP_PROPOSAL" \
  vpn.secrets "password=PASSWORD"
```

**Example:**
```bash
nmcli connection add \
  type vpn \
  con-name "MyVPN" \
  vpn-type org.freedesktop.NetworkManager.strongswan \
  vpn.data "address=vpn.example.com, method=eap, user=Firebox-DB\\mario.rossi, \
            certificate=/etc/pki/ca-trust/source/anchors/MyVPN-WatchGuard.pem, \
            virtual=yes, encap=no, ipcomp=no, proposal=yes, \
            ike=aes256-sha256-modp2048, esp=aes256-sha1" \
  vpn.secrets "password=yourpassword"
```

> **Username format:** WatchGuard uses `DOMAIN\User` (e.g. `Firebox-DB\mario.rossi`).
> Use double backslash `\\` in bash. In fish shell use single quotes: `'Firebox-DB\mario.rossi'`.

### Step 4 — First Connection

After adding or modifying `charon-nm.conf`, restart the `charon-nm` process once:

```bash
sudo kill $(pgrep charon-nm)   # NM will restart it automatically on connect
nmcli connection up ConnectionName
```

For subsequent connections:

```bash
nmcli connection up ConnectionName
```

---

## Modifying an Existing Connection

```bash
# Update connection data
nmcli connection modify "ConnectionName" vpn.data "..."

# Update password only
nmcli connection modify "ConnectionName" vpn.secrets "password=NEW_PASSWORD"
```

## Deleting a Connection

```bash
nmcli connection delete "ConnectionName"
```

## List All VPN Connections

```bash
nmcli connection show | grep vpn
```

---

## Troubleshooting

### `Invalid ESP proposal`

The ESP algorithm name is wrong. Use `sha1` (not `sha1_96`), `sha256` (not `sha2_256`).

### `signature validation failed`

- Verify that `charon-nm.conf` contains `signature_authentication = no`
  and `signature_authentication_constraints = no`
- Restart charon-nm: `sudo kill $(pgrep charon-nm)`

### `VPN service failed to start`

Check the IKE/ESP proposal syntax:

```bash
journalctl -u NetworkManager -n 30 | grep charon
```

### Connection drops or is slow behind NAT

Force UDP encapsulation:

```bash
nmcli connection modify "ConnectionName" vpn.data "..., encap=yes, ..."
```

---

## Common WatchGuard Algorithm Combinations

| Firmware configuration | IKE proposal | ESP proposal |
|---|---|---|
| Typical (AES256 + SHA256 + DH14) | `aes256-sha256-modp2048` | `aes256-sha1` |
| AES128 variant | `aes128-sha256-modp2048` | `aes128-sha1` |
| With PFS | `aes256-sha256-modp2048` | `aes256-sha1-modp2048` |
