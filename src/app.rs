use std::collections::HashMap;
use egui::Color32;

use crate::config::{Config, now_iso};
use crate::engine::{CarEntry, Engine, ScrapedOptions};
use crate::i18n::{self, Language};

// ── Tab enum ──────────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Setup,
    CarOverrides,
    ClassOverrides,
    Miscellaneous,
    Reverb,
    Profiles,
}

// ── Per-field override state ───────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct OverrideField {
    pub enabled: bool,
    pub value: String,
    /// True when the final effective value differs from the stock value
    pub is_diff: bool,
}

// ── Reverb field state ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ReverbField {
    pub name: String,
    pub mul_enabled: bool,
    pub mul_value: f64,
    pub off_enabled: bool,
    pub off_value: f64,
}

// ── Synth field state ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SynthField {
    pub element: String,
    pub attr: String,
    pub stock_value: Option<String>,
    pub enabled: bool,
    pub value: String,
    pub is_bool: bool,
    pub is_diff: bool,
}

// ── Dialog system ─────────────────────────────────────────────────────────────

pub enum Dialog {
    Confirm { title: String, message: String, action: ConfirmAction },
    TextInput { title: String, prompt: String, value: String, action: TextInputAction },
    Info { title: String, message: String },
}

pub enum ConfirmAction {
    CreateBackup,
    RestoreBackup,
    ClearGlobalOverrides,
    ClearCarOverrides(String),
    ClearSynthOverrides(String),
    DeleteProfile(String),
    LoadProfile(String),
    SaveToProfile(String),
    ApplyProfile(String),
    LoadBundled(String),
    ApplyBundled(String),
}

pub enum TextInputAction {
    NewProfile,
    DuplicateProfile(String),
}

// ── Main App struct ───────────────────────────────────────────────────────────

pub struct App {
    pub config: Config,

    // Game data
    pub car_list: Vec<CarEntry>,
    pub synth_list: Vec<String>,
    pub scraped: ScrapedOptions,

    // Navigation
    pub active_tab: Tab,
    pub backup_exists: bool,
    pub backup_status_msg: String,
    pub status_msg: String,

    // Setup tab
    pub game_path_edit: String,

    // Car / Global Overrides tab
    pub car_search: String,
    pub selected_car: Option<String>,        // None = nothing, Some("[Global Overrides]") = global
    pub car_fields: Vec<(String, OverrideField)>, // (key, state) for per-car
    pub global_fields: Vec<(String, OverrideField)>, // (key, state) for global overrides
    pub loaded_car_id: Option<String>,       // which car's data is in car_fields
    pub misc_stock: HashMap<String, String>,

    // Class tab
    pub synth_search: String,
    pub selected_synth: Option<String>,
    pub synth_fields: Vec<SynthField>,
    pub loaded_synth: Option<String>,
    pub navigate_to_synth: Option<String>,

    // Misc tab
    pub misc_blur_en: bool,
    pub misc_blur_val: String,
    pub misc_bfmin_en: bool,
    pub misc_bfmin_val: String,
    pub misc_bfmax_en: bool,
    pub misc_bfmax_val: String,

    // Reverb tab
    pub reverb_fields: Vec<ReverbField>,

    // Profiles tab
    pub selected_profile: Option<String>,
    pub selected_bundled: Option<String>,
    pub profile_name_buf: String,
    pub profile_desc_buf: String,
    pub bundled_profiles: Vec<(String, crate::config::ProfileData)>,

    // Dialog
    pub dialog: Option<Dialog>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_visuals(&cc.egui_ctx);
        let config = Config::new();
        let game_path_edit = config.data.game_path.clone();

