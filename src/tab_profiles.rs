use crate::app::{App, Dialog, ConfirmAction, TextInputAction, COL_GRAY, COL_GREEN};

const COL_BUILT_IN: egui::Color32 = egui::Color32::from_rgb(100, 180, 220);

pub fn render(app: &mut App, ui: &mut egui::Ui, _ctx: &egui::Context) {
    // Header banner
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(26, 26, 46))
        .inner_margin(egui::Margin::symmetric(20.0, 12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new(app.t("Sound Profiles")).color(egui::Color32::from_rgb(192, 200, 255)));
                ui.add_space(12.0);
                ui.colored_label(COL_GRAY, app.t("Save, load, and switch between complete sets of sound overrides"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let active = app.config.data.active_profile.clone();
                    if !active.is_empty() {
                        ui.colored_label(COL_GREEN, format!("{} {}", app.t("Active:"), active));
                    }
                });
            });
        });

    // Main body: left list + right editor
    egui::SidePanel::left("profiles_list_panel")
        .min_width(200.0)
        .max_width(300.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    // ── Built-in profiles ─────────────────────────────────────
                    ui.add_space(8.0);
                    ui.colored_label(COL_BUILT_IN, app.t("BUILT-IN PROFILES"));
                    ui.add_space(4.0);

                    let bundled_names: Vec<String> = app.bundled_profiles.iter()
                        .map(|(n, _)| n.clone())
                        .collect();
                    for name in bundled_names {
                        let sel = app.selected_bundled.as_deref() == Some(&name);
                        let label = egui::RichText::new(name.as_str()).color(COL_BUILT_IN);
                        if ui.selectable_label(sel, label).clicked() {
                            app.selected_bundled = Some(name.clone());
                            app.selected_profile = None;
                            let desc = app.bundled_profiles.iter()
                                .find(|(n, _)| n == &name)
                                .map(|(_, p)| p.description.clone())
                                .unwrap_or_default();
                            app.profile_desc_buf = desc;
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // ── User profiles ─────────────────────────────────────────
                    ui.add_space(4.0);
                    ui.colored_label(COL_GRAY, app.t("YOUR PROFILES"));
                    ui.add_space(4.0);

                    let active = app.config.data.active_profile.clone();
                    let mut names: Vec<String> = app.config.profiles.keys().cloned().collect();
                    names.sort();
                    for name in names {
                        let is_active = name == active;
                        let sel = app.selected_profile.as_deref() == Some(&name);
                        let label = if is_active {
                            egui::RichText::new(name.as_str()).color(COL_GREEN).strong()
                        } else {
                            egui::RichText::new(name.as_str())
                        };
                        if ui.selectable_label(sel, label).clicked() {
                            if app.selected_profile.as_deref() != Some(&name) {
                                app.selected_profile = Some(name.clone());
                                app.selected_bundled = None;
                                if let Some(p) = app.config.profiles.get(&name) {
                                    app.profile_name_buf = name.clone();
                                    app.profile_desc_buf = p.description.clone();
                                }
                            }
                        }
                    }
                });
            });

            ui.add_space(8.0);
            let row = ui.horizontal(|ui| {
                if ui.add(egui::Button::new(app.t("+ New"))
                    .fill(egui::Color32::from_rgb(26, 58, 42))).clicked()
                {
                    app.dialog = Some(Dialog::TextInput {
                        title: "New Profile".into(),
                        prompt: "Enter a name for the new profile:".into(),
                        value: String::new(),
                        action: TextInputAction::NewProfile,
                    });
                }
                let can_dup = app.selected_profile.is_some();
                ui.add_enabled_ui(can_dup, |ui| {
                    if ui.button(app.t("Duplicate")).clicked() {
                        if let Some(ref name) = app.selected_profile.clone() {
                            let src = name.clone();
                            let default_name = format!("{} (copy)", src);
                            app.dialog = Some(Dialog::TextInput {
                                title: "Duplicate Profile".into(),
                                prompt: "Name for the duplicate:".into(),
                                value: default_name,
                                action: TextInputAction::DuplicateProfile(src),
                            });
                        }
                    }
                });
            });
            let row_width = row.response.rect.width();
            ui.add_space(4.0);
            if ui.add(egui::Button::new(app.t("Open Profiles Folder"))
                .min_size(egui::vec2(row_width, 0.0))).clicked()
            {
                let dir = crate::config::profiles_dir();
                std::fs::create_dir_all(&dir).ok();
                opener::open(&dir).ok();
            }
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(ref name) = app.selected_bundled.clone() {
            render_bundled_editor(app, ui, name.clone());
        } else {
            match app.selected_profile.clone() {
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(80, 96, 170),
                            app.t("Select a profile from the list, or create a new one."));
                    });
                }
                Some(ref name) => {
                    if !app.config.profiles.contains_key(name) {
                        app.selected_profile = None;
                        return;
                    }
                    render_profile_editor(app, ui, name.clone());
                }
            }
        }
    });
}

