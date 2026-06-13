use crate::app::{App, Dialog, ConfirmAction};

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(16.0);

        // ── Game path ─────────────────────────────────────────────────────────
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Folder Selection");
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Forza Horizon 6 Directory:");
                ui.add_space(4.0);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut app.game_path_edit)
                        .desired_width(f32::INFINITY)
                );
                if resp.lost_focus() && app.game_path_edit != app.config.data.game_path {
                    app.config.data.game_path = app.game_path_edit.clone();
                    app.config.save();
                    app.refresh_scan();
                    app.load_misc_from_config();
                }
                if ui.button("Browse…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        app.game_path_edit = path.to_string_lossy().to_string();
                        app.config.data.game_path = app.game_path_edit.clone();
                        app.config.save();
                        app.refresh_scan();
                        app.load_misc_from_config();
                    }
                }
            });
        });

        ui.add_space(12.0);

        // ── Backup status ─────────────────────────────────────────────────────
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Backup & Restore");
            ui.add_space(4.0);

            let color = if app.backup_exists {
                crate::app::COL_GREEN
            } else {
                crate::app::COL_RED
            };
            ui.colored_label(color, format!("Backup Status: {}", app.backup_status_msg));

            ui.add_space(6.0);
            ui.label(egui::RichText::new(
                "The tool reads from a safe Backup folder ('media/Audio_Backup') and applies \
                 modifications to the active game folder. This backup copies only .xml config files, \
                 avoiding gigabytes of audio banks."
            ).color(crate::app::COL_GRAY).size(11.0));

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Create / Refresh Backup")
                    .fill(egui::Color32::from_rgb(40, 80, 40)))
                    .clicked()
                {
                    let game_path = &app.config.data.game_path;
                    let folder = std::path::Path::new(game_path)
                        .file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    if folder != "ForzaHorizon6" {
                        app.dialog = Some(Dialog::Confirm {
                            title: "Warning".into(),
                            message: format!(
                                "The selected folder '{}' is not named 'ForzaHorizon6'. Proceed anyway?",
                                folder
                            ),
                            action: ConfirmAction::CreateBackup,
                        });
                    } else {
                        app.do_create_backup();
                    }
                }

                ui.add_enabled_ui(app.backup_exists, |ui| {
                    if ui.add(egui::Button::new("Restore Original Stock Files")
                        .fill(egui::Color32::from_rgb(100, 30, 30)))
                        .clicked()
                    {
                        app.dialog = Some(Dialog::Confirm {
                            title: "Restore Game Defaults".into(),
                            message: "Restore all sound config files to stock?".into(),
                            action: ConfirmAction::RestoreBackup,
                        });
                    }
                });
            });
        });

        ui.add_space(12.0);

        // ── Instructions ──────────────────────────────────────────────────────
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Usage Instructions");
            ui.add_space(4.0);
            for (_, line) in [
                "1. Select your ForzaHorizon6 game path above.",
                "2. Click 'Create/Refresh Backup' to safe-keep original files (required before applying edits).",
                "3. Customize sound profiles globally or per-car under the Car Overrides tab.",
                "4. Fine-tune class synthesizers under Class Overrides.",
                "5. Adjust backfire and blur rim limits under Miscellaneous.",
                "6. Click the green 'Apply Mod Profile' button at the top-right to write overrides to the game.",
            ].iter().enumerate() {
                ui.label(*line);
            }
        });
    });
}

