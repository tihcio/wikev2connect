use egui::{
    Align, Button, Color32, Frame, Id, Label, Layout, Margin,
    RichText, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui, Vec2,
};
use crate::vpn::models::{VpnConnection, ConnectionStatus};
use crate::gui::theme::{self, Palette, darken};
use crate::gui::operations as ops;
use log::{error, info};
use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Duration;

// ── Background messages ───────────────────────────────────────────────────────

enum BgEvent {
    Connections(Vec<VpnConnection>),
    Error(String),
    // conn_name: connessione coinvolta, usata per liberare il pending set
    Done { msg: String, conn_name: Option<String> },
}

// ── Form state ────────────────────────────────────────────────────────────────

#[derive(Default, PartialEq, Clone)]
enum FormMode { #[default] New, Edit(String), Import }

#[derive(Default, Clone)]
struct ConnForm {
    mode: FormMode,
    name: String, server: String, user: String,
    password: String, ike: String, esp: String, cert: String,
    show_advanced: bool,
    error: Option<String>,
}

impl ConnForm {
    fn new_defaults() -> Self {
        Self { mode: FormMode::New,
               ike: "aes256-sha256-modp2048".into(),
               esp: "aes256-sha1".into(), ..Default::default() }
    }
    fn from_conn(c: &VpnConnection) -> Self {
        Self { mode: FormMode::Edit(c.name.clone()),
               name: c.name.clone(), server: c.server_address.clone(),
               user: c.username.clone(), ike: c.ike_proposal.clone(),
               esp: c.esp_proposal.clone(), cert: c.certificate_path.clone(),
               ..Default::default() }
    }
    fn from_import(c: VpnConnection) -> Self {
        Self { mode: FormMode::Import,
               name: c.name, server: c.server_address,
               ike: c.ike_proposal, esp: c.esp_proposal,
               cert: c.certificate_path, show_advanced: true,
               ..Default::default() }
    }
    fn to_vpn(&self) -> VpnConnection {
        VpnConnection {
            name: self.name.trim().to_string(),
            server_address: self.server.trim().to_string(),
            username: self.user.trim().to_string(),
            password: self.password.clone(),
            certificate_path: self.cert.trim().to_string(),
            ike_proposal: self.ike.trim().to_string(),
            esp_proposal: self.esp.trim().to_string(),
            status: ConnectionStatus::Disconnected,
            dns_suffix: None, encap: false, ipcomp: false,
        }
    }
}

enum Dialog { None, Form(ConnForm), ConfirmDelete(String), About }

struct Toast { text: String, ok: bool, until: std::time::Instant }

// ── App ───────────────────────────────────────────────────────────────────────

pub struct Wikev2App {
    connections: Vec<VpnConnection>,
    /// Nomi di connessioni con operazione in volo: i refresh non sovrascrivono il loro stato.
    pending: HashSet<String>,
    search: String,
    is_loading: bool,
    dialog: Dialog,
    toast: Option<Toast>,
    palette: Palette,
    ctx: egui::Context,
    event_tx: mpsc::SyncSender<BgEvent>,
    event_rx: mpsc::Receiver<BgEvent>,
    need_focus: bool,
}

impl Wikev2App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let palette = theme::load_palette();
        theme::apply(&cc.egui_ctx, &palette);

        let (tx, rx) = mpsc::sync_channel::<BgEvent>(32);

        // Initial load
        Self::bg_refresh(tx.clone(), cc.egui_ctx.clone());

