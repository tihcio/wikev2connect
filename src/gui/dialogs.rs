use fltk::{prelude::*, *};
use crate::errors::Result;
use crate::gui::theme::KdeColors;
use crate::vpn::models::{VpnConnection, ConnectionStatus};
use log::info;

// ─────────────────────────────────────────────────────────────────────────────
// EditConnectionDialog
// ─────────────────────────────────────────────────────────────────────────────

pub struct EditConnectionDialog {
    window: window::Window,
    inp_name: input::Input,
    inp_server: input::Input,
    inp_user: input::Input,
    inp_ike: input::Input,
    inp_esp: input::Input,
    inp_cert: input::Input,
    inp_pass: input::SecretInput,
    result: std::rc::Rc<std::cell::RefCell<Option<VpnConnection>>>,
}

impl EditConnectionDialog {
    pub fn new(title: &str, prefill: Option<VpnConnection>, colors: &KdeColors) -> Result<Self> {
        info!("Creating EditConnectionDialog: {}", title);

        const W: i32 = 540;
        const H: i32 = 460;

        let mut wind = window::Window::default()
            .with_size(W, H)
            .with_label(title);
        wind.set_color(KdeColors::fltk(colors.win_bg));
        wind.make_modal(true);

        // ── Dialog header strip ───────────────────────────────────────────────
        let mut hdr = frame::Frame::new(0, 0, W, 42, title);
        hdr.set_frame(enums::FrameType::FlatBox);
        hdr.set_color(KdeColors::fltk(colors.hdr_bg));
        hdr.set_label_font(enums::Font::HelveticaBold);
        hdr.set_label_size(13);
        hdr.set_label_color(KdeColors::fltk(colors.win_fg));
        hdr.set_align(enums::Align::Left | enums::Align::Inside);
        hdr.set_pos(14, 0);

        let mut accent = frame::Frame::new(0, 42, W, 2, "");
        accent.set_frame(enums::FrameType::FlatBox);
        accent.set_color(KdeColors::fltk(colors.accent));

        // ── Form layout ───────────────────────────────────────────────────────
        const LBL_W: i32 = 140;
        const INP_W: i32 = 360;
        const INP_X: i32 = 155;
        const LBL_X: i32 = 10;
        const ROW_H: i32 = 30;
        const GAP: i32 = 8;

        let text_color = KdeColors::fltk(colors.win_fg);
        let inp_bg = KdeColors::fltk(colors.view_bg);
        let inp_sel = KdeColors::fltk(colors.sel_bg);

        let mk_label = |text: &'static str, y: i32| {
            let mut f = frame::Frame::new(LBL_X, y, LBL_W, ROW_H, text);
            f.set_frame(enums::FrameType::NoBox);
            f.set_label_size(13);
            f.set_label_color(text_color);
            f.set_align(enums::Align::Right | enums::Align::Inside);
        };

        let mk_input = |y: i32, w: i32| {
            let mut i = input::Input::new(INP_X, y, w, ROW_H, "");
            i.set_color(inp_bg);
            i.set_selection_color(inp_sel);
            i.set_text_color(text_color);
            i.set_text_size(13);
            i.set_frame(enums::FrameType::BorderBox);
            i
        };

        let mut y = 52i32;

        mk_label("Nome connessione:", y);
        let inp_name = mk_input(y, INP_W); y += ROW_H + GAP;

        mk_label("Server VPN:", y);
        let inp_server = mk_input(y, INP_W); y += ROW_H + GAP;

        mk_label("Nome utente:", y);
        let inp_user = mk_input(y, INP_W); y += ROW_H + GAP;

        mk_label("IKE Proposal:", y);
        let inp_ike = mk_input(y, INP_W); y += ROW_H + GAP;

        mk_label("ESP Proposal:", y);
        let inp_esp = mk_input(y, INP_W); y += ROW_H + GAP;

