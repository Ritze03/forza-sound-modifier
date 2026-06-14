use egui::Color32;
use crate::app::{App, CAR_FIELD_DEFS, Dialog, ConfirmAction, COL_GOLD, COL_GRAY};
use crate::i18n;

pub fn render(app: &mut App, ui: &mut egui::Ui, _ctx: &egui::Context) {

    // Left panel: car list
    egui::SidePanel::left("car_list_panel")
        .min_width(220.0)
        .max_width(320.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.add_space(4.0);
            let hint = app.t("Search cars…");
            ui.add(egui::TextEdit::singleline(&mut app.car_search)
                .hint_text(hint)
                .desired_width(f32::INFINITY));
            ui.add_space(4.0);

            let search = app.car_search.to_lowercase();
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Global overrides entry
                let selected = app.selected_car.as_deref() == Some("[Global Overrides]");
                if ui.selectable_label(selected, "[Global Overrides]").clicked() {
                    app.selected_car = Some("[Global Overrides]".into());
                }
                ui.separator();

                // Car list
                let car_ids: Vec<String> = app.car_list.iter()
                    .filter(|c| search.is_empty() || c.id.to_lowercase().contains(&search))
                    .map(|c| c.id.clone())
                    .collect();

                for car_id in car_ids {
                    let sel = app.selected_car.as_deref() == Some(&car_id);
                    let has_override = app.config.data.car_overrides.contains_key(&car_id);
                    let label = if has_override {
                        egui::RichText::new(format!("● {}", car_id)).color(COL_GOLD)
                    } else {
                        egui::RichText::new(&car_id)
                    };
                    if ui.selectable_label(sel, label).clicked() {
                        app.selected_car = Some(car_id.clone());
                        app.loaded_car_id = None;
                    }
                }
            });
        });

    // Right panel: override editor
    egui::CentralPanel::default().show_inside(ui, |ui| {
        match app.selected_car.clone().as_deref() {
            None => {
                ui.centered_and_justified(|ui| {
                    ui.colored_label(COL_GRAY, app.t("Select a vehicle or [Global Overrides] from the list."));
                });
            }
            Some("[Global Overrides]") => {
                render_global(app, ui);
            }
            Some(car_id) => {
                let car_id = car_id.to_string();
                app.load_car_fields(&car_id);
                render_car(app, ui, &car_id);
            }
        }
    });
}

fn render_global(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading(app.t("Global Overrides"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(app.t("Clear All Global Overrides"))
                .fill(Color32::from_rgb(80, 30, 30))).clicked()
            {
                app.dialog = Some(Dialog::Confirm {
                    title: "Clear Global Overrides".into(),
                    message: "Clear all global overrides?".into(),
                    action: ConfirmAction::ClearGlobalOverrides,
                });
            }
        });
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Core Synthesizer & RPM Channels")).strong());
            ui.add_space(4.0);
            render_global_grid(app, ui, &[
                "RPMScalar_Stock", "RPMScalar_Street", "RPMScalar_Sport", "RPMScalar_Race",
                "Engine_Stock", "Engine_Street", "Engine_Sport", "Engine_Race",
                "Exhaust_Stock", "Exhaust_Street", "Exhaust_Sport", "Exhaust_Race",
                "Intake_Stock",  "Intake_Street",  "Intake_Sport",  "Intake_Race",
            ]);
        });

        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Induction & Gear Whine Profiles")).strong());
            ui.add_space(4.0);
            render_global_grid(app, ui, &[
                "Turbo_Profile", "SuperCSC_Profile", "SuperDSC_Profile", "Transmission_Profile",
            ]);
        });

        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Acoustic Sound Properties")).strong());
            ui.add_space(4.0);
            render_global_grid(app, ui, &[
                "Prop_EngineBank", "Prop_Burbles", "Prop_Backfire", "Prop_AntiLag",
                "Prop_TurboBOV", "Prop_CentrifugalBOV", "Prop_ThrottleBody", "Prop_Limiter",
                "Prop_GearCrack",
            ]);
        });
    });
}