        // Auto-refresh every 5 s
        let tx2 = tx.clone(); let ctx2 = cc.egui_ctx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                if let Ok(c) = ops::list_connections().await {
                    tx2.send(BgEvent::Connections(c)).ok();
                    ctx2.request_repaint();
                }
            }
        });

        Self {
            connections: Vec::new(), pending: HashSet::new(),
            search: String::new(), is_loading: true,
            dialog: Dialog::None, toast: None, palette,
            ctx: cc.egui_ctx.clone(),
            event_tx: tx, event_rx: rx,
            need_focus: true,
        }
    }

    fn bg_refresh(tx: mpsc::SyncSender<BgEvent>, ctx: egui::Context) {
        tokio::spawn(async move {
            match ops::list_connections().await {
                Ok(c)  => { tx.send(BgEvent::Connections(c)).ok(); }
                Err(e) => { tx.send(BgEvent::Error(e.to_string())).ok(); }
            }
            ctx.request_repaint();
        });
    }

    fn spawn_refresh(&self) {
        Self::bg_refresh(self.event_tx.clone(), self.ctx.clone());
    }

    fn spawn_op<F>(&self, ok_msg: &'static str, conn_name: Option<String>, fut: F)
    where F: std::future::Future<Output = crate::errors::Result<()>> + Send + 'static
    {
        let tx = self.event_tx.clone(); let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match fut.await {
                Ok(()) => { tx.send(BgEvent::Done { msg: ok_msg.to_string(), conn_name }).ok(); }
                Err(e) => { tx.send(BgEvent::Error(e.to_string())).ok(); }
            }
            ctx.request_repaint();
        });
    }

    fn poll(&mut self) {
        while let Ok(ev) = self.event_rx.try_recv() {
            match ev {
                BgEvent::Connections(fresh) => {
                    // Per le connessioni con operazione in volo, preserva lo stato ottimistico
                    // locale invece di sovrascrivere con il dato (potenzialmente stale) di NM.
                    self.connections = fresh.into_iter().map(|mut c| {
                        if self.pending.contains(&c.name) {
                            if let Some(cur) = self.connections.iter().find(|e| e.name == c.name) {
                                c.status = cur.status.clone();
                            }
                        }
                        c
                    }).collect();
                    self.is_loading = false;
                }
                BgEvent::Error(e) => { self.is_loading = false; self.toast_err(e); }
                BgEvent::Done { msg, conn_name } => {
                    self.toast_ok(msg);
                    if let Some(name) = conn_name { self.pending.remove(&name); }
                    self.spawn_refresh();
                }
            }
        }
    }

    fn toast_ok(&mut self, m: String) {
        self.toast = Some(Toast { text: m, ok: true,
            until: std::time::Instant::now() + Duration::from_secs(4) });
    }
    fn toast_err(&mut self, m: String) {
        error!("{}", m);
        self.toast = Some(Toast { text: m, ok: false,
            until: std::time::Instant::now() + Duration::from_secs(7) });
    }
}

impl eframe::App for Wikev2App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Su Wayland il compositor richiede un'esplicita richiesta di focus, altrimenti
        // il primo click/tasto attiva la finestra ma non viene passato all'applicazione.
        if self.need_focus {
            self.need_focus = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        self.poll();
        if let Some(t) = &self.toast {
            if std::time::Instant::now() > t.until { self.toast = None; }
            else { ctx.request_repaint_after(Duration::from_millis(500)); }
        }
        self.ui_toolbar(ctx);
        self.ui_statusbar(ctx);
        self.ui_central(ctx);
        self.ui_dialog(ctx);
    }
}

// ── Toolbar ───────────────────────────────────────────────────────────────────

impl Wikev2App {
    fn ui_toolbar(&mut self, ctx: &egui::Context) {
        let p = self.palette.clone();
        egui::TopBottomPanel::top("toolbar")
            .frame(Frame::none().fill(p.hdr_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("WIKEv2 Connect").size(17.0).strong().color(p.win_fg));
                    ui.label(RichText::new("— VPN Manager IKEv2").size(12.0).color(p.muted_fg));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Aggiorna").clicked() {
                            self.is_loading = true; self.spawn_refresh();
                        }
                        if btn_secondary(ui, "+ Nuova connessione", &p).clicked() {
                            self.dialog = Dialog::Form(ConnForm::new_defaults());
                        }
                        if btn_accent(ui, "Importa config...", &p).clicked() {
                            self.do_import();
                        }

                        // Search field — right_to_left layout, so it appears left of the buttons
                        let search_w = (ui.available_width() - 8.0).min(220.0).max(80.0);
                        let search_resp = ui.add(
                            TextEdit::singleline(&mut self.search)
                                .desired_width(search_w)
                                .hint_text("Cerca connessione...")
                                .frame(true),
                        );
                        if search_resp.changed() {
                            // Repaint is automatic for text edits, nothing extra needed
                        }
                        // Clear button (×) shown only when search is non-empty
                        if !self.search.is_empty() {
                            if ui.small_button("×").on_hover_text("Cancella ricerca").clicked() {
                                self.search.clear();
                            }
                        }
                    });
                });
            });
    }
}

