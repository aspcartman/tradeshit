use chrono::Utc;
use eframe::epaint::Color32;
use egui::RichText;

use crate::quotes::QuotesManager;

pub struct AppState {
    input: String,
    quotes: QuotesManager,
}

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>, qm: QuotesManager) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        AppState { input: "".to_string(), quotes: qm }
    }

    fn quotes_table(&mut self, ctx: &egui::Context) {
        egui::Window::new("Quotes").resizable(true).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.quotes.peek_quotes(|q| {
                    let formatter = timeago::Formatter::default();
                    egui::Grid::new("my_grid").num_columns(4).spacing([40.0, 4.0]).striped(true).show(ui, |ui| {
                        for sym in q.keys() {
                            let data = &q[sym];
                            let lp = formatter.convert_chrono(data.last_price_time, Utc::now());
                            let lu = formatter.convert_chrono(data.last_update, Utc::now());
                            let dt = Utc::now().timestamp() - data.last_update.timestamp();
                            let col = match dt {
                                0..=3 => Some(Color32::GREEN),
                                4..=30 => None,
                                _ => Some(Color32::GRAY),
                            };
                            let mut txt = RichText::new(sym).strong();
                            if let Some(col) = col {
                                txt = txt.color(col);
                            }
                            ui.label(txt);
                            ui.label(data.last_price.to_string());

                            ui.label(lp);
                            ui.label(lu);
                            ui.end_row();
                        }
                    });
                });
            });
            ui.add_space(16f32);

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.input);
                    if ui.button("Add symbol").clicked() {
                        // let inp = self.input.clone();
                        // spawn(async move {
                        //     self.quotes.subscribe(inp.deref()).await;
                        // });
                    };
                });
            });
        });
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.quotes_table(ctx);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
        ctx.request_repaint()
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
