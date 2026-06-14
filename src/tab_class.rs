use crate::app::{App, Dialog, ConfirmAction, COL_GOLD, COL_GRAY, COL_BLUE_HINT};
use crate::data::GLOBAL_CATEGORIES;
use crate::i18n;

pub fn render(app: &mut App, ui: &mut egui::Ui, _ctx: &egui::Context) {
    let lang = app.config.data.language;

    // Left panel: synth list
    egui::SidePanel::left("synth_list_panel")
        .min_width(220.0)
        .max_width(340.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.add_space(4.0);
            let hint = i18n::tr(lang, "Search synthesizers…");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.synth_search)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY)
            );
            if resp.changed() { /* filter is live */ }
            ui.add_space(4.0);

            let search = app.synth_search.to_lowercase();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    // Global category entries (always visible)
                    for (cat, _, _) in GLOBAL_CATEGORIES {
                        let sel = app.selected_synth.as_deref() == Some(cat);
                        let has_ov = app.config.data.synth_overrides.contains_key(*cat);
                        let display = i18n::tr_dyn(lang, *cat);
                        let label = if has_ov {
                            egui::RichText::new(format!("● {}", display)).color(COL_GOLD)
                        } else {
                            egui::RichText::new(display).color(COL_BLUE_HINT)
                        };
                        if ui.selectable_label(sel, label).clicked() {
                            if app.selected_synth.as_deref() != Some(cat) {
                                app.selected_synth = Some(cat.to_string());
                                app.loaded_synth = None;
                            }
                        }
                    }
                    ui.separator();

                    // Individual synth files
                    let files: Vec<String> = app.synth_list.iter()
                        .filter(|f| search.is_empty() || f.to_lowercase().contains(&search))
                        .cloned()
                        .collect();
                    for fname in files {
                        let sel = app.selected_synth.as_deref() == Some(&fname);
                        let has_ov = app.config.data.synth_overrides.contains_key(&fname);
                        let label = if has_ov {
                            egui::RichText::new(format!("● {}", fname)).color(COL_GOLD)
                        } else {
                            egui::RichText::new(&fname)
                        };
                        if ui.selectable_label(sel, label).clicked() {
                            if app.selected_synth.as_deref() != Some(&fname) {
                                app.selected_synth = Some(fname.clone());
                                app.loaded_synth = None;
                            }
                        }
                    }
                });
            });
        });

    // Right panel: editor
    egui::CentralPanel::default().show_inside(ui, |ui| {
        match app.selected_synth.clone().as_deref() {
            None => {
                ui.centered_and_justified(|ui| {
                    ui.colored_label(COL_GRAY,
                        i18n::tr(lang, "Select a synthesizer file from the list to modify its parameters."));
                });
            }
            Some(filename) => {
                let filename = filename.to_string();
                app.load_synth_fields(&filename);
                render_synth_editor(app, ui, &filename);
            }
        }
    });
}

fn render_synth_editor(app: &mut App, ui: &mut egui::Ui, filename: &str) {
    let is_global = filename.starts_with("[Global ");
    let lang = app.config.data.language;
    let display_name = i18n::tr_dyn(lang, filename);

    ui.horizontal(|ui| {
        ui.heading(display_name);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(i18n::tr(lang, "Clear Overrides for This Synth"))
                .fill(egui::Color32::from_rgb(80, 30, 30))).clicked()
            {
                app.dialog = Some(Dialog::Confirm {
                    title: "Clear Synth Overrides".into(),
                    message: format!("Clear all overrides for {}?", filename),
                    action: ConfirmAction::ClearSynthOverrides(filename.to_string()),
                });
            }
        });
    });

    // Description banner for global categories
    if is_global {
        if let Some((_, file_pat, desc)) = crate::data::GLOBAL_CATEGORIES.iter()
            .find(|(cat, _, _)| *cat == filename)
        {
            let banner_text = format!("{}  —  {}", file_pat, i18n::tr_dyn(lang, desc));
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 42, 58))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.colored_label(egui::Color32::from_rgb(160, 200, 255), banner_text);
                });
        }
    }

    ui.separator();

    let mut changed = false;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Channel group
        let channel_fields: Vec<usize> = app.synth_fields.iter().enumerate()
            .filter(|(_, f)| f.element == "Channel")
            .map(|(i, _)| i)
            .collect();
        if !channel_fields.is_empty() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                let ch_label = i18n::tr(lang, "Channel Volume & RPM Settings");
                let title = if is_global {
                    format!("{}  —  {}", ch_label, display_name)
                } else {
                    ch_label.to_string()
                };
                ui.label(egui::RichText::new(title).strong());
                ui.add_space(4.0);
                if render_synth_grid(app, ui, &channel_fields, &format!("synth_ch_{}", filename)) { changed = true; }
            });
            ui.add_space(8.0);
        }

        // GearCrack group
        let gc_fields: Vec<usize> = app.synth_fields.iter().enumerate()
            .filter(|(_, f)| f.element == "GearCrack")
            .map(|(i, _)| i)
            .collect();
        if !gc_fields.is_empty() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new(i18n::tr(lang, "Gear Crack Settings")).strong());
                ui.add_space(4.0);
                if render_synth_grid(app, ui, &gc_fields, &format!("synth_gc_{}", filename)) { changed = true; }
            });
            ui.add_space(8.0);
        }

        // Advanced group (fields beyond the core ones)
        let core_attrs = ["MasterVolume", "MinRPM", "MaxRPM", "Volume"];
        let adv_fields: Vec<usize> = app.synth_fields.iter().enumerate()
            .filter(|(_, f)| !core_attrs.contains(&f.attr.as_str()) && f.element != "Channel" && f.element != "GearCrack")
            .map(|(i, _)| i)
            .collect();
        if !adv_fields.is_empty() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new(i18n::tr(lang, "Advanced Attributes")).strong());
                ui.add_space(4.0);
                if render_synth_grid(app, ui, &adv_fields, &format!("synth_adv_{}", filename)) { changed = true; }
            });
        }
    });

    if changed {
        let fname = filename.to_string();
        app.save_synth_to_config(&fname);
    }
}

