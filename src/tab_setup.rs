use crate::app::{App, Dialog, ConfirmAction};
use crate::i18n::Language;

const WIZARD_STEPS: usize = 4;

const WIZARD_HIGHLIGHT: [Option<usize>; WIZARD_STEPS] = [
    Some(2), // step 0 → Language & Display
    Some(0), // step 1 → Folder Selection
    Some(1), // step 2 → Backup & Restore
    None,    // step 3 → done
];

// #2998C6  sky blue
const HL: egui::Color32 = egui::Color32::from_rgb(41, 152, 198);

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    // ── Wizard banner (above scroll area, pinned) ──────────────────────────
    if let Some(step) = app.wizard_step {
        render_wizard_banner(app, ui, step);
        ui.add_space(8.0);
    }

    let highlight = app.wizard_step
        .and_then(|s| WIZARD_HIGHLIGHT.get(s).copied().flatten());

    if highlight.is_some() {
        ui.ctx().request_repaint(); // keep animation running
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Capture width here (inside scroll area) so it accounts for any
        // scrollbar reservation, then pin the layout to it so frames cannot
        // grow wider than the visible viewport.
        let w = ui.available_width();
        ui.set_max_width(w);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.add_space(6.0);

            if app.wizard_step.is_none() {
                if ui.add_sized(
                    egui::vec2(200.0, 28.0),
                    egui::Button::new(app.t("Quick Start Guide"))
                        .fill(egui::Color32::from_rgb(30, 60, 100)),
                ).clicked() {
                    app.wizard_step = Some(0);
                }
                ui.add_space(12.0);
            }

            // ── Folder Selection ────────────────────────────────────────────
            let hl = highlight == Some(0);
            let r = section_frame(ui, hl).show(ui, |ui| {
                ui.heading(app.t("Folder Selection"));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(app.t("Forza Horizon 6 Directory:"));
                    ui.add_space(4.0);
                    let btn_w = 100.0;
                    let gap   = ui.spacing().item_spacing.x;
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut app.game_path_edit)
                            .desired_width(ui.available_width() - btn_w - gap)
                    );
                    if resp.lost_focus() && app.game_path_edit != app.config.data.game_path {
                        app.config.data.game_path = app.game_path_edit.clone();
                        app.config.save();
                        app.refresh_scan();
                        app.load_misc_from_config();
                    }
                    if ui.add(egui::Button::new(
                            format!("{} {}", crate::app::icons::FOLDER, app.t("Browse…")))
                        .min_size(egui::vec2(btn_w, 0.0))).clicked()
                    {
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
            if hl { pulse_rings(ui, r.response.rect); }

            ui.add_space(12.0);

            // ── Backup & Restore ────────────────────────────────────────────
            let hl = highlight == Some(1);
            let r = section_frame(ui, hl).show(ui, |ui| {
                ui.heading(app.t("Backup & Restore"));
                ui.add_space(4.0);
                let (status_color, status_icon) = if app.backup_exists {
                    (crate::app::COL_GREEN, crate::app::icons::CHECK)
                } else {
                    (crate::app::COL_RED, crate::app::icons::TIMES)
                };
                ui.colored_label(status_color,
                    format!("{} {} {}", status_icon, app.t("Backup Status:"), app.backup_status_msg));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(app.t(
                    "The tool reads from a safe Backup folder ('media/Audio_Backup') and applies \
                     modifications to the active game folder. This backup copies only .xml config files, \
                     avoiding gigabytes of audio banks."
                )).color(crate::app::COL_GRAY).size(11.0));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(
                            format!("{} {}", crate::app::icons::SYNC, app.t("Create / Refresh Backup")))
                        .fill(egui::Color32::from_rgb(40, 80, 40))).clicked()
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
                        if ui.add(egui::Button::new(
                                format!("{} {}", crate::app::icons::UNDO, app.t("Restore Original Stock Files")))
                            .fill(egui::Color32::from_rgb(100, 30, 30))).clicked()
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
            if hl { pulse_rings(ui, r.response.rect); }

            ui.add_space(12.0);

            // ── Language & Display ──────────────────────────────────────────
            let hl = highlight == Some(2);
            let r = section_frame(ui, hl).show(ui, |ui| {
                ui.heading(app.t("Language & Display"));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(app.t("Language"));
                    ui.add_space(8.0);
                    egui::ComboBox::from_id_salt("language_select")
                        .selected_text(app.config.data.language.label())
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for lang in [Language::English, Language::German] {
                                if ui.selectable_value(
                                    &mut app.config.data.language, lang, lang.label(),
                                ).changed() {
                                    app.config.save();
                                }
                            }
                        });
                });
                ui.add_space(4.0);
                let mut tp = app.config.data.translate_params;
                if ui.checkbox(&mut tp, app.t("Translate parameter names")).changed() {
                    app.config.data.translate_params = tp;
                    app.config.save();
                }
                let mut hr = app.config.data.human_readable_values;
                if ui.checkbox(&mut hr, app.t("Human-readable values")).changed() {
                    app.config.data.human_readable_values = hr;
                    app.config.save();
                }
            });
            if hl { pulse_rings(ui, r.response.rect); }

            ui.add_space(12.0);

            // ── Usage Instructions ──────────────────────────────────────────
            let hl = highlight == Some(3);
            let r = section_frame(ui, hl).show(ui, |ui| {
                ui.heading(app.t("Usage Instructions"));
                ui.add_space(4.0);
                for line in &[
                    "1. Select your ForzaHorizon6 game path above.",
                    "2. Click 'Create/Refresh Backup' to safe-keep original files (required before applying edits).",
                    "3. Customize sound profiles globally or per-car under the Car Overrides tab.",
                    "4. Fine-tune class synthesizers under Class Overrides.",
                    "5. Adjust backfire and blur rim limits under Miscellaneous.",
                    "6. Click the green 'Apply Mod Profile' button at the top-right to write overrides to the game.",
                ] {
                    ui.label(app.t(*line));
                }
            });
            if hl { pulse_rings(ui, r.response.rect); }
        });
    });
}