        let mut app = App {
            active_tab: Tab::Setup,
            backup_exists: false,
            backup_status_msg: String::new(),
            status_msg: "Ready".into(),
            car_search: String::new(),
            selected_car: None,
            car_fields: Vec::new(),
            global_fields: build_global_fields(&config),
            loaded_car_id: None,
            misc_stock: HashMap::new(),
            synth_search: String::new(),
            selected_synth: None,
            synth_fields: Vec::new(),
            loaded_synth: None,
            navigate_to_synth: None,
            misc_blur_en: false,
            misc_blur_val: "350.0".into(),
            misc_bfmin_en: false,
            misc_bfmin_val: "100.0".into(),
            misc_bfmax_en: false,
            misc_bfmax_val: "350.0".into(),
            reverb_fields: build_reverb_fields(&config),
            selected_profile: None,
            selected_bundled: None,
            profile_name_buf: String::new(),
            profile_desc_buf: String::new(),
            bundled_profiles: load_bundled_profiles(),
            dialog: None,
            car_list: Vec::new(),
            synth_list: Vec::new(),
            scraped: ScrapedOptions::default(),
            game_path_edit,
            config,
        };
        app.refresh_scan();
        app.load_misc_from_config();
        app
    }

    pub fn refresh_scan(&mut self) {
        let (ok, msg) = Engine::check_backup(&self.config);
        self.backup_exists = ok;
        self.backup_status_msg = msg;
        if ok {
            let (cars, scraped) = Engine::scan_cars(&self.config);
            self.car_list = cars;
            self.scraped = scraped;
            self.synth_list = Engine::scan_synths(&self.config);
            self.misc_stock = Engine::read_misc_stock(&self.config);
            self.rebuild_global_fields();
        } else {
            self.car_list.clear();
            self.synth_list.clear();
        }
    }

    fn rebuild_global_fields(&mut self) {
        self.global_fields = build_global_fields(&self.config);
    }

    pub fn load_car_fields(&mut self, car_id: &str) {
        if self.loaded_car_id.as_deref() == Some(car_id) { return; }
        let car = self.car_list.iter().find(|c| c.id == car_id).cloned();
        let car_ov = self.config.data.car_overrides.get(car_id).cloned().unwrap_or_default();
        let global_ov = &self.config.data.global_overrides;

        self.car_fields = CAR_FIELD_DEFS.iter().map(|(key, _, _is_float)| {
            let stock_val = car.as_ref().and_then(|c| c.stock_values.get(*key)).cloned();
            let ov_val = car_ov.get(*key).cloned();
            let enabled = ov_val.is_some();
            let value = ov_val.clone().unwrap_or_default();

            let effective = if enabled {
                ov_val.as_deref().unwrap_or("").to_string()
            } else if let Some(gv) = global_ov.get(*key) {
                gv.clone()
            } else {
                stock_val.clone().unwrap_or_default()
            };
            let is_diff = effective != stock_val.as_deref().unwrap_or("") && !effective.is_empty();

            (key.to_string(), OverrideField { enabled, value, is_diff })
        }).collect();

        self.loaded_car_id = Some(car_id.to_string());
    }

    pub fn save_car_to_config(&mut self, car_id: &str) {
        if car_id == "[Global Overrides]" { return; }
        let mut car_map: HashMap<String, String> = HashMap::new();
        for (key, field) in &self.car_fields {
            if field.enabled && !field.value.is_empty() {
                car_map.insert(key.clone(), field.value.replace(',', "."));
            }
        }
        if car_map.is_empty() {
            self.config.data.car_overrides.remove(car_id);
        } else {
            self.config.data.car_overrides.insert(car_id.to_string(), car_map);
        }
        self.config.save();
    }

    pub fn save_global_to_config(&mut self) {
        let mut map: HashMap<String, String> = HashMap::new();
        for (key, field) in &self.global_fields {
            if field.enabled && !field.value.is_empty() {
                map.insert(key.clone(), field.value.replace(',', "."));
            }
        }
        self.config.data.global_overrides = map;
        self.config.save();
    }

    pub fn recompute_car_diff(&mut self, car_id: &str) {
        let car = self.car_list.iter().find(|c| c.id == car_id).cloned();
        let global_ov = self.config.data.global_overrides.clone();
        for (key, field) in &mut self.car_fields {
            let stock_val = car.as_ref().and_then(|c| c.stock_values.get(key)).cloned().unwrap_or_default();
            let effective = if field.enabled && !field.value.is_empty() {
                field.value.clone()
            } else if let Some(gv) = global_ov.get(key) {
                gv.clone()
            } else {
                stock_val.clone()
            };
            field.is_diff = !effective.is_empty() && effective != stock_val;
        }
    }

    pub fn load_synth_fields(&mut self, filename: &str) {
        if self.loaded_synth.as_deref() == Some(filename) { return; }
        let is_global = filename.starts_with("[Global ");
        let overrides = self.config.data.synth_overrides.get(filename).cloned().unwrap_or_default();
        let mut fields = Vec::new();

        if is_global {
            for (elem, attrs_keys) in &[
                ("Channel",  vec!["MasterVolume", "MinRPM", "MaxRPM"]),
                ("GearCrack", vec!["Volume", "MinRPM"]),
            ] {
                let elem_ov = overrides.get(*elem).cloned().unwrap_or_default();
                for attr in attrs_keys {
                    let ov = elem_ov.get(*attr).cloned();
                    fields.push(SynthField {
                        element: elem.to_string(),
                        attr: attr.to_string(),
                        stock_value: None,
                        enabled: ov.is_some(),
                        value: ov.unwrap_or_default(),
                        is_bool: false,
                        is_diff: false,
                    });
                }
            }
            // Mark is_diff for globals
            for f in &mut fields {
                f.is_diff = f.enabled && !f.value.is_empty();
            }
        } else {
            let stock = Engine::load_synth_values(&self.config, filename);
            for (elem, core_attrs) in &[
                ("Channel",   vec!["MasterVolume", "MinRPM", "MaxRPM"]),
                ("GearCrack", vec!["Volume", "MinRPM"]),
            ] {
                let stock_elem = stock.get(*elem).cloned().unwrap_or_default();
                let ov_elem = overrides.get(*elem).cloned().unwrap_or_default();
                // Core attrs first
                for attr in core_attrs {
                    if let Some(sv) = stock_elem.get(*attr) {
                        let ov = ov_elem.get(*attr).cloned();
                        let is_bool = sv.to_lowercase() == "true" || sv.to_lowercase() == "false";
                        let is_diff = ov.as_deref().map_or(false, |v| v != sv);
                        fields.push(SynthField {
                            element: elem.to_string(),
                            attr: attr.to_string(),
                            stock_value: Some(sv.clone()),
                            enabled: ov.is_some(),
                            value: ov.unwrap_or_default(),
                            is_bool,
                            is_diff,
                        });
                    }
                }
                // Advanced attrs
                for (attr, sv) in &stock_elem {
                    if core_attrs.contains(&attr.as_str()) { continue; }
                    let ov = ov_elem.get(attr).cloned();
                    let is_bool = sv.to_lowercase() == "true" || sv.to_lowercase() == "false";
                    let is_diff = ov.as_deref().map_or(false, |v| v != sv);
                    fields.push(SynthField {
                        element: elem.to_string(),
                        attr: attr.clone(),
                        stock_value: Some(sv.clone()),
                        enabled: ov.is_some(),
                        value: ov.unwrap_or_default(),
                        is_bool,
                        is_diff,
                    });
                }
            }
        }

        self.synth_fields = fields;
        self.loaded_synth = Some(filename.to_string());
    }

    pub fn save_synth_to_config(&mut self, filename: &str) {
        let mut file_map: HashMap<String, HashMap<String, String>> = HashMap::new();
        for field in &self.synth_fields {
            if field.enabled && !field.value.is_empty() {
                file_map.entry(field.element.clone())
                    .or_default()
                    .insert(field.attr.clone(), field.value.replace(',', "."));
            }
        }
        if file_map.is_empty() {
            self.config.data.synth_overrides.remove(filename);
        } else {
            self.config.data.synth_overrides.insert(filename.to_string(), file_map);
        }
        self.config.save();
        // Recompute is_diff
        let filename = filename.to_string();
        self.loaded_synth = None; // Force reload
        self.load_synth_fields(&filename);
    }

    pub fn load_misc_from_config(&mut self) {
        self.misc_stock = Engine::read_misc_stock(&self.config);
        let misc_ov = &self.config.data.misc_overrides;
        if let Some(v) = misc_ov.get("blur_rim_max_rpm") {
            self.misc_blur_en = true; self.misc_blur_val = v.clone();
        } else {
            self.misc_blur_val = self.misc_stock.get("blur_rim_max_rpm").cloned().unwrap_or("350.0".into());
        }
        if let Some(v) = misc_ov.get("backfire_min_rand_time") {
            self.misc_bfmin_en = true; self.misc_bfmin_val = v.clone();
        } else {
            self.misc_bfmin_val = self.misc_stock.get("backfire_min_rand_time").cloned().unwrap_or("100.0".into());
        }
        if let Some(v) = misc_ov.get("backfire_max_rand_time") {
            self.misc_bfmax_en = true; self.misc_bfmax_val = v.clone();
        } else {
            self.misc_bfmax_val = self.misc_stock.get("backfire_max_rand_time").cloned().unwrap_or("350.0".into());
        }
    }

    pub fn save_misc_to_config(&mut self) {
        let mut map = HashMap::new();
        if self.misc_blur_en  && !self.misc_blur_val.is_empty()  { map.insert("blur_rim_max_rpm".into(),        self.misc_blur_val.clone()); }
        if self.misc_bfmin_en && !self.misc_bfmin_val.is_empty() { map.insert("backfire_min_rand_time".into(),  self.misc_bfmin_val.clone()); }
        if self.misc_bfmax_en && !self.misc_bfmax_val.is_empty() { map.insert("backfire_max_rand_time".into(),  self.misc_bfmax_val.clone()); }
        self.config.data.misc_overrides = map;
        self.config.save();
    }

    pub fn save_reverb_to_config(&mut self) {
        let mut map = HashMap::new();
        for f in &self.reverb_fields {
            if f.mul_enabled { map.insert(f.name.clone(), f.mul_value); }
            if f.off_enabled { map.insert(format!("{}_offset", f.name), f.off_value); }
        }
        self.config.data.reverb_overrides = map;
        self.config.save();
    }

    pub fn apply_changes(&mut self) -> Result<(), String> {
        self.save_misc_to_config();
        let mut _prog = 0u32;
        Engine::apply_overrides(&self.config, &mut |p| { _prog = p; })
    }

    pub fn do_create_backup(&mut self) {
        self.status_msg = "Creating backup...".into();
        let mut _p = 0u32;
        match Engine::create_backup(&self.config, &mut |p| { _p = p; }) {
            Ok(()) => {
                self.refresh_scan();
                self.load_misc_from_config();
                self.status_msg = "Backup created successfully.".into();
                self.dialog = Some(Dialog::Info {
                    title: "Success".into(),
                    message: "XML backup created successfully!".into(),
                });
            }
            Err(e) => {
                self.dialog = Some(Dialog::Info {
                    title: "Error".into(),
                    message: format!("Failed to create backup:\n{}", e),
                });
            }
        }
    }

    pub fn reload_all_ui_from_config(&mut self) {
        self.rebuild_global_fields();
        self.loaded_car_id = None;
        self.loaded_synth = None;
        self.load_misc_from_config();
        self.reverb_fields = build_reverb_fields(&self.config);
    }

    pub fn options_for_key(&self, key: &str) -> Vec<String> {
        use crate::data;
        if key.starts_with("Engine_") {
            self.scraped.engine.clone()
        } else if key.starts_with("Exhaust_") {
            self.scraped.exhaust.clone()
        } else if key.starts_with("Intake_") {
            self.scraped.intake.clone()
        } else {
            data::options_for_key(key).iter().map(|s| s.to_string()).collect()
        }
    }

    pub fn lang(&self) -> Language { self.config.data.language }

    pub fn t(&self, key: &str) -> &'static str {
        i18n::tr(self.lang(), key)
    }

    pub fn tr_param(&self, name: &str) -> &'static str {
        if self.config.data.translate_params {
            i18n::tr_param(self.lang(), name)
        } else {
            name
        }
    }

    pub fn disp_val(&self, raw: &str) -> String {
        if self.config.data.human_readable_values {
            i18n::humanize(self.lang(), raw)
        } else {
            raw.to_string()
        }
    }

    pub fn stock_label(&self, val: &str) -> String {
        format!("{}{}", self.t("Stock: "), self.disp_val(val))
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle pending navigation
        if let Some(synth) = self.navigate_to_synth.take() {
            self.active_tab = Tab::ClassOverrides;
            self.loaded_synth = None;
            self.selected_synth = Some(synth);
        }

        // Top panel: title + apply button
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Forza Horizon 6 Sound Modifier");
                ui.add_space(12.0);
                // Tab buttons
                for (tab, key) in &[
                    (Tab::Setup,         "Setup"),
                    (Tab::CarOverrides,  "Car Overrides"),
                    (Tab::ClassOverrides,"Class Overrides"),
                    (Tab::Miscellaneous, "Miscellaneous"),
                    (Tab::Reverb,        "Reverb"),
                    (Tab::Profiles,      "Profiles"),
                ] {
                    let selected = self.active_tab == *tab;
                    let enabled = selected || self.backup_exists || *tab == Tab::Setup || *tab == Tab::Profiles;
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.selectable_label(selected, self.t(*key)).clicked() {
                            self.active_tab = *tab;
                        }
                    });
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn = egui::Button::new(self.t("Apply Mod Profile"))
                        .fill(Color32::from_rgb(34, 139, 34));
                    if ui.add(btn).clicked() {
                        if !self.backup_exists {
                            self.dialog = Some(Dialog::Info {
                                title: "No Backup".into(),
                                message: "Please create a backup on the Setup tab first.".into(),
                            });
                        } else {
                            self.save_misc_to_config();
                            match self.apply_changes() {
                                Ok(()) => {
                                    self.status_msg = "Overrides applied successfully!".into();
                                    self.dialog = Some(Dialog::Info {
                                        title: "Success".into(),
                                        message: "Overrides applied to game files!".into(),
                                    });
                                }
                                Err(e) => {
                                    self.dialog = Some(Dialog::Info {
                                        title: "Error".into(),
                                        message: format!("Failed to apply overrides:\n{}", e),
                                    });
                                }
                            }
                        }
                    }
                });
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Setup         => crate::tab_setup::render(self, ui),
                Tab::CarOverrides  => crate::tab_car::render(self, ui, ctx),
                Tab::ClassOverrides=> crate::tab_class::render(self, ui, ctx),
                Tab::Miscellaneous => crate::tab_misc::render(self, ui),
                Tab::Reverb        => crate::tab_reverb::render(self, ui),
                Tab::Profiles      => crate::tab_profiles::render(self, ui, ctx),
            }
        });

        // Dialog overlay
        show_dialog(self, ctx);
    }
}

