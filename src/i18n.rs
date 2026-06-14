use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default, Debug)]
pub enum Language {
    #[default]
    English,
    German,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::German => "Deutsch",
        }
    }
}

/// The label shown for empty/cleared combo selections.
pub fn tr_none(lang: Language) -> &'static str {
    match lang { Language::German => "(kein)", _ => "(none)" }
}

/// Translates a static UI string. Falls back to the key itself.
pub fn tr(lang: Language, key: &'static str) -> &'static str {
    if lang == Language::English { return key; }
    match key {
        // Top bar
        "Apply Mod Profile"              => "Mod-Profil anwenden",
        // Tab names
        "Setup"                          => "Einrichtung",
        "Car Overrides"                  => "Fahrzeug-Overrides",
        "Class Overrides"                => "Klassen-Overrides",
        "Miscellaneous"                  => "Verschiedenes",
        "Reverb"                         => "Nachhall",
        "Profiles"                       => "Profile",
        // Setup tab
        "Folder Selection"               => "Ordnerauswahl",
        "Forza Horizon 6 Directory:"     => "Forza Horizon 6 Verzeichnis:",
        "Browse…"                        => "Durchsuchen…",
        "Backup & Restore"               => "Backup & Wiederherstellung",
        "Create / Refresh Backup"        => "Backup erstellen / aktualisieren",
        "Restore Original Stock Files"   => "Originaldateien wiederherstellen",
        "Usage Instructions"             => "Nutzungsanleitung",
        "Language & Display"             => "Sprache & Darstellung",
        "Language"                       => "Sprache",
        "Translate parameter names"      => "Parameternamen übersetzen",
        "Human-readable values"          => "Lesbare Wertbezeichnungen",
        "Backup Status:"                 => "Backup-Status:",
        "The tool reads from a safe Backup folder ('media/Audio_Backup') and applies \
         modifications to the active game folder. This backup copies only .xml config files, \
         avoiding gigabytes of audio banks."
            => "Das Tool liest aus 'media/Audio_Backup' und wendet Änderungen auf den aktiven \
                Spielordner an. Das Backup kopiert nur .xml-Konfigurationsdateien.",
        "1. Select your ForzaHorizon6 game path above."
            => "1. Wähle deinen ForzaHorizon6 Spielpfad oben aus.",
        "2. Click 'Create/Refresh Backup' to safe-keep original files (required before applying edits)."
            => "2. Klicke auf 'Backup erstellen', um Originaldateien zu sichern (erforderlich).",
        "3. Customize sound profiles globally or per-car under the Car Overrides tab."
            => "3. Passe Soundprofile global oder pro Fahrzeug unter 'Fahrzeug-Overrides' an.",
        "4. Fine-tune class synthesizers under Class Overrides."
            => "4. Verfeinere Klassen-Synthesizer unter 'Klassen-Overrides'.",
        "5. Adjust backfire and blur rim limits under Miscellaneous."
            => "5. Stelle Fehlzündungs- und Felgenunschärfe-Grenzen unter 'Verschiedenes' ein.",
        "6. Click the green 'Apply Mod Profile' button at the top-right to write overrides to the game."
            => "6. Klicke auf 'Mod-Profil anwenden' oben rechts, um Änderungen zu speichern.",
        // Car overrides tab
        "Core Sound Channels"             => "Haupt-Soundkanäle",
        "Induction & Gear Whine Profiles" => "Induktions- & Getriebeprofile",
        "Acoustic Sound Properties"       => "Akustische Klangeigenschaften",
        "Core Synthesizer & RPM Channels" => "Kern-Synthesizer & Drehzahlkanäle",
        "Select a vehicle or [Global Overrides] from the list."
            => "Fahrzeug oder [Globale Overrides] aus der Liste wählen.",
        "Clear Overrides for This Car"   => "Overrides für dieses Fahrzeug löschen",
        "Clear All Global Overrides"     => "Alle globalen Overrides löschen",
        "Search cars…"                   => "Fahrzeuge suchen…",
        // Class overrides tab
        "Channel Volume & RPM Settings"  => "Kanal-Lautstärke & Drehzahl",
        "Gear Crack Settings"            => "Schaltknacken-Einstellungen",
        "Advanced Attributes"            => "Erweiterte Attribute",
        "Select a synthesizer file from the list to modify its parameters."
            => "Synthesizerdatei aus der Liste wählen, um Parameter anzupassen.",
        "Clear Overrides for This Synth" => "Overrides für diesen Synthesizer löschen",
        "Search synthesizers…"           => "Synthesizer suchen…",
        "(global default)"               => "(globaler Standard)",
        // Shared column headers
        "Parameter"                      => "Parameter",
        "Stock Value"                    => "Serienwert",
        "Override?"                      => "Überschreiben?",
        "Override"                       => "Überschreiben",
        "Value"                          => "Wert",
        "New Value"                      => "Neuer Wert",
        "Link"                           => "Link",
        "Attribute"                      => "Attribut",
        "Stock: "                        => "Serien: ",
        // Reverb tab
        "Global Reverb Multipliers"      => "Globale Nachhall-Multiplikatoren",
        "Mul?"                           => "Mult.?",
        "Multiplier (1.0 = stock)"       => "Multiplikator (1.0 = Serien)",
        "Off?"                           => "Offset?",
        "Offset (+/−)"                   => "Offset (+/−)",
        // Misc tab
        "Global Vehicle Attributes Override" => "Globale Fahrzeugattribut-Overrides",
        "BlurRim Rotation Limit (MaxRPM)" => "Felgen-Unschärfe-Rotationslimit (MaxRPM)",
        "Backfire Minimum Delay (ms)"    => "Fehlzündung Min. Verzögerung (ms)",
        "Backfire Maximum Delay (ms)"    => "Fehlzündung Max. Verzögerung (ms)",
        // Setup wizard
        "Quick Start Guide" => "Schnellstartanleitung",
        "Back"        => "Zurück",
        "Next"        => "Weiter",
        "Finish"      => "Fertig",
        "Skip"        => "Überspringen",
        // Dialogs
        "Yes"    => "Ja",
        "No"     => "Nein",
        "OK"     => "OK",
        "Cancel" => "Abbrechen",
        // Profiles tab
        "Open Profiles Folder" => "Profilordner öffnen",
        _ => key,
    }
}

/// Translates a synthesizer attribute name (e.g. "MasterVolume") for the class tab.
/// Returns a borrowed str so it works with runtime (non-static) strings.
pub fn tr_attr<'a>(lang: Language, name: &'a str) -> &'a str {
    if lang != Language::German { return name; }
    match name {
        "MasterVolume" => "Hauptlautstärke",
        "MinRPM"       => "Min. Drehzahl",
        "MaxRPM"       => "Max. Drehzahl",
        "Volume"       => "Lautstärke",
        _ => name,
    }
}