fn render_global_grid(app: &mut App, ui: &mut egui::Ui, keys: &[&'static str]) {
    let mut changed = false;
    let lang = app.config.data.language;
    let human = app.config.data.human_readable_values;
    let transp = app.config.data.translate_params;
    let disp = move |v: &str| -> String {
        if v.is_empty() { return i18n::tr_none(lang).to_string(); }
        if human { i18n::humanize(lang, v) } else { v.to_string() }
    };

    egui::Grid::new(format!("global_grid_{:?}", keys.as_ptr()))
        .num_columns(3)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new(i18n::tr(lang, "Parameter")).weak());
            ui.label(egui::RichText::new(i18n::tr(lang, "Override?")).weak());
            ui.label(egui::RichText::new(i18n::tr(lang, "Value")).weak());
            ui.end_row();

            for key in keys {
                let def = CAR_FIELD_DEFS.iter().find(|(k, _, _)| k == key);
                let display_name = def.map(|(_, d, _)| *d).unwrap_or(key);
                let display_name = if transp { i18n::tr_param(lang, display_name) } else { display_name };
                let is_float = def.map(|(_, _, f)| *f).unwrap_or(false);

                let idx = app.global_fields.iter().position(|(k, _)| k == key);
                if let Some(idx) = idx {
                    let is_diff = app.global_fields[idx].1.is_diff;
                    let label_color = if is_diff { COL_GOLD } else { egui::Color32::GRAY };

                    ui.colored_label(label_color, display_name);

                    let mut en = app.global_fields[idx].1.enabled;
                    if ui.checkbox(&mut en, "").changed() {
                        app.global_fields[idx].1.enabled = en;
                        if !en { app.global_fields[idx].1.value.clear(); }
                        changed = true;
                    }

                    ui.add_enabled_ui(app.global_fields[idx].1.enabled, |ui| {
                        let options = app.options_for_key(key);
                        let val = &mut app.global_fields[idx].1.value;
                        let field_changed = if is_float {
                            ui.add(egui::TextEdit::singleline(val).desired_width(120.0)).changed()
                        } else if options.is_empty() {
                            ui.add(egui::TextEdit::singleline(val).desired_width(200.0)).changed()
                        } else {
                            let before = val.clone();
                            crate::app::combo_disp(ui, &format!("global_combo_{}", key), val, &options, &disp);
                            *val != before
                        };
                        if field_changed { changed = true; }
                    });

                    app.global_fields[idx].1.is_diff = app.global_fields[idx].1.enabled
                        && !app.global_fields[idx].1.value.is_empty();
                    ui.end_row();
                }
            }
        });

    if changed {
        app.save_global_to_config();
    }
}

fn render_car(app: &mut App, ui: &mut egui::Ui, car_id: &str) {
    let display_name = car_id.replace('_', " ");
    ui.horizontal(|ui| {
        ui.heading(&display_name);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(app.t("Clear Overrides for This Car"))
                .fill(Color32::from_rgb(80, 30, 30))).clicked()
            {
                app.dialog = Some(Dialog::Confirm {
                    title: "Clear Car Overrides".into(),
                    message: format!("Clear all overrides for {}?", car_id.replace('_', " ")),
                    action: ConfirmAction::ClearCarOverrides(car_id.to_string()),
                });
            }
        });
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Core Sound Channels")).strong());
            ui.add_space(4.0);
            render_car_grid(app, ui, car_id, &[
                "RPMScalar_Stock", "RPMScalar_Street", "RPMScalar_Sport", "RPMScalar_Race",
                "Engine_Stock", "Engine_Street", "Engine_Sport", "Engine_Race",
                "Exhaust_Stock", "Exhaust_Street", "Exhaust_Sport", "Exhaust_Race",
                "Intake_Stock",  "Intake_Street",  "Intake_Sport",  "Intake_Race",
            ]);
        });

        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Induction & Gear Whine Profiles")).strong());
            ui.add_space(4.0);
            render_car_grid(app, ui, car_id, &[
                "Turbo_Profile", "SuperCSC_Profile", "SuperDSC_Profile", "Transmission_Profile",
            ]);
        });

        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Acoustic Sound Properties")).strong());
            ui.add_space(4.0);
            render_car_grid(app, ui, car_id, &[
                "Prop_EngineBank", "Prop_Burbles", "Prop_Backfire", "Prop_AntiLag",
                "Prop_TurboBOV", "Prop_CentrifugalBOV", "Prop_ThrottleBody", "Prop_Limiter",
                "Prop_GearCrack",
            ]);
        });
    });
}

