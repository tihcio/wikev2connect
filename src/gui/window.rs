use fltk::{prelude::*, *};
use fltk::image::PngImage;
use crate::gui::state::AppState;
use crate::gui::app::GuiMessage;
use crate::gui::theme::KdeColors;
use crate::vpn::models::{VpnConnection, ConnectionStatus};
use crate::errors::Result;
use fltk::app::Sender;
use log::{info, warn};

static APP_ICON: &[u8] = include_bytes!("../../resources/icona.png");

pub struct MainWindow {
    pub window: window::Window,
    pub state: AppState,
    list: browser::HoldBrowser,
    status_bar: frame::Frame,
    colors: KdeColors,
}

impl MainWindow {
    pub fn new(state: AppState, sender: Sender<GuiMessage>, colors: KdeColors) -> Result<Self> {
        info!("Creating main window");

        const W: i32 = 920;
        const H: i32 = 630;

        // ── Window ────────────────────────────────────────────────────────────
        let mut win = window::Window::default()
            .with_size(W, H)
            .with_label("WIKEv2Connect - VPN Manager");
        win.set_color(KdeColors::fltk(colors.win_bg));

        // ── Menu bar (y=0, h=25) ─────────────────────────────────────────────
        let mut menu = menu::MenuBar::default()
            .with_pos(0, 0)
            .with_size(W, 25);
        menu.set_color(KdeColors::fltk(colors.hdr_bg));
        menu.set_text_color(KdeColors::fltk(colors.win_fg));

        let s = sender.clone();
        menu.add("&File/Importa configurazione\t", enums::Shortcut::Ctrl | 'i',
            menu::MenuFlag::Normal, move |_| { s.send(GuiMessage::ImportConfiguration); });
        let s = sender.clone();
        menu.add("&File/Preferenze\t", enums::Shortcut::None,
            menu::MenuFlag::Normal, move |_| { s.send(GuiMessage::Settings); });
        let s = sender.clone();
        menu.add("&File/Esci\t", enums::Shortcut::Ctrl | 'q',
            menu::MenuFlag::Normal, move |_| { s.send(GuiMessage::Exit); });
        let s = sender.clone();
        menu.add("&VPN/Nuova connessione\t", enums::Shortcut::Ctrl | 'n',
            menu::MenuFlag::Normal, move |_| { s.send(GuiMessage::NewConnection); });
        let s = sender.clone();
        menu.add("&Aiuto/Informazioni\t", enums::Shortcut::None,
            menu::MenuFlag::Normal, move |_| { s.send(GuiMessage::ShowAbout); });

        // ── Header bar (y=25, h=52) ──────────────────────────────────────────
        let mut hdr_bg = frame::Frame::new(0, 25, W, 52, "");
        hdr_bg.set_frame(enums::FrameType::FlatBox);
        hdr_bg.set_color(KdeColors::fltk(colors.hdr_bg));

        // Title — large bold on header background
        let mut title = frame::Frame::new(14, 28, W / 2, 24, "WIKEv2Connect");
        title.set_frame(enums::FrameType::NoBox);
        title.set_label_font(enums::Font::HelveticaBold);
        title.set_label_size(16);
        title.set_label_color(KdeColors::fltk(colors.win_fg));
        title.set_align(enums::Align::Left | enums::Align::Inside);

        // Subtitle — smaller muted text below title
        let muted = KdeColors::darken(colors.win_fg, 80);
        let mut subtitle = frame::Frame::new(15, 52, W / 2, 20,
            "Gestione VPN IKEv2 WatchGuard");
        subtitle.set_frame(enums::FrameType::NoBox);
        subtitle.set_label_size(11);
        subtitle.set_label_color(KdeColors::fltk(muted));
        subtitle.set_align(enums::Align::Left | enums::Align::Inside);

        // Accent line at bottom of header
        let mut accent_line = frame::Frame::new(0, 77, W, 3, "");
        accent_line.set_frame(enums::FrameType::FlatBox);
        accent_line.set_color(KdeColors::fltk(colors.accent));

        // ── Column header strip (y=80, h=22) ─────────────────────────────────
        let col_hdr_color = KdeColors::darken(colors.hdr_bg, 12);
        let mut col_hdr_strip = frame::Frame::new(0, 80, W, 22, "");
        col_hdr_strip.set_frame(enums::FrameType::FlatBox);
        col_hdr_strip.set_color(KdeColors::fltk(col_hdr_color));

        // Column header labels aligned to browser column widths
        // Browser at x=5, columns=[230, 210, 130, rest]
        let col_label_size = 11i32;
        let col_y = 80i32;
        let col_h = 22i32;
        let col_lbl_color = KdeColors::fltk(KdeColors::darken(colors.win_fg, 60));

        let mut ch1 = frame::Frame::new(14, col_y, 220, col_h, "Nome");
        ch1.set_frame(enums::FrameType::NoBox);
        ch1.set_label_size(col_label_size);
        ch1.set_label_color(col_lbl_color);
        ch1.set_align(enums::Align::Left | enums::Align::Inside);

        let mut ch2 = frame::Frame::new(14 + 230, col_y, 200, col_h, "Server");
        ch2.set_frame(enums::FrameType::NoBox);
        ch2.set_label_size(col_label_size);
        ch2.set_label_color(col_lbl_color);
        ch2.set_align(enums::Align::Left | enums::Align::Inside);

        let mut ch3 = frame::Frame::new(14 + 230 + 210, col_y, 120, col_h, "Stato");
        ch3.set_frame(enums::FrameType::NoBox);
        ch3.set_label_size(col_label_size);
        ch3.set_label_color(col_lbl_color);
        ch3.set_align(enums::Align::Left | enums::Align::Inside);

        let mut ch4 = frame::Frame::new(14 + 230 + 210 + 130, col_y, 240, col_h, "Utente");
        ch4.set_frame(enums::FrameType::NoBox);
        ch4.set_label_size(col_label_size);
        ch4.set_label_color(col_lbl_color);
        ch4.set_align(enums::Align::Left | enums::Align::Inside);

        // ── Connection list (y=102, h=453) ────────────────────────────────────
        let mut list = browser::HoldBrowser::new(5, 102, W - 10, 453, "");
        list.set_frame(enums::FrameType::BorderBox);
        list.set_color(KdeColors::fltk(colors.view_bg));
        list.set_selection_color(KdeColors::fltk(colors.sel_bg));
        list.set_text_size(13);
        list.set_column_widths(&[230, 210, 130, 0]);
        list.set_column_char('\t');
        list.add("  (Nessuna connessione VPN configurata)\t\t\t");

        let state_sel = state.clone();
        list.set_callback(move |b| {
            let idx = b.value();
            if idx > 0 {
                if let Some(line) = b.text(idx) {
                    // Strip leading @Bxxx@. prefix if present
                    let text = if line.starts_with('@') {
                        line.splitn(2, "@.").nth(1).unwrap_or(&line).to_string()
                    } else {
                        line.clone()
                    };
                    let name = text.split('\t').next().unwrap_or("").trim().to_string();
                    if !name.is_empty() && !name.starts_with('(') {
                        state_sel.select_connection(name);
                    }
                }
            }
        });

        // ── Buttons (y=561, h=38) ─────────────────────────────────────────────
        //
        // Left group  (create): Importa [10..120]  Nuova [125..235]
        // Right group (manage): Connetti[340..450] Disconnetti[455..565]
        //                       Modifica[570..680] Elimina[685..795] Aggiorna[800..910]
        //
        let btn_y = 561i32;
        let btn_h = 38i32;
        let btn_w = 110i32;

        let style_btn = |b: &mut button::Button, highlight: bool| {
            b.set_color(KdeColors::fltk(colors.btn_bg));
            if highlight {
                b.set_selection_color(KdeColors::fltk(colors.accent));
            } else {
                b.set_selection_color(KdeColors::fltk(KdeColors::darken(colors.btn_bg, 20)));
            }
            b.set_label_color(KdeColors::fltk(colors.win_fg));
            b.set_label_size(13);
        };

        let mut btn_import = button::Button::new(10, btn_y, btn_w, btn_h, "Importa");
        style_btn(&mut btn_import, true);
        let s = sender.clone();
        btn_import.set_callback(move |_| { s.send(GuiMessage::ImportConfiguration); });

        let mut btn_new = button::Button::new(125, btn_y, btn_w, btn_h, "Nuova");
        style_btn(&mut btn_new, true);
        let s = sender.clone();
        btn_new.set_callback(move |_| { s.send(GuiMessage::NewConnection); });

        let mut btn_connect = button::Button::new(340, btn_y, btn_w, btn_h, "Connetti");
        style_btn(&mut btn_connect, false);
        let state_c = state.clone();
        let s = sender.clone();
        btn_connect.set_callback(move |_| {
            if let Some(name) = state_c.get_selected() {
                s.send(GuiMessage::Connect(name));
            }
        });

        let mut btn_disc = button::Button::new(455, btn_y, btn_w, btn_h, "Disconnetti");
        style_btn(&mut btn_disc, false);
        let state_c = state.clone();
        let s = sender.clone();
        btn_disc.set_callback(move |_| {
            if let Some(name) = state_c.get_selected() {
                s.send(GuiMessage::Disconnect(name));
            }
        });

        let mut btn_edit = button::Button::new(570, btn_y, btn_w, btn_h, "Modifica");
        style_btn(&mut btn_edit, false);
        let s = sender.clone();
        btn_edit.set_callback(move |_| { s.send(GuiMessage::EditConnection); });

        let mut btn_del = button::Button::new(685, btn_y, btn_w, btn_h, "Elimina");
        style_btn(&mut btn_del, false);
        let s = sender.clone();
        btn_del.set_callback(move |_| { s.send(GuiMessage::DeleteConnection); });

        let mut btn_refresh = button::Button::new(800, btn_y, btn_w, btn_h, "Aggiorna");
        style_btn(&mut btn_refresh, false);
        let s = sender;
        btn_refresh.set_callback(move |_| { s.send(GuiMessage::RefreshConnections); });

        // ── Status bar (y=607, h=23) ──────────────────────────────────────────
        let status_bg = KdeColors::darken(colors.win_bg, 8);
        let mut status_bar = frame::Frame::new(0, 607, W, 23, " Pronto");
        status_bar.set_frame(enums::FrameType::FlatBox);
        status_bar.set_color(KdeColors::fltk(status_bg));
        status_bar.set_label_size(11);
        status_bar.set_label_color(KdeColors::fltk(muted));
        status_bar.set_align(enums::Align::Left | enums::Align::Inside);

        win.end();

        // set_icon must be called after end() and before show()
        match PngImage::from_data(APP_ICON) {
            Ok(mut icon) => {
                icon.scale(256, 256, true, false);
                win.set_icon(Some(icon));
            }
            Err(e) => warn!("Failed to load app icon: {}", e),
        }

        Ok(Self { window: win, state, list, status_bar, colors })
    }