fn show_dialog(app: &mut App, ctx: &egui::Context) {
    // Take the dialog out so we can call app methods freely after rendering.
    let mut dialog = match app.dialog.take() {
        Some(d) => d,
        None => return,
    };

    #[derive(PartialEq)]
    enum Outcome { Open, Close, Confirm, TextOk }
    let mut outcome = Outcome::Open;

    match dialog {
        Dialog::Info { ref title, ref message } => {
            egui::Window::new(title.as_str())
                .collapsible(false).resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(message.as_str());
                    ui.add_space(8.0);
                    if ui.button(app.t("OK")).clicked() { outcome = Outcome::Close; }
                });
        }
        Dialog::Confirm { ref title, ref message, .. } => {
            let t = title.clone();
            let m = message.clone();
            egui::Window::new(t.as_str())
                .collapsible(false).resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(m.as_str());
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(app.t("Yes")).clicked() { outcome = Outcome::Confirm; }
                        if ui.button(app.t("No")).clicked()  { outcome = Outcome::Close; }
                    });
                });
        }
        Dialog::TextInput { ref title, ref prompt, ref mut value, .. } => {
            let t = title.clone();
            let p = prompt.clone();
            egui::Window::new(t.as_str())
                .collapsible(false).resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .min_width(300.0)
                .show(ctx, |ui| {
                    ui.label(p.as_str());
                    let resp = ui.text_edit_singleline(value);
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        outcome = Outcome::TextOk;
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(app.t("OK")).clicked()     { outcome = Outcome::TextOk; }
                        if ui.button(app.t("Cancel")).clicked() { outcome = Outcome::Close; }
                    });
                });
        }
    }

    match outcome {
        Outcome::Open => {
            app.dialog = Some(dialog); // still open, put it back
        }
        Outcome::Close => {
            // discarded
        }
        Outcome::Confirm => {
            if let Dialog::Confirm { action, .. } = dialog {
                dispatch_confirm(app, action);
            }
        }
        Outcome::TextOk => {
            if let Dialog::TextInput { value, action, .. } = dialog {
                dispatch_text(app, action, value);
            }
        }
    }
}

