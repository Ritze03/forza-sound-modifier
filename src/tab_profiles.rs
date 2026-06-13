use crate::app::{App, Dialog, ConfirmAction, TextInputAction, COL_GRAY, COL_GREEN};

pub fn render(app: &mut App, ui: &mut egui::Ui, _ctx: &egui::Context) {
    // Header banner
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(26, 26, 46))
        .inner_margin(egui::Margin::symmetric(20.0, 12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Sound Profiles").color(egui::Color32::from_rgb(192, 200, 255)));
                ui.add_space(12.0);
                ui.colored_label(COL_GRAY, "Save, load, and switch between complete sets of sound overrides");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let active = app.config.data.active_profile.clone();
                    if !active.is_empty() {
                        ui.colored_label(COL_GREEN, format!("Active: {}", active));
                    }
                });
            });
        });

    ui.add_space(0.0);

    // Main body: left list + right editor
    egui::SidePanel::left("profiles_list_panel")
        .min_width(200.0)
        .max_width(300.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.add_space(8.0);
            ui.colored_label(COL_GRAY, "SAVED PROFILES");
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let active = app.config.data.active_profile.clone();
                let mut names: Vec<String> = app.config.profiles.keys().cloned().collect();
                names.sort();
                for name in names {
                    let is_active = name == active;
                    let sel = app.selected_profile.as_deref() == Some(&name);
                    let label = if is_active {
                        egui::RichText::new(format!("★  {}", name)).color(COL_GREEN)
                    } else {
                        egui::RichText::new(format!("   {}", name))
                    };
                    if ui.selectable_label(sel, label).clicked() {
                        if app.selected_profile.as_deref() != Some(&name) {
                            app.selected_profile = Some(name.clone());
                            let profile = app.config.profiles.get(&name).cloned();
                            if let Some(p) = profile {
                                app.profile_name_buf = name.clone();
                                app.profile_desc_buf = p.description.clone();
                            }
                        }
                    }
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("＋ New")
                    .fill(egui::Color32::from_rgb(26, 58, 42))).clicked()
                {
                    app.dialog = Some(Dialog::TextInput {
                        title: "New Profile".into(),
                        prompt: "Enter a name for the new profile:".into(),
                        value: String::new(),
                        action: TextInputAction::NewProfile,
                    });
                }
                if ui.button("⧉ Duplicate").clicked() {
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

    egui::CentralPanel::default().show_inside(ui, |ui| {
        match app.selected_profile.clone() {
            None => {
                ui.centered_and_justified(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(80, 96, 170),
                        "Select a profile from the list, or create a new one.");
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
    });

}

fn render_profile_editor(app: &mut App, ui: &mut egui::Ui, name: String) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        // Name row
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new("Profile Name").strong());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut app.profile_name_buf)
                    .desired_width(240.0)
                    .hint_text("Profile name…"));
                if ui.button("✏ Rename").clicked() {
                    let new_name = app.profile_name_buf.trim().to_string();
                    if !new_name.is_empty() && new_name != name {
                        do_rename_profile(app, &name, &new_name);
                    }
                }
            });
        });

        ui.add_space(8.0);

        // Description
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new("Description (optional)").strong());
            ui.add(egui::TextEdit::multiline(&mut app.profile_desc_buf)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text("Add a description…"));
            if ui.button("Save Description").clicked() {
                let desc = app.profile_desc_buf.clone();
                if let Some(p) = app.config.profiles.get_mut(&name) {
                    p.description = desc;
                }
                let profile = app.config.profiles.get(&name).cloned();
                if let Some(p) = profile {
                    app.config.save_profile(&name, &p);
                }
            }
        });

        ui.add_space(8.0);

        // Stats
        if let Some(p) = app.config.profiles.get(&name) {
            let n_car = p.car_overrides.values().filter(|v| !v.is_empty()).count();
            let n_global = p.global_overrides.len();
            let n_synth = p.synth_overrides.len();
            let n_misc = p.misc_overrides.len();
            let is_active = name == app.config.data.active_profile;
            let active_tag = if is_active { "  ✓ ACTIVE" } else { "" };
            ui.colored_label(COL_GRAY,
                format!("Created: {}{}\nCar overrides: {}  ·  Global: {}  ·  Class: {}  ·  Misc: {}",
                    p.created, active_tag, n_car, n_global, n_synth, n_misc));
        }

        ui.separator();
        ui.add_space(4.0);

        // Action buttons
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new("Profile Actions").strong());
            ui.add_space(4.0);

            // Save current settings to profile
            if ui.add(egui::Button::new("💾  Save Current Settings to This Profile")
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

            // Load profile into editor
            if ui.add(egui::Button::new("📂  Load Profile into Editor")
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

            // Apply profile to game
            if ui.add(egui::Button::new("▶  Apply Profile to Game")
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

            // Delete
            let can_delete = app.config.profiles.len() > 1;
            ui.add_enabled_ui(can_delete, |ui| {
                if ui.add(egui::Button::new("🗑  Delete Profile")
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
    let profile = app.config.profiles.get(old_name).cloned();
    if let Some(p) = profile {
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