// ── Status bar ────────────────────────────────────────────────────────────────

impl Wikev2App {
    fn ui_statusbar(&mut self, ctx: &egui::Context) {
        let p = self.palette.clone();
        egui::TopBottomPanel::bottom("statusbar")
            .frame(Frame::none().fill(darken(p.win_bg, 10)).inner_margin(Margin::symmetric(16.0, 5.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(t) = &self.toast {
                        let (icon, color) = if t.ok { ("OK:", p.ok) } else { ("Errore:", p.danger) };
                        ui.label(RichText::new(icon).size(12.0).color(color).strong());
                        ui.label(RichText::new(&t.text).size(12.0).color(color));
                    } else if self.is_loading {
                        ui.spinner();
                        ui.label(RichText::new("Aggiornamento in corso...").size(12.0).color(p.muted_fg));
                    } else {
                        let n = self.connections.len();
                        let a = self.connections.iter().filter(|c| c.status == ConnectionStatus::Connected).count();
                        let txt = match n {
                            0 => "Nessuna connessione configurata".into(),
                            _ => format!("{n} connession{} · {a} attiv{}", if n==1 {"e"} else {"i"}, if a==1{"a"} else{"e"}),
                        };
                        ui.label(RichText::new(txt).size(12.0).color(p.muted_fg));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("?").on_hover_text("Informazioni").clicked() {
                            self.dialog = Dialog::About;
                        }
                    });
                });
            });
    }
}

// ── Central panel ─────────────────────────────────────────────────────────────

impl Wikev2App {
    fn ui_central(&mut self, ctx: &egui::Context) {
        let p = self.palette.clone();
        egui::CentralPanel::default()
            .frame(Frame::none().fill(p.win_bg).inner_margin(Margin::symmetric(24.0, 20.0)))
            .show(ctx, |ui| {
                let query = self.search.trim().to_lowercase();
                let visible: Vec<VpnConnection> = self.connections.iter()
                    .filter(|c| {
                        query.is_empty()
                            || c.name.to_lowercase().contains(&query)
                            || c.server_address.to_lowercase().contains(&query)
                            || c.username.to_lowercase().contains(&query)
                    })
                    .cloned()
                    .collect();

                if self.connections.is_empty() && !self.is_loading {
                    self.ui_empty(ui, &p);
                } else if visible.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(RichText::new(format!("Nessun risultato per \"{}\"", self.search))
                            .size(16.0).color(p.muted_fg));
                    });
                } else {
                    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        let mut act_connect: Option<String>    = None;
                        let mut act_disconnect: Option<String> = None;
                        let mut act_edit: Option<String>       = None;
                        let mut act_delete: Option<String>     = None;

                        for conn in &visible {
                            match connection_card(ui, conn, &p) {
                                Action::Connect    => act_connect    = Some(conn.name.clone()),
                                Action::Disconnect => act_disconnect = Some(conn.name.clone()),
                                Action::Edit       => act_edit       = Some(conn.name.clone()),
                                Action::Delete     => act_delete     = Some(conn.name.clone()),
                                Action::None       => {}
                            }
                            ui.add_space(12.0);
                        }

                        if let Some(n) = act_connect {
                            if let Some(c) = self.connections.iter_mut().find(|c| c.name == n) {
                                c.status = ConnectionStatus::Connecting;
                            }
                            self.pending.insert(n.clone());
                            self.spawn_op("Connessione avviata", Some(n.clone()),
                                async move { ops::connect_vpn(&n).await });
                        }
                        if let Some(n) = act_disconnect {
                            if let Some(c) = self.connections.iter_mut().find(|c| c.name == n) {
                                c.status = ConnectionStatus::Disconnected;
                            }
                            self.pending.insert(n.clone());
                            self.spawn_op("Disconnessione completata", Some(n.clone()),
                                async move { ops::disconnect_vpn(&n).await });
                        }
                        if let Some(n) = act_edit {
                            if let Some(c) = self.connections.iter().find(|c| c.name == n) {
                                self.dialog = Dialog::Form(ConnForm::from_conn(c));
                            }
                        }
                        if let Some(n) = act_delete {
                            self.dialog = Dialog::ConfirmDelete(n);
                        }
                    });
                }
            });
    }

    fn ui_empty(&mut self, ui: &mut Ui, p: &Palette) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.label(RichText::new("Nessuna connessione VPN configurata")
                .size(18.0).strong().color(p.win_fg));
            ui.add_space(6.0);
            ui.label(RichText::new(
                "Importa un file .zip o .ps1 da WatchGuard, oppure crea una connessione manualmente.")
                .size(14.0).color(p.muted_fg));
            ui.add_space(28.0);
            ui.horizontal(|ui| {
                let avail = ui.available_width();
                ui.add_space((avail - 310.0).max(0.0) / 2.0);
                if btn_accent_sized(ui, "Importa configurazione...", Vec2::new(220.0, 38.0), p).clicked() {
                    self.do_import();
                }
                ui.add_space(8.0);
                if btn_secondary_sized(ui, "+ Nuova", Vec2::new(80.0, 38.0), p).clicked() {
                    self.dialog = Dialog::Form(ConnForm::new_defaults());
                }
            });
        });
    }
}