/// Translates dynamic strings (category names, descriptions) that aren't compile-time literals.
/// Returns a borrowed str so it works with runtime String values.
pub fn tr_dyn<'a>(lang: Language, key: &'a str) -> &'a str {
    if lang != Language::German { return key; }
    match key {
        // Global category names (used as headings and list labels)
        "[Global Engine]"        => "[Globaler Motor]",
        "[Global Exhaust]"       => "[Globaler Auspuff]",
        "[Global Intake]"        => "[Globale Ansaugung]",
        "[Global Transmission]"  => "[Globales Getriebe]",
        "[Global Turbo]"         => "[Globaler Turbo]",
        "[Global Supercharger]"  => "[Globaler Kompressor]",
        // Global category descriptions
        "Sets Channel volume/RPM for ALL engine synth files."
            => "Stellt Kanal-Lautstärke/Drehzahl für ALLE Motor-Synthesizer ein.",
        "Sets Channel volume/RPM for ALL exhaust synth files."
            => "Stellt Kanal-Lautstärke/Drehzahl für ALLE Auspuff-Synthesizer ein.",
        "Sets Channel volume/RPM for ALL intake synth files."
            => "Stellt Kanal-Lautstärke/Drehzahl für ALLE Ansaugungs-Synthesizer ein.",
        "Sets Channel volume/RPM for ALL transmission synth files."
            => "Stellt Kanal-Lautstärke/Drehzahl für ALLE Getriebe-Synthesizer ein.",
        "Sets Channel volume/RPM for ALL turbocharger synth files."
            => "Stellt Kanal-Lautstärke/Drehzahl für ALLE Turbo-Synthesizer ein.",
        "Sets Channel volume/RPM for ALL supercharger synth files."
            => "Stellt Kanal-Lautstärke/Drehzahl für ALLE Kompressor-Synthesizer ein.",
        _ => key,
    }
}