fn render_car_grid(app: &mut App, ui: &mut egui::Ui, car_id: &str, keys: &[&'static str]) {
    let car = app.car_list.iter().find(|c| c.id == car_id).cloned();
    let lang = app.config.data.language;
    let human = app.config.data.human_readable_values;
    let transp = app.config.data.translate_params;
    let disp = move |v: &str| -> String {
        if v.is_empty() { return i18n::tr_none(lang).to_string(); }
        if human { i18n::humanize(lang, v) } else { v.to_string() }
    };

    egui::Grid::new(format!("car_grid_{}_{:?}", car_id, keys.as_ptr()))
        .num_columns(5)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new(i18n::tr(lang, "Parameter")).weak());
            ui.label(egui::RichText::new(i18n::tr(lang, "Stock Value")).weak());
            ui.label(egui::RichText::new(i18n::tr(lang, "Override?")).weak());
            ui.label(egui::RichText::new(i18n::tr(lang, "New Value")).weak());
            ui.label(egui::RichText::new(i18n::tr(lang, "Link")).weak());
            ui.end_row();

            for key in keys {
                let def = CAR_FIELD_DEFS.iter().find(|(k, _, _)| k == key);
                let display_name = def.map(|(_, d, _)| *d).unwrap_or(key);
                let display_name = if transp { i18n::tr_param(lang, display_name) } else { display_name };
                let is_float = def.map(|(_, _, f)| *f).unwrap_or(false);

                let stock_val = car.as_ref()
                    .and_then(|c| c.stock_values.get(*key))
                    .cloned()
                    .unwrap_or_else(|| "—".into());

                let idx = app.car_fields.iter().position(|(k, _)| k == key);
                if let Some(idx) = idx {
                    let (_, field) = &app.car_fields[idx];
                    let is_diff = field.is_diff;

                    // Parameter name
                    let name_color = if is_diff { COL_GOLD } else { egui::Color32::GRAY };
                    ui.colored_label(name_color, display_name);

                    // Stock value (humanized if enabled)
                    let stock_disp = if human { i18n::humanize(lang, &stock_val) } else { stock_val.clone() };
                    let stock_color = if is_diff { COL_GOLD } else { COL_GRAY };
                    ui.colored_label(stock_color, format!("{}{}", i18n::tr(lang, "Stock: "), stock_disp));

                    // Override checkbox
                    let mut en = app.car_fields[idx].1.enabled;
                    if ui.checkbox(&mut en, "").changed() {
                        app.car_fields[idx].1.enabled = en;
                        if !en { app.car_fields[idx].1.value.clear(); }
                        let cid = car_id.to_string();
                        app.save_car_to_config(&cid);
                        app.recompute_car_diff(&cid);
                    }

                    // Value editor
                    ui.add_enabled_ui(app.car_fields[idx].1.enabled, |ui| {
                        let options = app.options_for_key(key);
                        let val = &mut app.car_fields[idx].1.value;
                        let changed = if is_float {
                            ui.add(egui::TextEdit::singleline(val).desired_width(120.0)).changed()
                        } else if options.is_empty() {
                            ui.add(egui::TextEdit::singleline(val).desired_width(200.0)).changed()
                        } else {
                            let before = val.clone();
                            crate::app::combo_disp(ui, &format!("car_combo_{}_{}", car_id, key), val, &options, &disp);
                            *val != before
                        };
                        if changed {
                            let cid = car_id.to_string();
                            app.save_car_to_config(&cid);
                            app.recompute_car_diff(&cid);
                        }
                    });

                    // Link button
                    if !is_float {
                        let nav_val = {
                            let f = &app.car_fields[idx].1;
                            if f.enabled && !f.value.is_empty() {
                                Some(f.value.clone())
                            } else if let Some(gf) = app.global_fields.iter().find(|(k, _)| k == key) {
                                if gf.1.enabled && !gf.1.value.is_empty() { Some(gf.1.value.clone()) }
                                else { Some(stock_val.clone()) }
                            } else {
                                Some(stock_val.clone())
                            }
                        };
                        let has_target = nav_val.as_deref().map_or(false, |v| !v.is_empty() && v != "—");
                        if ui.add_enabled(has_target, egui::Button::new("->").small()).clicked() {
                            if let Some(v) = nav_val {
                                app.navigate_to_synth = Some(format!("{}.xml", v));
                            }
                        }
                    } else {
                        ui.label("");
                    }

                    ui.end_row();
                }
            }
        });
}