fn dispatch_confirm(app: &mut App, action: ConfirmAction) {
    match action {
        ConfirmAction::CreateBackup => {
            app.do_create_backup();
        }
        ConfirmAction::RestoreBackup => {
            match crate::engine::Engine::restore_backup(&app.config) {
                Ok(()) => {
                    app.refresh_scan();
                    app.status_msg = "Stock files restored.".into();
                    app.dialog = Some(Dialog::Info {
                        title: "Success".into(),
                        message: "Restored stock game files successfully!".into(),
                    });
                }
                Err(e) => {
                    app.dialog = Some(Dialog::Info {
                        title: "Error".into(),
                        message: format!("Failed to restore backup:\n{}", e),
                    });
                }
            }
        }
        ConfirmAction::ClearGlobalOverrides => {
            app.config.data.global_overrides.clear();
            app.config.save();
            app.rebuild_global_fields();
        }
        ConfirmAction::ClearCarOverrides(car_id) => {
            app.config.data.car_overrides.remove(&car_id);
            app.config.save();
            app.loaded_car_id = None;
            app.load_car_fields(&car_id);
        }
        ConfirmAction::ClearSynthOverrides(fname) => {
            app.config.data.synth_overrides.remove(&fname);
            app.config.save();
            app.loaded_synth = None;
            app.load_synth_fields(&fname);
        }
        ConfirmAction::DeleteProfile(name) => {
            app.config.delete_profile(&name);
            if app.config.data.active_profile == name {
                let first = app.config.profiles.keys().next().cloned().unwrap_or_default();
                app.config.data.active_profile = first;
                app.config.save();
            }
            app.selected_profile = None;
            app.status_msg = format!("Profile '{}' deleted.", name);
        }
        ConfirmAction::LoadProfile(name) => {
            let snap = app.config.profiles.get(&name).cloned();
            if let Some(snap) = snap {
                app.config.data.active_profile = name.clone();
                app.config.apply_snapshot(&snap);
                app.reload_all_ui_from_config();
                app.status_msg = format!("Profile '{}' loaded.", name);
            }
        }
        ConfirmAction::SaveToProfile(name) => {
            app.save_misc_to_config();
            let desc = app.config.profiles.get(&name).map(|p| p.description.clone()).unwrap_or_default();
            let created = app.config.profiles.get(&name).map(|p| p.created.clone()).unwrap_or_else(now_iso);
            let mut snap = app.config.snapshot();
            snap.description = desc;
            snap.created = created;
            app.config.save_profile(&name, &snap);
            app.status_msg = format!("Settings saved to profile '{}'.", name);
            app.dialog = Some(Dialog::Info {
                title: "Saved".into(),
                message: format!("Settings saved to profile '{}'.", name),
            });
        }
        ConfirmAction::ApplyProfile(name) => {
            let snap = app.config.profiles.get(&name).cloned();
            if let Some(snap) = snap {
                app.config.data.active_profile = name.clone();
                app.config.apply_snapshot(&snap);
                app.reload_all_ui_from_config();
            }
            match app.apply_changes() {
                Ok(()) => {
                    app.status_msg = format!("Profile '{}' applied.", name);
                    app.dialog = Some(Dialog::Info {
                        title: "Success".into(),
                        message: format!("Profile '{}' applied to game files!", name),
                    });
                }
                Err(e) => {
                    app.dialog = Some(Dialog::Info {
                        title: "Error".into(),
                        message: format!("Failed to apply profile:\n{}", e),
                    });
                }
            }
        }
        ConfirmAction::LoadBundled(name) => {
            let snap = app.bundled_profiles.iter()
                .find(|(n, _)| n == &name)
                .map(|(_, p)| p.clone());
            if let Some(snap) = snap {
                app.config.apply_snapshot(&snap);
                app.reload_all_ui_from_config();
                app.status_msg = format!("Built-in profile '{}' loaded into editor.", name);
            }
        }
        ConfirmAction::ApplyBundled(name) => {
            let snap = app.bundled_profiles.iter()
                .find(|(n, _)| n == &name)
                .map(|(_, p)| p.clone());
            if let Some(snap) = snap {
                app.config.apply_snapshot(&snap);
                app.reload_all_ui_from_config();
            }
            match app.apply_changes() {
                Ok(()) => {
                    app.status_msg = format!("Built-in profile '{}' applied.", name);
                    app.dialog = Some(Dialog::Info {
                        title: "Success".into(),
                        message: format!("Profile '{}' applied to game files!", name),
                    });
                }
                Err(e) => {
                    app.dialog = Some(Dialog::Info {
                        title: "Error".into(),
                        message: format!("Failed to apply profile:\n{}", e),
                    });
                }
            }
        }
    }
}