        // Certificato with Browse button
        mk_label("Certificato CA:", y);
        let mut inp_cert = input::Input::new(INP_X, y, INP_W - 90, ROW_H, "");
        inp_cert.set_color(inp_bg);
        inp_cert.set_selection_color(inp_sel);
        inp_cert.set_text_color(text_color);
        inp_cert.set_text_size(13);
        inp_cert.set_frame(enums::FrameType::BorderBox);

        let mut btn_browse = button::Button::new(INP_X + INP_W - 85, y, 85, ROW_H, "Sfoglia...");
        btn_browse.set_color(KdeColors::fltk(colors.btn_bg));
        btn_browse.set_selection_color(KdeColors::fltk(colors.accent));
        btn_browse.set_label_color(text_color);
        btn_browse.set_label_size(13);
        let mut inp_cert_clone = inp_cert.clone();
        btn_browse.set_callback(move |_| {
            if let Some(path) = dialog::file_chooser(
                "Seleziona certificato CA",
                "*.pem\t*.crt\t*.der",
                ".",
                false,
            ) {
                inp_cert_clone.set_value(&path);
            }
        });
        y += ROW_H + GAP;

        // Password
        mk_label("Password:", y);
        let mut inp_pass = input::SecretInput::new(INP_X, y, INP_W, ROW_H, "");
        inp_pass.set_color(inp_bg);
        inp_pass.set_selection_color(inp_sel);
        inp_pass.set_text_color(text_color);
        inp_pass.set_text_size(13);
        inp_pass.set_frame(enums::FrameType::BorderBox);
        y += ROW_H + GAP;

        // Separator line
        let mut sep = frame::Frame::new(10, y + 4, W - 20, 1, "");
        sep.set_frame(enums::FrameType::FlatBox);
        sep.set_color(KdeColors::fltk(KdeColors::darken(colors.win_bg, 20)));
        y += 14;

        // Action buttons (right-aligned)
        let btn_w = 100i32;
        let btn_h = 34i32;
        let btn_y = y;
        let btn_save_x = W - 10 - btn_w - 10 - btn_w;
        let btn_cancel_x = W - 10 - btn_w;

        let mut btn_save = button::Button::new(btn_save_x, btn_y, btn_w, btn_h, "Salva");
        btn_save.set_color(KdeColors::fltk(colors.accent));
        btn_save.set_selection_color(KdeColors::fltk(KdeColors::darken(colors.accent, 20)));
        btn_save.set_label_color(enums::Color::White);
        btn_save.set_label_size(13);

        let mut btn_cancel = button::Button::new(btn_cancel_x, btn_y, btn_w, btn_h, "Annulla");
        btn_cancel.set_color(KdeColors::fltk(colors.btn_bg));
        btn_cancel.set_selection_color(KdeColors::fltk(KdeColors::darken(colors.btn_bg, 20)));
        btn_cancel.set_label_color(text_color);
        btn_cancel.set_label_size(13);

        wind.end();

        // Pre-fill fields
        if let Some(ref conn) = prefill {
            inp_name.clone().set_value(&conn.name);
            inp_server.clone().set_value(&conn.server_address);
            inp_user.clone().set_value(&conn.username);
            inp_ike.clone().set_value(&conn.ike_proposal);
            inp_esp.clone().set_value(&conn.esp_proposal);
            inp_cert.clone().set_value(&conn.certificate_path);
            inp_pass.clone().set_value(&conn.password);
        } else {
            inp_ike.clone().set_value("aes256-sha256-modp2048");
            inp_esp.clone().set_value("aes256-sha1");
        }

