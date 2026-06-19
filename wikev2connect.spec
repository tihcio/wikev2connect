Name:           wikev2connect
Version:        0.1.0
Release:        1%{?dist}
Summary:        Gestione VPN IKEv2 WatchGuard per Fedora Linux
License:        MIT
URL:            https://github.com/YOUR_USERNAME/wikev2connect
Source0:        %{name}-%{version}.tar.gz

# ── Build dependencies ────────────────────────────────────────────────────────
# Rust toolchain: usa i pacchetti sistema oppure rustup.
# Con rustup usare: rpmbuild -ba --nodeps
BuildRequires:  gcc-c++

# egui/eframe con backend glow (OpenGL) — X11 e Wayland nativi via winit
BuildRequires:  libX11-devel
BuildRequires:  libXcursor-devel
BuildRequires:  libXi-devel
BuildRequires:  libXrandr-devel
BuildRequires:  wayland-devel
BuildRequires:  libxkbcommon-devel

# OpenGL / EGL per il backend glow
BuildRequires:  mesa-libGL-devel
BuildRequires:  mesa-libEGL-devel

# Compressione e TLS
BuildRequires:  zlib-devel
BuildRequires:  openssl-devel

# dbus (usato da secret-service per kwallet)
BuildRequires:  dbus-devel

# ── Runtime dependencies ──────────────────────────────────────────────────────
# VPN backend
Requires:       NetworkManager
Requires:       NetworkManager-strongswan
Requires:       strongswan
Requires:       openssl

# X11/Wayland runtime (egui/eframe con glow — nativi Wayland, X11 via XWayland)
Requires:       libX11
Requires:       libXcursor
Requires:       libXi
Requires:       libXrandr
Requires:       libxkbcommon
Requires:       mesa-libGL
Requires:       mesa-libEGL

%description
WIKEv2 Connect è un'applicazione per gestire connessioni VPN IKEv2 verso
dispositivi WatchGuard Firebox su Fedora Linux.

Funzionalità:
  - Importazione configurazioni ZIP/PS1 da WatchGuard
  - Estrazione automatica dei parametri IKE/ESP e certificati CA
  - Installazione automatica certificati in /etc/pki/trust/anchors/
  - Creazione, modifica ed eliminazione connessioni via NetworkManager
  - Connetti/Disconnetti con stato in tempo reale e feedback immediato
  - Supporto MFA/AuthPoint con hint visivo durante l'autenticazione
  - Filtro di ricerca in tempo reale sulla lista connessioni
  - Tema colori adattivo letto da ~/.config/kdeglobals (KDE Plasma)

GUI: egui/eframe con rendering OpenGL (glow) e supporto nativo Wayland
tramite winit. Compatibile con sessioni X11 e Wayland KDE Plasma.

%prep
%setup -q

%build
export CARGO_HOME=%{_builddir}/cargo-home
cargo build --release

%install
install -Dm755 target/release/wikev2connect \
    %{buildroot}%{_bindir}/wikev2connect

install -Dm644 resources/wikev2connect.desktop \
    %{buildroot}%{_datadir}/applications/wikev2connect.desktop

install -Dm644 resources/icona.png \
    %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/wikev2connect.png

install -Dm644 resources/wikev2connect.metainfo.xml \
    %{buildroot}%{_datadir}/metainfo/wikev2connect.metainfo.xml

%files
%license LICENSE
%doc README.md
%{_bindir}/wikev2connect
%{_datadir}/applications/wikev2connect.desktop
%{_datadir}/icons/hicolor/256x256/apps/wikev2connect.png
%{_datadir}/metainfo/wikev2connect.metainfo.xml

%changelog
* Thu Jun 19 2025 Maintainer <your@email.com> - 0.1.0-1
- Prima release pubblica con nome wikev2connect
- GUI egui/eframe, tema KDE adattivo
- Gestione VPN IKEv2 WatchGuard tramite NetworkManager/strongSwan
- Importazione ZIP/PS1 con parsing PowerShell automatico
- Installazione certificati CA via PolicyKit (ksshaskpass)
- Feedback immediato su connect/disconnect con stato ottimistico
- Supporto MFA AuthPoint con hint visivo
- Ricerca in tempo reale
- Fix Wayland "primo click ignorato"
