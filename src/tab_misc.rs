use crate::app::{App, COL_GOLD, COL_GRAY};
use crate::i18n;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(16.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Global Vehicle Attributes Override")).strong().size(14.0));
            ui.add_space(8.0);

            let mut changed = false;
            let lang = app.config.data.language;

            egui::Grid::new("misc_grid")
                .num_columns(4)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(i18n::tr(lang, "Attribute")).weak());
                    ui.label(egui::RichText::new(i18n::tr(lang, "Stock Value")).weak());
                    ui.label(egui::RichText::new(i18n::tr(lang, "Override?")).weak());
                    ui.label(egui::RichText::new(i18n::tr(lang, "New Value")).weak());
                    ui.end_row();

                    // BlurRim MaxRPM
                    {
                        let stock = app.misc_stock.get("blur_rim_max_rpm").cloned().unwrap_or("350.0".into());
                        let is_diff = app.misc_blur_en
                            && app.misc_blur_val != stock
                            && !app.misc_blur_val.is_empty();
                        let nc = if is_diff { COL_GOLD } else { egui::Color32::GRAY };
                        let sc = if is_diff { COL_GOLD } else { COL_GRAY };

                        ui.colored_label(nc, i18n::tr(lang, "BlurRim Rotation Limit (MaxRPM)"));
                        ui.colored_label(sc, format!("{}{}", i18n::tr(lang, "Stock: "), stock));

                        let mut en = app.misc_blur_en;
                        if ui.checkbox(&mut en, "").changed() {
                            app.misc_blur_en = en;
                            if !en { app.misc_blur_val = stock.clone(); }
                            changed = true;
                        }
                        ui.add_enabled_ui(app.misc_blur_en, |ui| {
                            if ui.add(egui::TextEdit::singleline(&mut app.misc_blur_val)
                                .desired_width(120.0)).changed() { changed = true; }
                        });
                        ui.end_row();
                    }

                    // BackfireMinRandTime
                    {
                        let stock = app.misc_stock.get("backfire_min_rand_time").cloned().unwrap_or("100.0".into());
                        let is_diff = app.misc_bfmin_en
                            && app.misc_bfmin_val != stock
                            && !app.misc_bfmin_val.is_empty();
                        let nc = if is_diff { COL_GOLD } else { egui::Color32::GRAY };
                        let sc = if is_diff { COL_GOLD } else { COL_GRAY };

                        ui.colored_label(nc, i18n::tr(lang, "Backfire Minimum Delay (ms)"));
                        ui.colored_label(sc, format!("{}{}", i18n::tr(lang, "Stock: "), stock));

                        let mut en = app.misc_bfmin_en;
                        if ui.checkbox(&mut en, "").changed() {
                            app.misc_bfmin_en = en;
                            if !en { app.misc_bfmin_val = stock.clone(); }
                            changed = true;
                        }
                        ui.add_enabled_ui(app.misc_bfmin_en, |ui| {
                            if ui.add(egui::TextEdit::singleline(&mut app.misc_bfmin_val)
                                .desired_width(120.0)).changed() { changed = true; }
                        });
                        ui.end_row();
                    }

                    // BackfireMaxRandTime
                    {
                        let stock = app.misc_stock.get("backfire_max_rand_time").cloned().unwrap_or("350.0".into());
                        let is_diff = app.misc_bfmax_en
                            && app.misc_bfmax_val != stock
                            && !app.misc_bfmax_val.is_empty();
                        let nc = if is_diff { COL_GOLD } else { egui::Color32::GRAY };
                        let sc = if is_diff { COL_GOLD } else { COL_GRAY };

                        ui.colored_label(nc, i18n::tr(lang, "Backfire Maximum Delay (ms)"));
                        ui.colored_label(sc, format!("{}{}", i18n::tr(lang, "Stock: "), stock));

                        let mut en = app.misc_bfmax_en;
                        if ui.checkbox(&mut en, "").changed() {
                            app.misc_bfmax_en = en;
                            if !en { app.misc_bfmax_val = stock.clone(); }
                            changed = true;
                        }
                        ui.add_enabled_ui(app.misc_bfmax_en, |ui| {
                            if ui.add(egui::TextEdit::singleline(&mut app.misc_bfmax_val)
                                .desired_width(120.0)).changed() { changed = true; }
                        });
                        ui.end_row();
                    }
                });

            if changed { app.save_misc_to_config(); }
        });
    });
}