fn dispatch_text(app: &mut App, action: TextInputAction, value: String) {
    let name = value.trim().to_string();
    if name.is_empty() { return; }

    match action {
        TextInputAction::NewProfile => {
            if app.config.profiles.contains_key(&name) {
                app.dialog = Some(Dialog::Info {
                    title: "Name Taken".into(),
                    message: format!("A profile named '{}' already exists.", name),
                });
                return;
            }
            app.save_misc_to_config();
            let mut snap = app.config.snapshot();
            snap.description = String::new();
            snap.created = now_iso();
            app.config.save_profile(&name, &snap);
            app.selected_profile = Some(name.clone());
            app.profile_name_buf = name.clone();
            app.profile_desc_buf = snap.description;
            app.status_msg = format!("Profile '{}' created.", name);
        }
        TextInputAction::DuplicateProfile(source) => {
            let final_name = if !app.config.profiles.contains_key(&name) {
                name.clone()
            } else {
                let mut n = 2;
                loop {
                    let candidate = format!("{} ({})", name, n);
                    if !app.config.profiles.contains_key(&candidate) { break candidate; }
                    n += 1;
                }
            };
            let mut profile = app.config.profiles.get(&source).cloned().unwrap_or_default();
            profile.created = now_iso();
            profile.description = profile.description.clone();
            app.config.save_profile(&final_name, &profile);
            app.selected_profile = Some(final_name.clone());
            app.profile_name_buf = final_name.clone();
            app.profile_desc_buf = profile.description;
            app.status_msg = format!("Profile '{}' duplicated from '{}'.", final_name, source);
        }
    }
}