// ── Connection card ───────────────────────────────────────────────────────────

enum Action { None, Connect, Disconnect, Edit, Delete }

fn connection_card(ui: &mut Ui, conn: &VpnConnection, p: &Palette) -> Action {
    let mut action = Action::None;

    let connected   = conn.status == ConnectionStatus::Connected;
    let connecting  = conn.status == ConnectionStatus::Connecting;
    let has_error   = matches!(&conn.status, ConnectionStatus::Error(_));

    let (dot_color, status_text, status_color) = match &conn.status {
        ConnectionStatus::Connected    => (p.ok,      "CONNESSA",         p.ok),
        ConnectionStatus::Connecting   => (p.warn,    "IN CONNESSIONE...", p.warn),
        ConnectionStatus::Disconnected => (p.muted_fg,"DISCONNESSA",       p.muted_fg),
        ConnectionStatus::Error(_)     => (p.danger,  "ERRORE",            p.danger),
    };

    // Card border color changes with status
    let border_color = if connected { p.ok } else if has_error { p.danger } else { p.border };
    let border_width = if connected || has_error { 2.0f32 } else { 1.0f32 };

    Frame::none()
        .fill(p.card_bg)
        .rounding(Rounding::same(10.0))
        .stroke(Stroke::new(border_width, border_color))
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            // ── Row 1: name + status badge + action buttons ──────────────────
            ui.horizontal(|ui| {
                // Colored status dot (drawn geometrically — no font needed)
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 6.0, dot_color);
                ui.add_space(4.0);

                // Connection name
                ui.label(RichText::new(&conn.name).size(16.0).strong().color(p.win_fg));

                // Status badge (colored text label)
                ui.add_space(8.0);
                Frame::none()
                    .fill(status_color.linear_multiply(0.15))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(status_text).size(11.0).strong().color(status_color));
                    });

                // Edit / Delete (right-aligned)
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Delete button (red outline)
                    if ui.add(
                        Button::new(RichText::new("Elimina").size(12.0).color(p.danger))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, p.danger))
                            .rounding(Rounding::same(4.0))
                    ).clicked() { action = Action::Delete; }

                    ui.add_space(4.0);

                    // Edit button (secondary)
                    if ui.add(
                        Button::new(RichText::new("Modifica").size(12.0).color(p.muted_fg))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, p.border))
                            .rounding(Rounding::same(4.0))
                    ).clicked() { action = Action::Edit; }
                });
            });

            ui.add_space(6.0);

            // ── Row 2: server / user info ────────────────────────────────────
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                if !conn.server_address.is_empty() {
                    ui.label(RichText::new(&conn.server_address).size(13.0).color(p.muted_fg));
                }
                if !conn.username.is_empty() {
                    ui.label(RichText::new("  —  ").size(13.0).color(p.border));
                    ui.label(RichText::new(&conn.username).size(13.0).color(p.muted_fg));
                }
            });

            // Error detail if any
            if let ConnectionStatus::Error(msg) = &conn.status {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    ui.label(RichText::new(msg).size(12.0).color(p.danger));
                });
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // ── Row 3: main action button ────────────────────────────────────
            let full_w = ui.available_width();

            if connecting {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Connessione in corso...").size(13.0).color(p.muted_fg));
                });
                ui.add_space(6.0);
                // MFA hint — always shown when connecting, since WatchGuard AuthPoint
                // sends a push and the server stays silent until the user approves it.
                Frame::none()
                    .fill(p.warn.linear_multiply(0.12))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::symmetric(10.0, 7.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(
                            "Se il server usa MFA (AuthPoint), approva la notifica push \
                             sul tuo dispositivo. La connessione si completa entro ~90 secondi.")
                            .size(12.0).color(p.warn));
                    });
            } else if connected {
                // Red "Disconnetti" button
                if ui.add_sized(
                    Vec2::new(full_w, 34.0),
                    Button::new(RichText::new("Disconnetti").size(14.0).strong().color(Color32::WHITE))
                        .fill(p.danger)
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(6.0)),
                ).clicked() { action = Action::Disconnect; }
            } else {
                // Olive green "Connetti" button
                if ui.add_sized(
                    Vec2::new(full_w, 34.0),
                    Button::new(RichText::new("Connetti").size(14.0).strong().color(Color32::WHITE))
                        .fill(p.accent)
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(6.0)),
                ).clicked() { action = Action::Connect; }
            }
        });

    action
}