// ── Built-in profile view (load/apply only) ───────────────────────────────────

fn render_bundled_editor(app: &mut App, ui: &mut egui::Ui, name: String) {
    let profile = app.bundled_profiles.iter()
        .find(|(n, _)| n == &name)
        .map(|(_, p)| p.clone());
    let Some(profile) = profile else { return; };

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        // Name + badge
        ui.horizontal(|ui| {
            ui.heading(&name);
            ui.add_space(6.0);
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 60, 80))
                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                .rounding(egui::Rounding::same(4.0))
                .show(ui, |ui| {
                    ui.colored_label(COL_BUILT_IN, app.t("BUILT-IN"));
                });
        });

        // Description
        if !profile.description.is_empty() {
            ui.add_space(6.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.colored_label(COL_GRAY, &profile.description);
            });
        }

        // Stats
        ui.add_space(6.0);
        let n_synth  = profile.synth_overrides.len();
        let n_misc   = profile.misc_overrides.len();
        let n_global = profile.global_overrides.len();
        ui.colored_label(COL_GRAY, format!(
            "{} {}   ·   {} {}  ·  {} {}  ·  {} {}",
            app.t("Created:"), profile.created,
            app.t("Class:"), n_synth,
            app.t("Global:"), n_global,
            app.t("Misc:"), n_misc,
        ));

        ui.separator();
        ui.add_space(4.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Profile Actions")).strong());
            ui.add_space(4.0);

            if ui.add(egui::Button::new(app.t("Load Profile into Editor"))
                .fill(egui::Color32::from_rgb(42, 46, 30))
                .min_size(egui::vec2(300.0, 0.0)))
                .clicked()
            {
                app.dialog = Some(Dialog::Confirm {
                    title: "Load Built-in Profile".into(),
                    message: format!("Load '{}'? This will replace all current settings in the editor.", name),
                    action: ConfirmAction::LoadBundled(name.clone()),
                });
            }

            ui.add_space(2.0);

            if ui.add(egui::Button::new(app.t("Apply Profile to Game"))
                .fill(egui::Color32::from_rgb(26, 58, 26))
                .min_size(egui::vec2(300.0, 0.0)))
                .clicked()
            {
                if !app.backup_exists {
                    app.dialog = Some(Dialog::Info {
                        title: "No Backup".into(),
                        message: "Please create a backup on the Setup tab first.".into(),
                    });
                } else {
                    app.dialog = Some(Dialog::Confirm {
                        title: "Apply Built-in Profile".into(),
                        message: format!("Apply '{}' to game files?", name),
                        action: ConfirmAction::ApplyBundled(name.clone()),
                    });
                }
            }
        });
    });
}

// ── User profile editor ───────────────────────────────────────────────────────