// ── Bundled profile loader ────────────────────────────────────────────────────

fn load_bundled_profiles() -> Vec<(String, crate::config::ProfileData)> {
    crate::data::BUNDLED_PROFILES.iter().filter_map(|(name, json)| {
        serde_json::from_str::<crate::config::ProfileData>(json).ok()
            .map(|p| (name.to_string(), p))
    }).collect()
}

// ── Helpers to build initial field lists ──────────────────────────────────────

pub static CAR_FIELD_DEFS: &[(&str, &str, bool)] = &[
    ("RPMScalar_Stock",   "RPMScalar (Stock)",   true),
    ("RPMScalar_Street",  "RPMScalar (Street)",  true),
    ("RPMScalar_Sport",   "RPMScalar (Sport)",   true),
    ("RPMScalar_Race",    "RPMScalar (Race)",    true),
    ("Engine_Stock",      "Engine (Stock)",      false),
    ("Engine_Street",     "Engine (Street)",     false),
    ("Engine_Sport",      "Engine (Sport)",      false),
    ("Engine_Race",       "Engine (Race)",       false),
    ("Exhaust_Stock",     "Exhaust (Stock)",     false),
    ("Exhaust_Street",    "Exhaust (Street)",    false),
    ("Exhaust_Sport",     "Exhaust (Sport)",     false),
    ("Exhaust_Race",      "Exhaust (Race)",      false),
    ("Intake_Stock",      "Intake (Stock)",      false),
    ("Intake_Street",     "Intake (Street)",     false),
    ("Intake_Sport",      "Intake (Sport)",      false),
    ("Intake_Race",       "Intake (Race)",       false),
    ("Turbo_Profile",        "Turbo Profile",        false),
    ("SuperCSC_Profile",     "SuperCSC Profile",     false),
    ("SuperDSC_Profile",     "SuperDSC Profile",     false),
    ("Transmission_Profile", "Transmission Profile", false),
    ("Prop_EngineBank",      "EngineBank",           false),
    ("Prop_Burbles",         "Burbles",              false),
    ("Prop_Backfire",        "Backfire",             false),
    ("Prop_AntiLag",         "AntiLag",              false),
    ("Prop_TurboBOV",        "TurboBOV",             false),
    ("Prop_CentrifugalBOV",  "CentrifugalBOV",       false),
    ("Prop_ThrottleBody",    "ThrottleBody",         false),
    ("Prop_Limiter",         "Limiter",              false),
    ("Prop_GearCrack",       "GearCrack",            false),
];

