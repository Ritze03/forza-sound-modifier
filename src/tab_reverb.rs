use crate::app::App;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new("Global Reverb Multipliers").strong().size(14.0));
            ui.add_space(6.0);

            egui::Grid::new("reverb_grid")
                .num_columns(5)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Attribute").weak());
                    ui.label(egui::RichText::new("Mul?").weak());
                    ui.label(egui::RichText::new("Multiplier (1.0 = stock)").weak());
                    ui.label(egui::RichText::new("Off?").weak());
                    ui.label(egui::RichText::new("Offset (+/−)").weak());
                    ui.end_row();

                    let mut changed = false;
                    let n = app.reverb_fields.len();
                    for i in 0..n {
                        let name = app.reverb_fields[i].name.clone();
                        ui.label(&name);

                        let mut mul_en = app.reverb_fields[i].mul_enabled;
                        if ui.checkbox(&mut mul_en, "").changed() {
                            app.reverb_fields[i].mul_enabled = mul_en;
                            changed = true;
                        }

                        ui.add_enabled_ui(app.reverb_fields[i].mul_enabled, |ui| {
                            let resp = ui.add(
                                egui::DragValue::new(&mut app.reverb_fields[i].mul_value)
                                    .speed(0.01)
                                    .range(-100.0..=100.0)
                                    .max_decimals(3)
                            );
                            if resp.changed() { changed = true; }
                        });

                        let mut off_en = app.reverb_fields[i].off_enabled;
                        if ui.checkbox(&mut off_en, "").changed() {
                            app.reverb_fields[i].off_enabled = off_en;
                            changed = true;
                        }

                        ui.add_enabled_ui(app.reverb_fields[i].off_enabled, |ui| {
                            let resp = ui.add(
                                egui::DragValue::new(&mut app.reverb_fields[i].off_value)
                                    .speed(1.0)
                                    .range(-20000.0..=20000.0)
                                    .max_decimals(3)
                            );
                            if resp.changed() { changed = true; }
                        });

                        ui.end_row();
                    }

                    if changed { app.save_reverb_to_config(); }
                });
        });
    });
}