// ── Dialogs ───────────────────────────────────────────────────────────────────

impl Wikev2App {
    fn ui_dialog(&mut self, ctx: &egui::Context) {
        match &self.dialog {
            Dialog::None => {}
            Dialog::Form(_)          => self.ui_form(ctx),
            Dialog::ConfirmDelete(_) => self.ui_confirm_delete(ctx),
            Dialog::About            => self.ui_about(ctx),
        }
    }

    // ── Connection form (new / edit / import) ─────────────────────────────────

    fn ui_form(&mut self, ctx: &egui::Context) {
        let p = self.palette.clone();
        let mut open = true;
        let mut cancel = false;
        let mut save: Option<ConnForm> = None;
        let mut browse_cert = false;

        if let Dialog::Form(form) = &mut self.dialog {
            let title = match &form.mode {
                FormMode::New    => "Nuova connessione VPN",
                FormMode::Edit(_)=> "Modifica connessione VPN",
                FormMode::Import => "Importa configurazione VPN",
            };

            egui::Window::new(title)
                .id(Id::new("conn_form"))
                .collapsible(false).resizable(false)
                .fixed_size(Vec2::new(500.0, 0.0))
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(Frame::window(&ctx.style()).fill(p.view_bg).rounding(Rounding::same(10.0)))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 9.0);

                    // Import info banner (shows only for import mode)
                    if form.mode == FormMode::Import {
                        Frame::none()
                            .fill(p.accent.linear_multiply(0.12))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(Margin::same(10.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(
                                    "Dati importati dalla configurazione WatchGuard. Verifica i campi prima di salvare.")
                                    .size(12.0).color(p.accent));
                            });
                        ui.add_space(4.0);
                    }

                    field_row(ui, &p, "Nome connessione *", |ui| {
                        ui.add(TextEdit::singleline(&mut form.name)
                            .desired_width(f32::INFINITY).hint_text("Es. Office VPN"));
                    });
                    field_row(ui, &p, "Server VPN *", |ui| {
                        ui.add(TextEdit::singleline(&mut form.server)
                            .desired_width(f32::INFINITY).hint_text("Es. vpn.azienda.it"));
                    });
                    field_row(ui, &p, "Nome utente", |ui| {
                        ui.add(TextEdit::singleline(&mut form.user)
                            .desired_width(f32::INFINITY).hint_text("Es. mario.rossi"));
                    });
                    field_row(ui, &p, "Password", |ui| {
                        ui.add(TextEdit::singleline(&mut form.password)
                            .password(true).desired_width(f32::INFINITY));
                    });

                    ui.add_space(2.0);
                    ui.separator();