fn build_global_fields(config: &Config) -> Vec<(String, OverrideField)> {
    let global_ov = &config.data.global_overrides;
    CAR_FIELD_DEFS.iter().map(|(key, _, _)| {
        let ov = global_ov.get(*key);
        (key.to_string(), OverrideField {
            enabled: ov.is_some(),
            value: ov.cloned().unwrap_or_default(),
            is_diff: ov.is_some(),
        })
    }).collect()
}

fn build_reverb_fields(config: &Config) -> Vec<ReverbField> {
    let ov = &config.data.reverb_overrides;
    crate::data::REVERB_FIELDS.iter().map(|name| {
        let mul_enabled = ov.contains_key(*name);
        let off_enabled = ov.contains_key(&format!("{}_offset", name));
        ReverbField {
            name: name.to_string(),
            mul_enabled,
            mul_value: ov.get(*name).copied().unwrap_or(1.0),
            off_enabled,
            off_value: ov.get(&format!("{}_offset", name)).copied().unwrap_or(0.0),
        }
    }).collect()
}

fn setup_visuals(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill           = Color32::from_rgb(30,  30,  36);
    v.window_fill          = Color32::from_rgb(43,  43,  51);
    v.extreme_bg_color     = Color32::from_rgb(21,  21,  26);
    v.faint_bg_color       = Color32::from_rgb(43,  43,  51);
    v.widgets.noninteractive.bg_fill  = Color32::from_rgb(43,  43,  51);
    v.widgets.inactive.bg_fill        = Color32::from_rgb(43,  43,  51);
    v.widgets.hovered.bg_fill         = Color32::from_rgb(76,  76,  86);
    v.widgets.active.bg_fill          = Color32::from_rgb(43,  43,  51);
    v.widgets.open.bg_fill            = Color32::from_rgb(60,  60,  68);
    v.selection.bg_fill               = Color32::from_rgb(58,  58,  69);
    ctx.set_visuals(v);
}