// ── Pulsating ring animation ───────────────────────────────────────────────────

fn pulse_rings(ui: &egui::Ui, rect: egui::Rect) {
    let time = ui.input(|i| i.time) as f32;
    // Allow painter to draw up to 24px outside the normal clip rect
    let painter = ui.painter()
        .with_clip_rect(ui.clip_rect().expand(24.0));
    let base_round = 6.0_f32;

    // Fixed solid border
    painter.rect_stroke(rect, egui::Rounding::same(base_round),
        egui::Stroke::new(2.0, HL));

    // Three rings at equally spaced phases — each completes in ~1.6 s
    for i in 0..3usize {
        let phase  = i as f32 / 3.0;
        let t      = (time * 0.625 + phase) % 1.0;
        let expand = t * 22.0;
        let alpha  = ((1.0 - t) * 170.0) as u8;
        let width  = 1.8 * (1.0 - t * 0.6);
        let round  = base_round + expand * 0.35;
        let color  = egui::Color32::from_rgba_unmultiplied(41, 152, 198, alpha);
        painter.rect_stroke(
            rect.expand(expand),
            egui::Rounding::same(round),
            egui::Stroke::new(width, color),
        );
    }
}

// ── Wizard banner ──────────────────────────────────────────────────────────────

fn render_wizard_banner(app: &mut App, ui: &mut egui::Ui, step: usize) {
    let de = app.config.data.language == Language::German;

    let title: &str = match step {
        0 => if de { "Schritt 1 von 4 — Sprache & Darstellung"  } else { "Step 1 of 4 — Language & Display" },
        1 => if de { "Schritt 2 von 4 — Spielverzeichnis"       } else { "Step 2 of 4 — Game Directory" },
        2 => if de { "Schritt 3 von 4 — Backup erstellen"       } else { "Step 3 of 4 — Create a Backup" },
        _ => if de { "Schritt 4 von 4 — Alles bereit!"          } else { "Step 4 of 4 — You're all set!" },
    };

    let body: &str = match step {
        0 => if de {
            "Wähle deine bevorzugte Sprache und Anzeigeoptionen unten aus.\n\
             Du kannst diese Einstellungen jederzeit ändern."
        } else {
            "Choose your preferred language and display options below.\n\
             You can always change these later."
        },
        1 => if de {
            "Wähle deinen Forza Horizon 6 Installationsordner unten aus.\n\
             Dieser befindet sich meist unter  C:\\XboxGames\\Forza Horizon 6\\Content."
        } else {
            "Point to your Forza Horizon 6 installation folder below.\n\
             It is usually located at  C:\\XboxGames\\Forza Horizon 6\\Content."
        },
        2 => if de {
            "Erstelle ein Backup der originalen XML-Konfigurationsdateien, bevor du Mods anwendest.\n\
             So kannst du das Spiel jederzeit auf den Originalzustand zurücksetzen."
        } else {
            "Before applying any sound mods, create a backup of the original XML config files.\n\
             This lets you restore the game to stock at any time without re-downloading files."
        },
        _ => if de {
            "Das Wichtigste auf einen Blick:\n\
             •  Gelb hervorgehobene Einträge weichen vom Serienwert ab.\n\
             •  Fahrzeug-Overrides — Soundprofile pro Fahrzeug anpassen.\n\
             •  Klassen-Overrides — Synthesizer-Parameter für Motorklassen anpassen.\n\
             •  Verschiedenes — Fehlzündungs- und Felgenunschärfe-Grenzen setzen.\n\
             •  Mod-Profil anwenden (oben rechts) — schreibt alle Änderungen ins Spiel."
        } else {
            "Here is what to know:\n\
             •  Items highlighted in yellow have been changed from their stock value.\n\
             •  Car Overrides — tune sound profiles per vehicle.\n\
             •  Class Overrides — adjust synthesizer parameters for engine classes.\n\
             •  Miscellaneous — set backfire and blur-rim limits.\n\
             •  Apply Mod Profile (top-right) — writes all overrides to the game files."
        },
    };

    let lang = app.config.data.language;

    egui::Frame::none()
        .fill(egui::Color32::from_rgb(20, 38, 60))
        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(60, 110, 180)))
        .inner_margin(egui::Margin::symmetric(16.0, 12.0))
        .rounding(egui::Rounding::same(6.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .strong()
                    .color(egui::Color32::from_rgb(140, 190, 255))
                    .size(14.0),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(body)
                    .color(egui::Color32::from_rgb(200, 210, 225)),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                // Step dots
                for i in 0..WIZARD_STEPS {
                    let dot_color = if i == step {
                        egui::Color32::from_rgb(100, 160, 255)
                    } else {
                        egui::Color32::from_rgb(60, 80, 110)
                    };
                    let (dot_rect, _) = ui.allocate_exact_size(
                        egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);
                    ui.add_space(4.0);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let next_label = if step + 1 >= WIZARD_STEPS {
                        crate::i18n::tr(lang, "Finish")
                    } else {
                        crate::i18n::tr(lang, "Next")
                    };
                    if ui.add(
                        egui::Button::new(next_label)
                            .fill(egui::Color32::from_rgb(30, 80, 160))
                            .min_size(egui::vec2(80.0, 24.0)),
                    ).clicked() {
                        app.wizard_step = if step + 1 >= WIZARD_STEPS { None } else { Some(step + 1) };
                    }
                    ui.add_space(6.0);
                    ui.add_enabled_ui(step > 0, |ui| {
                        if ui.add(
                            egui::Button::new(crate::i18n::tr(lang, "Back"))
                                .min_size(egui::vec2(60.0, 24.0)),
                        ).clicked() {
                            app.wizard_step = Some(step - 1);
                        }
                    });
                    ui.add_space(6.0);
                    if ui.add(
                        egui::Button::new(crate::i18n::tr(lang, "Skip"))
                            .fill(egui::Color32::TRANSPARENT)
                            .min_size(egui::vec2(50.0, 24.0)),
                    ).clicked() {
                        app.wizard_step = None;
                    }
                });
            });
        });
}

// ── Frame helper ───────────────────────────────────────────────────────────────

fn section_frame(ui: &egui::Ui, highlighted: bool) -> egui::Frame {
    // outer_margin.right gives the stroke breathing room so it stays inside
    // the visible viewport even when the scroll area width is tight.
    let margin = egui::Margin { right: 6.0, ..Default::default() };
    if highlighted {
        egui::Frame::group(ui.style())
            .outer_margin(margin)
            .stroke(egui::Stroke::new(2.0, HL))
    } else {
        egui::Frame::group(ui.style())
            .outer_margin(margin)
    }
}