fn render_profile_editor(app: &mut App, ui: &mut egui::Ui, name: String) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Profile Name")).strong());
            ui.horizontal(|ui| {
                let hint = app.t("Profile name…");
                ui.add(egui::TextEdit::singleline(&mut app.profile_name_buf)
                    .desired_width(240.0)
                    .hint_text(hint));
                if ui.button(app.t("Rename")).clicked() {
                    let new_name = app.profile_name_buf.trim().to_string();
                    if !new_name.is_empty() && new_name != name {
                        do_rename_profile(app, &name, &new_name);
                    }
                }
            });
        });

        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Description (optional)")).strong());
            let hint = app.t("Add a description…");
            ui.add(egui::TextEdit::multiline(&mut app.profile_desc_buf)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text(hint));
            if ui.button(app.t("Save Description")).clicked() {
                let desc = app.profile_desc_buf.clone();
                if let Some(p) = app.config.profiles.get_mut(&name) {
                    p.description = desc;
                }
                if let Some(p) = app.config.profiles.get(&name).cloned() {
                    app.config.save_profile(&name, &p);
                }
            }
        });

        ui.add_space(8.0);

        if let Some(p) = app.config.profiles.get(&name) {
            let n_car    = p.car_overrides.values().filter(|v| !v.is_empty()).count();
            let n_global = p.global_overrides.len();
            let n_synth  = p.synth_overrides.len();
            let n_misc   = p.misc_overrides.len();
            let is_active = name == app.config.data.active_profile;
            let active_tag = if is_active { format!("  {}", app.t("[ACTIVE]")) } else { String::new() };
            ui.colored_label(COL_GRAY, format!(
                "{} {}{}\n{} {}  ·  {} {}  ·  {} {}  ·  {} {}",
                app.t("Created:"), p.created, active_tag,
                app.t("Car overrides:"), n_car,
                app.t("Global:"), n_global,
                app.t("Class:"), n_synth,
                app.t("Misc:"), n_misc,
            ));
        }

        ui.separator();
        ui.add_space(4.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(app.t("Profile Actions")).strong());
            ui.add_space(4.0);

            if ui.add(egui::Button::new(app.t("Save Current Settings to This Profile"))
                .fill(egui::Color32::from_rgb(30, 46, 74))
                .min_size(egui::vec2(300.0, 0.0)))
                .clicked()
            {
                app.dialog = Some(Dialog::Confirm {
                    title: "Save to Profile".into(),
                    message: format!("Save current settings to profile '{}'?", name),
                    action: ConfirmAction::SaveToProfile(name.clone()),
                });
            }

            ui.add_space(2.0);

            if ui.add(egui::Button::new(app.t("Load Profile into Editor"))
                .fill(egui::Color32::from_rgb(42, 46, 30))
                .min_size(egui::vec2(300.0, 0.0)))
                .clicked()
            {
                app.dialog = Some(Dialog::Confirm {
                    title: "Load Profile".into(),
                    message: format!("Load '{}'? This will replace all current settings in the editor.", name),
                    action: ConfirmAction::LoadProfile(name.clone()),
                });
            }

            ui.add_space(2.0);

            if ui.add(egui::Button::new(app.t("Apply Profile to Game"))
                .fill(egui::Color32::from_rgb(26, 58, 26))
                .min_size(egui::vec2(300.0, 0.0)))
                .clicked()
            {
                if !app.backup_exists {
                    app.dialog = Some(Dialog::Info {
                        title: "No Backup".into(),
                        message: "Please create a backup on the Setup tab first.".into(),
                    });
                } else {
                    app.dialog = Some(Dialog::Confirm {
                        title: "Apply Profile".into(),
                        message: format!("Apply profile '{}' to game files?", name),
                        action: ConfirmAction::ApplyProfile(name.clone()),
                    });
                }
            }

            ui.add_space(2.0);

            let can_delete = app.config.profiles.len() > 1;
            ui.add_enabled_ui(can_delete, |ui| {
                if ui.add(egui::Button::new(app.t("Delete Profile"))
                    .fill(egui::Color32::from_rgb(58, 26, 26))
                    .min_size(egui::vec2(300.0, 0.0)))
                    .clicked()
                {
                    app.dialog = Some(Dialog::Confirm {
                        title: "Delete Profile".into(),
                        message: format!("Delete profile '{}'? This cannot be undone.", name),
                        action: ConfirmAction::DeleteProfile(name.clone()),
                    });
                }
            });
        });
    });
}

// ── Profile operations ────────────────────────────────────────────────────────

fn do_rename_profile(app: &mut App, old_name: &str, new_name: &str) {
    if app.config.profiles.contains_key(new_name) {
        app.dialog = Some(Dialog::Info {
            title: "Name Taken".into(),
            message: format!("A profile named '{}' already exists.", new_name),
        });
        return;
    }
    if let Some(p) = app.config.profiles.get(old_name).cloned() {
        app.config.save_profile(new_name, &p);
        app.config.delete_profile(old_name);
        if app.config.data.active_profile == old_name {
            app.config.data.active_profile = new_name.to_string();
            app.config.save();
        }
        app.selected_profile = Some(new_name.to_string());
        app.profile_name_buf = new_name.to_string();
    }
}