    pub fn show(&mut self) {
        self.window.show();
        info!("Main window displayed");
    }

    pub fn redraw(&mut self) {
        self.window.redraw();
    }

    pub fn update_connection_list(&mut self, connections: Vec<VpnConnection>) {
        self.list.clear();

        if connections.is_empty() {
            self.list.add("  (Nessuna connessione VPN configurata)\t\t\t");
            self.status_bar.set_label(" Nessuna connessione configurata");
        } else {
            let alt_bg = KdeColors::fltk(self.colors.view_alt).bits();
            let mut n_connected = 0u32;

            for (i, conn) in connections.iter().enumerate() {
                let status_str = match &conn.status {
                    ConnectionStatus::Connected    => { n_connected += 1; "Connessa" }
                    ConnectionStatus::Disconnected => "Disconnessa",
                    ConnectionStatus::Connecting   => "In corso...",
                    ConnectionStatus::Error(_)     => "Errore",
                };
                let row = format!("{}\t{}\t{}\t{}",
                    conn.name, conn.server_address, status_str, conn.username);

                if i % 2 == 0 {
                    self.list.add(&row);
                } else {
                    self.list.add(&format!("@B{}@.{}", alt_bg, row));
                }
            }

            let n = connections.len();
            let status_text = if n_connected > 0 {
                format!(" {} connession{} — {} attiva/e",
                    n, if n == 1 { "e" } else { "i" }, n_connected)
            } else {
                format!(" {} connession{} configurata/e — nessuna attiva",
                    n, if n == 1 { "e" } else { "i" })
            };
            self.status_bar.set_label(&status_text);
        }

        self.window.redraw();
    }
}