fn render_synth_grid(app: &mut App, ui: &mut egui::Ui, indices: &[usize], id: &str) -> bool {
    let mut changed = false;
    let lang = app.config.data.language;
    egui::Grid::new(id)
        .num_columns(4)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            let h = ui.text_style_height(&egui::TextStyle::Body);
            ui.add_sized([180.0, h], egui::Label::new(egui::RichText::new(i18n::tr(lang, "Attribute")).weak()));
            ui.add_sized([250.0, h], egui::Label::new(egui::RichText::new(i18n::tr(lang, "Stock Value")).weak()));
            ui.label(egui::RichText::new(i18n::tr(lang, "Override?")).weak());
            ui.add_sized([280.0, h], egui::Label::new(egui::RichText::new(i18n::tr(lang, "Value")).weak()));
            ui.end_row();

            for &idx in indices {
                if render_synth_row(app, ui, idx) { changed = true; }
            }
        });
    changed
}

fn render_synth_row(app: &mut App, ui: &mut egui::Ui, idx: usize) -> bool {
    let mut changed = false;
    let field = app.synth_fields[idx].clone();
    let lang = app.config.data.language;
    let transp = app.config.data.translate_params;
    let human = app.config.data.human_readable_values;

    // Col 1: Attribute name
    let name_color = if field.is_diff { COL_GOLD } else { egui::Color32::GRAY };
    let display_attr = if transp { i18n::tr_attr(lang, &field.attr) } else { &field.attr };
    ui.colored_label(name_color, display_attr);

    // Col 2: Stock value
    if let Some(sv) = &field.stock_value {
        let sc = if field.is_diff { COL_GOLD } else { COL_GRAY };
        let sv_disp = if human { i18n::humanize(lang, sv) } else { sv.clone() };
        ui.colored_label(sc, format!("{}{}", i18n::tr(lang, "Stock: "), sv_disp));
    } else {
        ui.colored_label(COL_BLUE_HINT, i18n::tr(lang, "(global default)"));
    }

    // Col 3: Override checkbox - centered under header
    let mut en = app.synth_fields[idx].enabled;
    if ui.centered_and_justified(|ui| ui.checkbox(&mut en, "")).inner.changed() {
        app.synth_fields[idx].enabled = en;
        if !en { app.synth_fields[idx].value.clear(); }
        changed = true;
    }

    // Col 4: Value editor
    ui.add_enabled_ui(app.synth_fields[idx].enabled, |ui| {
        if field.is_bool {
            let before = app.synth_fields[idx].value.clone();
            egui::ComboBox::from_id_salt(format!("synth_bool_{}_{}", idx, field.attr))
                .selected_text(&app.synth_fields[idx].value)
                .width(280.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.synth_fields[idx].value, "True".into(), "True");
                    ui.selectable_value(&mut app.synth_fields[idx].value, "False".into(), "False");
                });
            if app.synth_fields[idx].value != before { changed = true; }
        } else {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.synth_fields[idx].value)
                    .desired_width(280.0)
            );
            if resp.changed() { changed = true; }
        }
    });

    // Recompute diff
    let sv = app.synth_fields[idx].stock_value.as_deref().unwrap_or("");
    let fname = app.selected_synth.as_deref().unwrap_or("").to_string();
    let is_global_entry = fname.starts_with("[Global ");
    app.synth_fields[idx].is_diff = if is_global_entry || sv.is_empty() {
        app.synth_fields[idx].enabled && !app.synth_fields[idx].value.is_empty()
    } else {
        app.synth_fields[idx].enabled
            && !app.synth_fields[idx].value.is_empty()
            && app.synth_fields[idx].value != sv
    };

    ui.end_row();
    changed
}