// ── Shared UI helpers used by multiple tabs ───────────────────────────────────

pub const COL_GOLD: Color32     = Color32::from_rgb(212, 175,  55);
pub const COL_GRAY: Color32     = Color32::from_rgb(160, 160, 170);
pub const COL_GREEN: Color32    = Color32::from_rgb(100, 220, 100);
pub const COL_RED: Color32      = Color32::from_rgb(220,  80,  80);
pub const COL_BLUE_HINT: Color32= Color32::from_rgb(104, 136, 170);

/// Combo box with a custom display function for each option.
/// The display function is also called with `""` to get the "(none)" label.
pub fn combo_disp(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut String,
    options: &[String],
    display: impl Fn(&str) -> String,
) -> bool {
    let mut changed = false;
    let selected_text = display(value);
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected_text)
        .width(200.0)
        .show_ui(ui, |ui| {
            if ui.selectable_value(value, String::new(), display("")).changed() { changed = true; }
            for opt in options {
                if ui.selectable_value(value, opt.clone(), display(opt)).changed() { changed = true; }
            }
        });
    changed
}

/// A combo box backed by a `String`. Returns true if the value changed.
pub fn combo(ui: &mut egui::Ui, id: &str, value: &mut String, options: &[String]) -> bool {
    combo_disp(ui, id, value, options, |v| if v.is_empty() { "(none)".to_string() } else { v.to_string() })
}