                    // Advanced (collapsible)
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ctx, ui.make_persistent_id("adv"), form.show_advanced,
                    )
                    .show_header(ui, |ui| {
                        ui.label(RichText::new("Impostazioni avanzate (IKE / ESP / Certificato)")
                            .size(12.0).color(p.muted_fg));
                    })
                    .body(|ui| {
                        form.show_advanced = true;
                        field_row(ui, &p, "IKE Proposal", |ui| {
                            ui.add(TextEdit::singleline(&mut form.ike)
                                .desired_width(f32::INFINITY).hint_text("aes256-sha256-modp2048"));
                        });
                        field_row(ui, &p, "ESP Proposal", |ui| {
                            ui.add(TextEdit::singleline(&mut form.esp)
                                .desired_width(f32::INFINITY).hint_text("aes256-sha1"));
                        });

                        // Certificate field
                        field_row(ui, &p, "Certificato CA", |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(TextEdit::singleline(&mut form.cert)
                                        .desired_width(ui.available_width() - 88.0)
                                        .hint_text("Seleziona file .pem / .crt"));
                                    if btn_secondary(ui, "Sfoglia...", &p).clicked() {
                                        browse_cert = true;
                                    }
                                });
                                // Contextual note
                                let cert_note = if form.cert.is_empty() {
                                    if form.mode == FormMode::Import {
                                        Some("Nessun certificato trovato nel file importato. Selezionane uno se richiesto.")
                                    } else {
                                        Some("Opzionale: certificato CA della VPN (.pem o .crt).")
                                    }
                                } else if form.cert.starts_with("/etc/pki") {
                                    Some("Certificato gia' presente nel sistema.")
                                } else {
                                    Some("Il certificato verra' copiato in /etc/pki/trust/anchors/ al momento del salvataggio.")
                                };
                                if let Some(note) = cert_note {
                                    ui.label(RichText::new(note).size(11.0).color(p.muted_fg).italics());
                                }
                            });
                        });
                    });

                    if let Some(err) = &form.error {
                        ui.add_space(4.0);
                        ui.label(RichText::new(format!("Attenzione: {}", err))
                            .size(12.0).color(p.danger));
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if btn_accent(ui, "Salva", &p).clicked() {
                                if form.name.trim().is_empty() || form.server.trim().is_empty() {
                                    form.error = Some("Nome e Server sono obbligatori.".into());
                                } else {
                                    save = Some(form.clone());
                                }
                            }
                            if ui.button("Annulla").clicked() { cancel = true; }
                        });
                    });
                });
        }

        // File browser (outside Dialog borrow)
        if browse_cert {
            if let Some(p) = rfd::FileDialog::new()
                .set_title("Seleziona certificato CA")
                .add_filter("Certificati", &["pem", "crt", "der"])
                .pick_file()
            {
                if let Dialog::Form(f) = &mut self.dialog {
                    f.cert = p.to_string_lossy().to_string();
                }
            }
        }

        if !open || cancel { self.dialog = Dialog::None; }

        if let Some(form) = save {
            self.dialog = Dialog::None;
            let conn = form.to_vpn();
            match &form.mode {
                FormMode::Edit(orig) => {
                    let orig = orig.clone();
                    self.spawn_op("Connessione aggiornata", None,
                        async move { ops::update_connection(&orig, conn).await });
                }
                _ => {
                    self.spawn_op("Connessione creata", None,
                        async move { ops::create_connection(conn).await });
                }
            }
        }
    }

    // ── Confirm delete ────────────────────────────────────────────────────────

    fn ui_confirm_delete(&mut self, ctx: &egui::Context) {
        let p = self.palette.clone();
        let mut open = true;
        let mut confirm = false;
        let mut cancel = false;

        if let Dialog::ConfirmDelete(name) = &self.dialog {
            let name = name.clone();
            egui::Window::new("Conferma eliminazione")
                .id(Id::new("del_confirm"))
                .collapsible(false).resizable(false)
                .fixed_size(Vec2::new(360.0, 0.0))
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(Frame::window(&ctx.style()).fill(p.view_bg).rounding(Rounding::same(10.0)))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("Eliminare la connessione VPN \"{name}\"?"))
                        .size(14.0).color(p.win_fg));
                    ui.add_space(4.0);
                    ui.label(RichText::new("L'operazione rimuove la connessione da NetworkManager.\nI file di certificato non vengono eliminati.")
                        .size(12.0).color(p.muted_fg));
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.add(Button::new(
                                    RichText::new("Elimina").color(Color32::WHITE))
                                .fill(p.danger).rounding(Rounding::same(5.0))).clicked() {
                                confirm = true;
                            }
                            if ui.button("Annulla").clicked() { cancel = true; }
                        });
                    });
                });
        }

        if !open || cancel { self.dialog = Dialog::None; }
        if confirm {
            if let Dialog::ConfirmDelete(name) = &self.dialog {
                let name = name.clone();
                self.spawn_op("Connessione eliminata", None,
                    async move { ops::delete_connection(&name).await });
            }
            self.dialog = Dialog::None;
        }
    }

    // ── About ─────────────────────────────────────────────────────────────────

    fn ui_about(&mut self, ctx: &egui::Context) {
        let p = self.palette.clone();
        let mut open = true;
        let mut close = false;
        egui::Window::new("Informazioni su WIKEv2 Connect")
            .id(Id::new("about"))
            .collapsible(false).resizable(false)
            .fixed_size(Vec2::new(340.0, 0.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(Frame::window(&ctx.style()).fill(p.view_bg).rounding(Rounding::same(10.0)))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("WIKEv2 Connect").size(22.0).strong().color(p.win_fg));
                    ui.label(RichText::new("v0.1.0").size(13.0).color(p.muted_fg));
                    ui.add_space(10.0);
                    ui.label(RichText::new("Gestione VPN IKEv2 WatchGuard\nper Fedora Linux con KDE Plasma")
                        .size(13.0).color(p.win_fg));
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(RichText::new("Rust  /  egui  /  NetworkManager  /  strongSwan")
                        .size(11.0).color(p.muted_fg));
                    ui.add_space(12.0);
                    if ui.button("Chiudi").clicked() { close = true; }
                });
            });
        if !open || close { self.dialog = Dialog::None; }
    }
}