        let result: std::rc::Rc<std::cell::RefCell<Option<VpnConnection>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));

        let result_save = result.clone();
        let mut win_save = wind.clone();
        let n = inp_name.clone();
        let sv = inp_server.clone();
        let u = inp_user.clone();
        let ike = inp_ike.clone();
        let esp = inp_esp.clone();
        let cert = inp_cert.clone();
        let pass = inp_pass.clone();
        btn_save.set_callback(move |_| {
            let name = n.value();
            let server = sv.value();
            if name.trim().is_empty() || server.trim().is_empty() {
                dialog::alert_default("Nome e Server sono obbligatori.");
                return;
            }
            *result_save.borrow_mut() = Some(VpnConnection {
                name: name.trim().to_string(),
                server_address: server.trim().to_string(),
                username: u.value().trim().to_string(),
                certificate_path: cert.value().trim().to_string(),
                ike_proposal: ike.value().trim().to_string(),
                esp_proposal: esp.value().trim().to_string(),
                status: ConnectionStatus::Disconnected,
                dns_suffix: None,
                encap: false,
                ipcomp: false,
                password: pass.value(),
            });
            win_save.hide();
        });

        let mut win_cancel = wind.clone();
        btn_cancel.set_callback(move |_| { win_cancel.hide(); });

        Ok(Self { window: wind, inp_name, inp_server, inp_user, inp_ike, inp_esp, inp_cert, inp_pass, result })
    }

    pub fn show_and_wait(&mut self) -> Option<VpnConnection> {
        *self.result.borrow_mut() = None;
        self.window.show();
        let app = app::App::default();
        while self.window.shown() {
            if !app.wait() { break; }
        }
        self.result.borrow().clone()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SettingsDialog
// ─────────────────────────────────────────────────────────────────────────────

pub struct SettingsDialog {
    window: window::Window,
}

impl SettingsDialog {
    pub fn new(colors: &KdeColors) -> Result<Self> {
        info!("Creating settings dialog");

        let mut wind = window::Window::default()
            .with_size(460, 300)
            .with_label("Preferenze WIKEv2Connect");
        wind.set_color(KdeColors::fltk(colors.win_bg));
        wind.make_modal(true);

        let mut hdr = frame::Frame::new(0, 0, 460, 40, "Preferenze");
        hdr.set_frame(enums::FrameType::FlatBox);
        hdr.set_color(KdeColors::fltk(colors.hdr_bg));
        hdr.set_label_font(enums::Font::HelveticaBold);
        hdr.set_label_size(13);
        hdr.set_label_color(KdeColors::fltk(colors.win_fg));
        hdr.set_align(enums::Align::Left | enums::Align::Inside);
        hdr.set_pos(14, 0);

        let mut accent = frame::Frame::new(0, 40, 460, 2, "");
        accent.set_frame(enums::FrameType::FlatBox);
        accent.set_color(KdeColors::fltk(colors.accent));

        let mut lbl = frame::Frame::new(20, 55, 420, 180,
            "Le preferenze saranno disponibili in una versione futura.\n\n\
             Impostazioni correnti:\n\
             - Aggiornamento automatico ogni 5 secondi\n\
             - Configurazioni lette da NetworkManager\n\
             - Certificati installati in /etc/pki/trust/anchors/");
        lbl.set_frame(enums::FrameType::NoBox);
        lbl.set_label_size(13);
        lbl.set_label_color(KdeColors::fltk(colors.win_fg));
        lbl.set_align(enums::Align::TopLeft | enums::Align::Inside | enums::Align::Wrap);

        let mut btn_ok = button::Button::new(180, 255, 100, 32, "OK");
        btn_ok.set_color(KdeColors::fltk(colors.btn_bg));
        btn_ok.set_selection_color(KdeColors::fltk(colors.accent));
        btn_ok.set_label_color(KdeColors::fltk(colors.win_fg));
        btn_ok.set_label_size(13);

        wind.end();

        let mut win_clone = wind.clone();
        btn_ok.set_callback(move |_| { win_clone.hide(); });

        Ok(Self { window: wind })
    }

    pub fn show(&mut self) {
        self.window.show();
        let app = app::App::default();
        while self.window.shown() {
            if !app.wait() { break; }
        }
    }
}

impl Default for SettingsDialog {
    fn default() -> Self {
        Self::new(&KdeColors::default()).expect("Failed to create settings dialog")
    }
}
