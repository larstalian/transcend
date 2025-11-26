use {
    crate::{plugin::Plugin, pty::ShellHost, queue::IoChannels, session::Session},
    anyhow::Error,
    eframe::egui,
    std::thread::JoinHandle,
};

type PtyResult = Result<(), Error>;

pub struct App {
    session: Session,
    pty_handle: Option<JoinHandle<PtyResult>>,
    input_line: String,
    ai_mode: bool,
    plugins: Vec<Box<dyn Plugin>>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let IoChannels {
            out_prod,
            out_cons,
            in_prod,
            in_cons,
        } = IoChannels::new();

        let pty = ShellHost::new(out_prod, in_cons);
        let pty_handle = Some(pty.spawn());
        let session = Session::new(out_cons, in_prod);

        Self {
            session,
            pty_handle,
            input_line: String::new(),
            ai_mode: false,
            plugins: Vec::new(), // later: load statically linked plugins here
        }
    }

    fn handle_shell_enter(&mut self) {
        let cmd = std::mem::take(&mut self.input_line);
        self.session.send_line(&cmd);
    }

    fn handle_ai_enter(&mut self) {
        let prompt = std::mem::take(&mut self.input_line);
        let ctx = self.session.snapshot(200);

        self.session.inject_line("[AI ?]", &prompt);

        // TODO: call real AI here. For now, stub:
        let fake_answer = format!("(stub) ctx.len={} prompt={:?}", ctx.len(), prompt);
        self.session.inject_line("[AI]", &fake_answer);

        self.ai_mode = false;
    }

    // UI

    fn ui_input_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("input_bar").show(ctx, |ui| {
            ui.set_height(30.0);

            ui.horizontal(|ui| {
                // left: AI toggle
                let ai_label = if self.ai_mode { "AI ON" } else { "AI OFF" };
                if ui.toggle_value(&mut self.ai_mode, ai_label).clicked() {
                    // do nothing
                }

                ui.separator();

                // middle: prefix + text edit
                let prefix = if self.ai_mode { "[AI] " } else { "> " };
                ui.label(prefix);

                let resp = ui.text_edit_singleline(&mut self.input_line);

                // submit on Enter
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if self.ai_mode {
                        self.handle_ai_enter();
                    } else {
                        self.handle_shell_enter();
                    }
                    // keep focus on the input
                    resp.request_focus();
                }
            });
        });
    }

    fn ui_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // right plugin panel
            egui::SidePanel::right("plugin_panel")
                .resizable(true)
                .default_width(220.0)
                .show_inside(ui, |plugin_ui| {
                    let snapshot = self.session.snapshot(100);
                    for plugin in &mut self.plugins {
                        plugin.ui(plugin_ui, &snapshot);
                        plugin_ui.separator();
                    }
                });

            // terminal scrollback
            let text = self.session.snapshot(200); // reuse snapshot for now
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.monospace(text);
            });
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(handle) = self.pty_handle.take() {
            let _ = handle.join();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.session.poll_ring();

        self.ui_main(ctx);

        self.ui_input_bar(ctx);

        //TODO: make smarter later
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}