// ── Import action ─────────────────────────────────────────────────────────────

impl Wikev2App {
    fn do_import(&mut self) {
        let file = rfd::FileDialog::new()
            .set_title("Seleziona file di configurazione VPN")
            .add_filter("WatchGuard VPN Config", &["zip", "ps1"])
            .pick_file();

        if let Some(path) = file {
            match ops::parse_config_file(&path.to_string_lossy()) {
                Ok(conn) => {
                    info!("Config parsed: {} (cert: {})",
                        conn.name, if conn.certificate_path.is_empty() { "nessuno" } else { "trovato" });
                    self.dialog = Dialog::Form(ConnForm::from_import(conn));
                }
                Err(e) => {
                    self.toast_err(format!("Errore parsing: {e}"));
                }
            }
        }
    }
}

// ── Reusable button helpers ───────────────────────────────────────────────────

fn btn_accent(ui: &mut Ui, label: &str, p: &Palette) -> egui::Response {
    ui.add(Button::new(RichText::new(label).color(Color32::WHITE).size(14.0))
        .fill(p.accent).stroke(Stroke::NONE).rounding(Rounding::same(6.0)))
}
fn btn_accent_sized(ui: &mut Ui, label: &str, size: Vec2, p: &Palette) -> egui::Response {
    ui.add_sized(size, Button::new(RichText::new(label).color(Color32::WHITE).size(14.0))
        .fill(p.accent).stroke(Stroke::NONE).rounding(Rounding::same(6.0)))
}
fn btn_secondary(ui: &mut Ui, label: &str, p: &Palette) -> egui::Response {
    ui.add(Button::new(RichText::new(label).color(p.win_fg).size(14.0))
        .fill(p.btn_bg).stroke(Stroke::new(1.0, p.border)).rounding(Rounding::same(6.0)))
}
fn btn_secondary_sized(ui: &mut Ui, label: &str, size: Vec2, p: &Palette) -> egui::Response {
    ui.add_sized(size, Button::new(RichText::new(label).color(p.win_fg).size(14.0))
        .fill(p.btn_bg).stroke(Stroke::new(1.0, p.border)).rounding(Rounding::same(6.0)))
}
fn field_row(ui: &mut Ui, p: &Palette, label: &str, content: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(Vec2::new(145.0, 20.0),
            Label::new(RichText::new(label).size(13.0).color(p.muted_fg)));
        ui.with_layout(Layout::left_to_right(Align::Center).with_main_justify(true), content);
    });
}
