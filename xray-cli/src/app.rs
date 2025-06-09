use eframe::egui;

pub struct XrayGui {
    input_url: String,
    log_output: String,
    xray: super::xray::XrayController,
}

impl Default for XrayGui {
    fn default() -> Self {
        Self {
            input_url: "".to_owned(),
            log_output: "Logs will appear here...".to_owned(),
            xray: super::xray::XrayController::new(),
        }
    }
}

impl XrayGui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for XrayGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Xray GUI Client");

            ui.horizontal(|ui| {
                let input = ui.add(
                    egui::TextEdit::singleline(&mut self.input_url)
                        .hint_text("Paste VLESS URL here"),
                );
                if input.changed() {
                    // Можно добавить валидацию в реальном времени
                }
            });

            let indicator = if self.xray.is_running() {
                '🌑'
            } else {
                '🌕'
            };
            ui.label(format!("{}", indicator));

            if ui.button("Start Proxy").clicked() {
                if let Some(err) = self.xray.start(self.input_url.as_str()).err() {
                    self.log_output
                        .push_str(format!("Error: {}\n", err).as_str());
                }
            }

            if ui.button("Stop Proxy").clicked() {
                if let Some(err) = self.xray.stop().err() {
                    self.log_output
                        .push_str(format!("Error: {}\n", err).as_str());
                }
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(&self.log_output);
            });
        });
    }
}