/// Translates a parameter display name (e.g. "Engine (Stock)") when translate_params is on.
pub fn tr_param(lang: Language, name: &'static str) -> &'static str {
    if lang != Language::German { return name; }
    match name {
        "RPMScalar (Stock)"    => "Drehzahlskalierung (Serien)",
        "RPMScalar (Street)"   => "Drehzahlskalierung (Street)",
        "RPMScalar (Sport)"    => "Drehzahlskalierung (Sport)",
        "RPMScalar (Race)"     => "Drehzahlskalierung (Rennen)",
        "Engine (Stock)"       => "Motor (Serien)",
        "Engine (Street)"      => "Motor (Street)",
        "Engine (Sport)"       => "Motor (Sport)",
        "Engine (Race)"        => "Motor (Rennen)",
        "Exhaust (Stock)"      => "Auspuff (Serien)",
        "Exhaust (Street)"     => "Auspuff (Street)",
        "Exhaust (Sport)"      => "Auspuff (Sport)",
        "Exhaust (Race)"       => "Auspuff (Rennen)",
        "Intake (Stock)"       => "Ansaugung (Serien)",
        "Intake (Street)"      => "Ansaugung (Street)",
        "Intake (Sport)"       => "Ansaugung (Sport)",
        "Intake (Race)"        => "Ansaugung (Rennen)",
        "Turbo Profile"        => "Turbo-Profil",
        "SuperCSC Profile"     => "Kompressorprofil (CSC)",
        "SuperDSC Profile"     => "Kompressorprofil (DSC)",
        "Transmission Profile" => "Getriebeprofil",
        "EngineBank"           => "Motorbank",
        "Burbles"              => "Blubbern",
        "Backfire"             => "Fehlzündung",
        "AntiLag"              => "Anti-Lag",
        "TurboBOV"             => "Turbo-Druckventil",
        "CentrifugalBOV"       => "Zentrifugal-Druckventil",
        "ThrottleBody"         => "Drosselklappe",
        "Limiter"              => "Begrenzer",
        "GearCrack"            => "Schaltknacken",
        "MasterVolume"         => "Hauptlautstärke",
        "MinRPM"               => "Min. Drehzahl",
        "MaxRPM"               => "Max. Drehzahl",
        "Volume"               => "Lautstärke",
        _ => name,
    }
}

/// Returns a human-readable display string for a raw game option value.
pub fn humanize(lang: Language, raw: &str) -> String {
    match raw {
        "" => return match lang { Language::German => "(kein)".to_string(), _ => "(none)".to_string() },
        "None" => return match lang { Language::German => "Kein".to_string(), _ => "None".to_string() },
        "E0" => return match lang { Language::German => "Elektrisch".to_string(), _ => "Electric".to_string() },
        "Stock" => return match lang { Language::German => "Serienmäßig".to_string(), _ => "Stock".to_string() },
        "Manual" => return match lang { Language::German => "Schaltgetriebe".to_string(), _ => "Manual".to_string() },
        "DCT" => return match lang { Language::German => "Doppelkupplung".to_string(), _ => "DCT".to_string() },
        "DSG" => return "DSG".to_string(),
        "Sequential" | "GearCrack_Sequential" => return match lang { Language::German => "Sequenziell".to_string(), _ => "Sequential".to_string() },
        "JDM" => return "JDM".to_string(),
        "Diesel" => return "Diesel".to_string(),
        "Scallop" => return "Scallop".to_string(),
        "GS_ModularCar"     => return match lang { Language::German => "Modular-Auto".to_string(),       _ => "Modular Car".to_string() },
        "GS_ModularCar_ALT" => return match lang { Language::German => "Modular-Auto (Alt)".to_string(), _ => "Modular Car (Alt)".to_string() },
        "I1" => return inline_n(lang, 1), "I2" => return inline_n(lang, 2),
        "I3" => return inline_n(lang, 3), "I4" => return inline_n(lang, 4),
        "I5" => return inline_n(lang, 5), "I6" => return inline_n(lang, 6),
        "I8" => return inline_n(lang, 8),
        "V4" => return "V4".to_string(),  "V6"  => return "V6".to_string(),
        "V8" => return "V8".to_string(),  "V10" => return "V10".to_string(),
        "V12" => return "V12".to_string(),
        "F2"  => return flat_n(lang, 2),  "F4"  => return flat_n(lang, 4),
        "F6"  => return flat_n(lang, 6),  "F12" => return flat_n(lang, 12),
        "W12" => return "W12".to_string(), "W16" => return "W16".to_string(),
        "Rally"  => return match lang { Language::German => "Rallye".to_string(),      _ => "Rally".to_string() },
        "Rotary" => return match lang { Language::German => "Kreiskolben".to_string(), _ => "Rotary".to_string() },
        "Race"   => return match lang { Language::German => "Rennen".to_string(),      _ => "Race".to_string() },
        _ => {}
    }

    if let Some(rest) = raw.strip_prefix("Limiter_") {
        return match lang {
            Language::German => format!("{} Begrenzer", era_de(rest)),
            _ => format!("{} Limiter", rest),
        };
    }
    if let Some(rest) = raw.strip_prefix("ThrottleBody_") {
        return match lang {
            Language::German => format!("{} Drosselklappe", era_de(rest)),
            _ => format!("{} Throttle", rest),
        };
    }
    if let Some(rest) = raw.strip_prefix("Electric_") {
        let type_s = if lang == Language::German { car_type_de(rest) } else { split_camel(rest) };
        return match lang {
            Language::German => format!("Elektro-{}", type_s),
            _ => format!("Electric {}", type_s),
        };
    }

    parse_compound(lang, raw)
}

fn inline_n(lang: Language, n: u32) -> String {
    match lang { Language::German => format!("Reihen-{}", n), _ => format!("Inline {}", n) }
}

fn flat_n(lang: Language, n: u32) -> String {
    match lang { Language::German => format!("Boxer {}", n), _ => format!("Flat {}", n) }
}

fn era_de(era: &str) -> &str {
    match era { "Modern" => "Modern", "Classic" => "Klassik", "Vintage" => "Oldtimer", other => other }
}

fn car_type_de(t: &str) -> String {
    match t {
        "Muscle" | "MuscleALT" => "Muscle".to_string(),
        "Offroad"    => "Geländewagen".to_string(),
        "SportsCar"  => "Sportwagen".to_string(),
        "SUV"        => "SUV".to_string(),
        "Saloon"     => "Limousine".to_string(),
        "Supercar"   => "Supersportwagen".to_string(),
        "Hypercar"   => "Hypersportwagen".to_string(),
        "TrackToy"   => "Rennstrecken-Auto".to_string(),
        "Truck" | "TruckALT" => "LKW".to_string(),
        "Van"        => "Transporter".to_string(),
        "Classic"    => "Klassiker".to_string(),
        "HotRod"     => "Hot Rod".to_string(),
        "Race"       => "Rennwagen".to_string(),
        "Rally"      => "Rallye".to_string(),
        "HotHatch"   => "Schrägheck".to_string(),
        "Kei"        => "Kei-Auto".to_string(),
        "Buggy"      => "Buggy".to_string(),
        _            => split_camel(t),
    }
}

fn car_type_en(t: &str) -> String {
    match t {
        "SportsCar"  => "Sports Car".to_string(),
        "TrackToy"   => "Track Toy".to_string(),
        "HotHatch"   => "Hot Hatch".to_string(),
        "HotRod"     => "Hot Rod".to_string(),
        "TruckALT"   => "Truck (Alt)".to_string(),
        "MuscleALT"  => "Muscle (Alt)".to_string(),
        _            => split_camel(t),
    }
}

fn parse_compound(lang: Language, raw: &str) -> String {
    let (era, rest) = if raw.starts_with("Modern") {
        ("Modern", &raw[6..])
    } else if raw.starts_with("Classic") {
        ("Classic", &raw[7..])
    } else if raw.starts_with("Vintage") {
        ("Vintage", &raw[7..])
    } else {
        return if lang == Language::German { car_type_de(raw) } else { car_type_en(raw) };
    };

    let era_s = if lang == Language::German { era_de(era) } else { era };
    let (cyl_s, type_s) = split_cyl_type(lang, rest);

    if cyl_s.is_empty() {
        format!("{} {}", era_s, type_s)
    } else {
        format!("{} {} {}", era_s, cyl_s, type_s)
    }
}

fn split_cyl_type(lang: Language, s: &str) -> (String, String) {
    if let Some(after_rotary) = s.strip_prefix("Rotary") {
        let num_end = after_rotary.find(|c: char| !c.is_ascii_digit()).unwrap_or(after_rotary.len());
        let (num_s, type_s) = (&after_rotary[..num_end], &after_rotary[num_end..]);
        let cyl = match lang { Language::German => format!("Kreiskolben {}", num_s), _ => format!("Rotary {}", num_s) };
        let type_out = if lang == Language::German { car_type_de(type_s) } else { car_type_en(type_s) };
        return (cyl, type_out);
    }

    if let Some(first) = s.chars().next() {
        if matches!(first, 'V' | 'I' | 'F' | 'W') {
            let digits = &s[1..];
            let num_end = digits.find(|c: char| !c.is_ascii_digit()).unwrap_or(digits.len());
            if num_end > 0 {
                let (num_s, type_s) = (&digits[..num_end], &digits[num_end..]);
                let cyl = match first {
                    'I' => match lang { Language::German => format!("Reihen-{}", num_s), _ => format!("Inline-{}", num_s) },
                    'F' => match lang { Language::German => format!("Boxer-{}", num_s),  _ => format!("Flat-{}", num_s) },
                    _   => format!("{}{}", first, num_s),
                };
                let type_out = if lang == Language::German { car_type_de(type_s) } else { car_type_en(type_s) };
                return (cyl, type_out);
            }
        }
    }

    let type_out = if lang == Language::German { car_type_de(s) } else { car_type_en(s) };
    (String::new(), type_out)
}

fn split_camel(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.char_indices() {
        if i > 0 && c.is_uppercase() { out.push(' '); }
        out.push(c);
    }
    out
}
