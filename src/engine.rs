use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use xmltree::{Element, XMLNode};

use crate::config::Config;

#[derive(Clone)]
pub struct CarEntry {
    pub id: String,
    pub stock_values: HashMap<String, String>,
}

#[derive(Default)]
pub struct ScrapedOptions {
    pub engine: Vec<String>,
    pub exhaust: Vec<String>,
    pub intake: Vec<String>,
}

pub struct Engine;

impl Engine {
    pub fn audio_paths(cfg: &Config) -> (PathBuf, PathBuf) {
        let base = PathBuf::from(&cfg.data.game_path).join("media").join("Audio");
        let backup = PathBuf::from(&cfg.data.game_path).join("media").join("Audio_Backup");
        (base, backup)
    }

    pub fn misc_paths(cfg: &Config) -> (PathBuf, PathBuf) {
        let base = PathBuf::from(&cfg.data.game_path)
            .join("media").join("Cars").join("_library").join("GlobalCarAttributes.xml");
        let backup = PathBuf::from(&cfg.data.game_path)
            .join("media").join("Cars").join("_library").join("GlobalCarAttributes_Backup.xml");
        (base, backup)
    }

    pub fn check_backup(cfg: &Config) -> (bool, String) {
        let (audio, backup) = Self::audio_paths(cfg);
        if !audio.exists() {
            return (false, "ForzaHorizon6/media/Audio folder not found at specified path!".into());
        }
        if !backup.exists() {
            return (false, "Backup folder (Audio_Backup) not found.".into());
        }
        let count = walk_xml_count(&backup);
        if count == 0 {
            return (false, "Backup folder is empty.".into());
        }
        (true, format!("Backup complete ({} XML configuration files backed up).", count))
    }

    pub fn create_backup(cfg: &Config, progress: &mut impl FnMut(u32)) -> Result<(), String> {
        let (audio, backup) = Self::audio_paths(cfg);
        if !audio.exists() {
            return Err(format!("Audio folder not found at {}", audio.display()));
        }
        if backup.exists() {
            std::fs::remove_dir_all(&backup).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&backup).map_err(|e| e.to_string())?;

        let all_xmls: Vec<(PathBuf, PathBuf)> = collect_xml_pairs(&audio, &backup);
        let total = all_xmls.len().max(1);

        for (i, (src, dst)) in all_xmls.iter().enumerate() {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::copy(src, dst).map_err(|e| format!("{}: {}", src.display(), e))?;
            progress(((i + 1) * 100 / total) as u32);
        }

        let (misc, misc_bak) = Self::misc_paths(cfg);
        if misc.exists() && !misc_bak.exists() {
            std::fs::copy(&misc, &misc_bak).ok();
        }
        Ok(())
    }

    pub fn restore_backup(cfg: &Config) -> Result<(), String> {
        let (audio, backup) = Self::audio_paths(cfg);
        if !backup.exists() {
            return Err("Backup folder not found! Cannot restore.".into());
        }
        for (src, dst) in collect_xml_pairs(&backup, &audio) {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::copy(&src, &dst).map_err(|e| format!("{}: {}", src.display(), e))?;
        }
        let (misc, misc_bak) = Self::misc_paths(cfg);
        if misc_bak.exists() {
            std::fs::copy(&misc_bak, &misc).ok();
        }
        Ok(())
    }

    pub fn scan_cars(cfg: &Config) -> (Vec<CarEntry>, ScrapedOptions) {
        let (_, backup) = Self::audio_paths(cfg);
        let mc_dir = backup.join("ModularCars");
        let mut cars = Vec::new();
        let mut scraped = ScrapedOptions::default();
        let mut eng_set = HashSet::new();
        let mut exh_set = HashSet::new();
        let mut int_set = HashSet::new();

        if let Ok(entries) = std::fs::read_dir(&mc_dir) {
            let mut files: Vec<_> = entries.flatten().collect();
            files.sort_by_key(|e| e.file_name());
            for entry in files {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("-Engine.xml") {
                    let car_id = name.trim_end_matches("-Engine.xml").to_string();
                    if let Ok(data) = std::fs::read_to_string(&path) {
                        if let Ok(root) = Element::parse(data.as_bytes()) {
                            let sv = extract_values(&root);
                            for level in &["Stock", "Street", "Sport", "Race"] {
                                if let Some(v) = sv.get(&format!("Engine_{}", level)) { eng_set.insert(v.clone()); }
                                if let Some(v) = sv.get(&format!("Exhaust_{}", level)) { exh_set.insert(v.clone()); }
                                if let Some(v) = sv.get(&format!("Intake_{}", level))  { int_set.insert(v.clone()); }
                            }
                            cars.push(CarEntry { id: car_id, stock_values: sv });
                        }
                    }
                }
            }
        }

        let mut ev: Vec<_> = eng_set.into_iter().collect(); ev.sort();
        let mut xv: Vec<_> = exh_set.into_iter().collect(); xv.sort();
        let mut iv: Vec<_> = int_set.into_iter().collect(); iv.sort();
        scraped.engine = ev;
        scraped.exhaust = xv;
        scraped.intake = iv;
        (cars, scraped)
    }

    pub fn scan_synths(cfg: &Config) -> Vec<String> {
        let (_, backup) = Self::audio_paths(cfg);
        let dir = backup.join("EngineSynth");
        let mut list = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut files: Vec<_> = entries.flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("xml"))
                .collect();
            files.sort_by_key(|e| e.file_name());
            for entry in files {
                list.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        list
    }

    pub fn load_synth_values(cfg: &Config, filename: &str) -> HashMap<String, HashMap<String, String>> {
        let (_, backup) = Self::audio_paths(cfg);
        let path = backup.join("EngineSynth").join(filename);
        let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(root) = Element::parse(data.as_bytes()) {
                if let Some(ch) = find_child(&root, "Channel") {
                    result.insert("Channel".into(), ch.attributes.clone());
                }
                if let Some(gc) = find_child(&root, "GearCrack") {
                    result.insert("GearCrack".into(), gc.attributes.clone());
                }
            }
        }
        result
    }

    pub fn read_misc_stock(cfg: &Config) -> HashMap<String, String> {
        let (misc, misc_bak) = Self::misc_paths(cfg);
        let target = if misc_bak.exists() { misc_bak } else { misc };
        let mut vals = HashMap::from([
            ("blur_rim_max_rpm".into(), "350.0".into()),
            ("backfire_min_rand_time".into(), "100.0".into()),
            ("backfire_max_rand_time".into(), "350.0".into()),
        ]);
        if let Ok(text) = std::fs::read_to_string(&target) {
            use regex::Regex;
            if let Ok(re) = Regex::new(r#"<BlurRim\s+[^>]*?MaxRPM="([^"]*?)""#) {
                if let Some(c) = re.captures(&text) { vals.insert("blur_rim_max_rpm".into(), c[1].to_string()); }
            }
            if let Ok(re) = Regex::new(r#"<Effect\s+[^>]*?BackfireMinRandTime="([^"]*?)""#) {
                if let Some(c) = re.captures(&text) { vals.insert("backfire_min_rand_time".into(), c[1].to_string()); }
            }
            if let Ok(re) = Regex::new(r#"<Effect\s+[^>]*?BackfireMaxRandTime="([^"]*?)""#) {
                if let Some(c) = re.captures(&text) { vals.insert("backfire_max_rand_time".into(), c[1].to_string()); }
            }
        }
        vals
    }

    pub fn apply_overrides(cfg: &Config, progress: &mut impl FnMut(u32)) -> Result<(), String> {
        let (audio, backup) = Self::audio_paths(cfg);
        if !backup.exists() {
            return Err("Backup folder not found! Perform backup first.".into());
        }

        let mc_backup = backup.join("ModularCars");
        let mc_active = audio.join("ModularCars");
        std::fs::create_dir_all(&mc_active).map_err(|e| e.to_string())?;

        let global_ov = &cfg.data.global_overrides;
        let car_ov_db = &cfg.data.car_overrides;

        let car_files: Vec<_> = std::fs::read_dir(&mc_backup)
            .map_err(|e| e.to_string())?
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with("-Engine.xml"))
            .collect();
        let total = car_files.len().max(1);

        for (idx, entry) in car_files.iter().enumerate() {
            let fname = entry.file_name().to_string_lossy().to_string();
            let car_id = fname.trim_end_matches("-Engine.xml");
            let src = mc_backup.join(&fname);
            let dst = mc_active.join(&fname);

            let mut combined: HashMap<String, String> = global_ov.clone();
            if let Some(car_map) = car_ov_db.get(car_id) {
                for (k, v) in car_map {
                    combined.insert(k.clone(), v.clone());
                }
            }

            if let Ok(data) = std::fs::read_to_string(&src) {
                if let Ok(mut root) = Element::parse(data.as_bytes()) {
                    if !combined.is_empty() {
                        apply_values(&mut root, &combined);
                    }
                    write_xml(&root, &dst)?;
                } else {
                    std::fs::copy(&src, &dst).ok();
                }
            }
            progress(((idx + 1) * 80 / total) as u32);
        }

        // Copy all other XMLs, applying synth/reverb overrides
        let other_pairs = collect_other_xml_pairs(&backup, &audio);
        let total_other = other_pairs.len().max(1);
        let synth_ov = &cfg.data.synth_overrides;
        let reverb_ov = &cfg.data.reverb_overrides;

        for (idx, (src, dst)) in other_pairs.iter().enumerate() {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let fname = src.file_name().unwrap_or_default().to_string_lossy().to_string();
            let is_synth = src.to_string_lossy().contains("EngineSynth") && fname.ends_with(".xml");
            let is_reverb = fname == "Reverb_Templates.xml";

            if is_synth {
                let category = synth_category(&fname);
                let mut combined: HashMap<String, HashMap<String, String>> = HashMap::new();
                if let Some(cat) = category {
                    if let Some(cat_ov) = synth_ov.get(cat) {
                        for (elem, attrs) in cat_ov {
                            combined.insert(elem.clone(), attrs.clone());
                        }
                    }
                }
                if let Some(file_ov) = synth_ov.get(&fname) {
                    for (elem, attrs) in file_ov {
                        let entry = combined.entry(elem.clone()).or_default();
                        for (a, v) in attrs {
                            entry.insert(a.clone(), v.clone());
                        }
                    }
                }
                if combined.is_empty() {
                    std::fs::copy(&src, &dst).ok();
                } else if let Ok(data) = std::fs::read_to_string(&src) {
                    if let Ok(mut root) = Element::parse(data.as_bytes()) {
                        apply_synth_overrides(&mut root, &combined);
                        if write_xml(&root, &dst).is_err() {
                            std::fs::copy(&src, &dst).ok();
                        }
                    } else {
                        std::fs::copy(&src, &dst).ok();
                    }
                }
            } else if is_reverb && !reverb_ov.is_empty() {
                if let Ok(data) = std::fs::read_to_string(&src) {
                    if let Ok(mut root) = Element::parse(data.as_bytes()) {
                        apply_reverb_overrides(&mut root, reverb_ov);
                        if write_xml(&root, &dst).is_err() {
                            std::fs::copy(&src, &dst).ok();
                        }
                    } else {
                        std::fs::copy(&src, &dst).ok();
                    }
                }
            } else {
                std::fs::copy(&src, &dst).ok();
            }
            progress(80 + ((idx + 1) * 15 / total_other) as u32);
        }

        // Apply misc overrides to GlobalCarAttributes.xml
        let (misc, misc_bak) = Self::misc_paths(cfg);
        if misc_bak.exists() {
            apply_misc_overrides(&misc_bak, &misc, &cfg.data.misc_overrides).ok();
        }

        progress(100);
        Ok(())
    }
}

// ── XML helpers ───────────────────────────────────────────────────────────────

pub fn extract_values(root: &Element) -> HashMap<String, String> {
    let mut vals = HashMap::new();
    if let Some(ge) = find_child(root, "GranularEngine") {
        if let Some(param) = find_child_with_attr(ge, "Parameter", "Name", "RPMScalar") {
            for lv in &["Stock", "Street", "Sport", "Race"] {
                if let Some(v) = param.attributes.get(*lv) {
                    vals.insert(format!("RPMScalar_{}", lv), v.clone());
                }
            }
        }
        for ch_name in &["Engine", "Exhaust", "Intake"] {
            if let Some(ch) = find_child_with_attr(ge, "Channel", "Name", ch_name) {
                for lv in &["Stock", "Street", "Sport", "Race"] {
                    if let Some(v) = ch.attributes.get(*lv) {
                        vals.insert(format!("{}_{}", ch_name, lv), v.clone());
                    }
                }
            }
        }
        for ch_name in &["Turbo", "SuperCSC", "SuperDSC", "Transmission"] {
            if let Some(ch) = find_child_with_attr(ge, "Channel", "Name", ch_name) {
                if let Some(v) = ch.attributes.get("Profile") {
                    vals.insert(format!("{}_Profile", ch_name), v.clone());
                }
            }
        }
    }
    if let Some(props) = find_child(root, "Properties") {
        for child in &props.children {
            if let XMLNode::Element(ref p) = child {
                if p.name == "Property" {
                    if let (Some(n), Some(v)) = (p.attributes.get("Name"), p.attributes.get("Value")) {
                        vals.insert(format!("Prop_{}", n), v.clone());
                    }
                }
            }
        }
    }
    vals
}

fn apply_values(root: &mut Element, overrides: &HashMap<String, String>) {
    if let Some(ge) = find_child_mut(root, "GranularEngine") {
        // RPMScalar
        let has_rpm = overrides.keys().any(|k| k.starts_with("RPMScalar_"));
        if has_rpm {
            let param = find_or_create_child_with_attr(ge, "Parameter", "Name", "RPMScalar");
            for lv in &["Stock", "Street", "Sport", "Race"] {
                if let Some(v) = overrides.get(&format!("RPMScalar_{}", lv)) {
                    param.attributes.insert(lv.to_string(), v.clone());
                }
            }
        }
        // Engine/Exhaust/Intake channels
        for ch_name in &["Engine", "Exhaust", "Intake"] {
            let has_ch = overrides.keys().any(|k| k.starts_with(&format!("{}_", ch_name)));
            if has_ch {
                let ch = find_or_create_child_with_attr(ge, "Channel", "Name", ch_name);
                for lv in &["Stock", "Street", "Sport", "Race"] {
                    if let Some(v) = overrides.get(&format!("{}_{}", ch_name, lv)) {
                        ch.attributes.insert(lv.to_string(), v.clone());
                    }
                }
            }
        }
        // Profile channels
        for ch_name in &["Turbo", "SuperCSC", "SuperDSC", "Transmission"] {
            if let Some(v) = overrides.get(&format!("{}_Profile", ch_name)) {
                let ch = find_or_create_child_with_attr(ge, "Channel", "Name", ch_name);
                ch.attributes.insert("Profile".into(), v.clone());
            }
        }
    }
    // Properties
    let prop_keys: Vec<_> = overrides.keys().filter(|k| k.starts_with("Prop_")).cloned().collect();
    if !prop_keys.is_empty() {
        let props = find_or_create_child(root, "Properties");
        for key in prop_keys {
            let prop_name = &key[5..]; // strip "Prop_"
            if let Some(v) = overrides.get(&key) {
                let prop = find_or_create_child_with_attr(props, "Property", "Name", prop_name);
                prop.attributes.insert("Value".into(), v.clone());
            }
        }
    }
}

fn apply_synth_overrides(root: &mut Element, overrides: &HashMap<String, HashMap<String, String>>) {
    if let Some(attrs) = overrides.get("Channel") {
        if let Some(ch) = find_child_mut(root, "Channel") {
            for (k, v) in attrs { ch.attributes.insert(k.clone(), v.clone()); }
        }
    }
    if let Some(attrs) = overrides.get("GearCrack") {
        if let Some(gc) = find_child_mut(root, "GearCrack") {
            for (k, v) in attrs { gc.attributes.insert(k.clone(), v.clone()); }
        }
    }
}

fn apply_reverb_overrides(root: &mut Element, overrides: &HashMap<String, f64>) {
    apply_reverb_recursive(root, overrides);
}

fn apply_reverb_recursive(elem: &mut Element, overrides: &HashMap<String, f64>) {
    if elem.name == "Template" {
        let keys: Vec<String> = elem.attributes.keys().cloned().collect();
        for attr in keys {
            let mul = overrides.get(&attr).copied();
            let off = overrides.get(&format!("{}_offset", attr)).copied();
            if mul.is_some() || off.is_some() {
                if let Some(orig_str) = elem.attributes.get(&attr).cloned() {
                    if let Ok(v) = orig_str.parse::<f64>() {
                        let mut nv = v;
                        if let Some(m) = mul { nv *= m; }
                        if let Some(o) = off { nv += o; }
                        let formatted = if orig_str.contains('.') {
                            format!("{:.6}", nv)
                        } else {
                            format!("{}", nv.round() as i64)
                        };
                        elem.attributes.insert(attr, formatted);
                    }
                }
            }
        }
    }
    for child in &mut elem.children {
        if let XMLNode::Element(ref mut e) = child {
            apply_reverb_recursive(e, overrides);
        }
    }
}

fn apply_misc_overrides(src: &Path, dst: &Path, overrides: &HashMap<String, String>) -> Result<(), String> {
    let mut text = std::fs::read_to_string(src).map_err(|e| e.to_string())?;
    use regex::Regex;
    if let Some(v) = overrides.get("blur_rim_max_rpm") {
        if let Ok(re) = Regex::new(r#"(<BlurRim\s+[^>]*?MaxRPM=")([^"]*?)("[^>]*?>)"#) {
            text = re.replace(&text, format!("${{1}}{}{}", v, "$3")).to_string();
        }
    }
    if let Ok(re) = Regex::new(r#"<Effect\s+[^>]*?>"#) {
        if let Some(m) = re.find(&text) {
            let mut tag = m.as_str().to_string();
            if let Some(v) = overrides.get("backfire_min_rand_time") {
                if let Ok(re2) = Regex::new(r#"(BackfireMinRandTime=")([^"]*?)(")"#) {
                    tag = re2.replace(&tag, format!("${{1}}{}{}", v, "$3")).to_string();
                }
            }
            if let Some(v) = overrides.get("backfire_max_rand_time") {
                if let Ok(re2) = Regex::new(r#"(BackfireMaxRandTime=")([^"]*?)(")"#) {
                    tag = re2.replace(&tag, format!("${{1}}{}{}", v, "$3")).to_string();
                }
            }
            text = text.replace(m.as_str(), &tag);
        }
    }
    if let Some(parent) = dst.parent() { std::fs::create_dir_all(parent).ok(); }
    std::fs::write(dst, text).map_err(|e| e.to_string())
}

fn write_xml(root: &Element, path: &Path) -> Result<(), String> {
    let cfg = xml::EmitterConfig::new()
        .write_document_declaration(true)
        .perform_indent(false);
    let mut buf = Vec::new();
    root.write_with_config(&mut buf, cfg).map_err(|e| e.to_string())?;
    std::fs::write(path, buf).map_err(|e| e.to_string())
}

fn synth_category(fname: &str) -> Option<&'static str> {
    if fname.ends_with("_Eng.xml")        { Some("[Global Engine]") }
    else if fname.ends_with("_Exh.xml")   { Some("[Global Exhaust]") }
    else if fname.ends_with("_Int.xml")   { Some("[Global Intake]") }
    else if fname.ends_with("_Trn.xml")   { Some("[Global Transmission]") }
    else if fname.ends_with("_Tbo.xml")   { Some("[Global Turbo]") }
    else if fname.ends_with("_CSC.xml") || fname.ends_with("_DSC.xml") { Some("[Global Supercharger]") }
    else { None }
}

// ── Tree navigation helpers ───────────────────────────────────────────────────

pub fn find_child<'a>(elem: &'a Element, name: &str) -> Option<&'a Element> {
    for child in &elem.children {
        if let XMLNode::Element(ref e) = child {
            if e.name == name { return Some(e); }
        }
    }
    None
}

pub fn find_child_with_attr<'a>(elem: &'a Element, tag: &str, attr: &str, val: &str) -> Option<&'a Element> {
    for child in &elem.children {
        if let XMLNode::Element(ref e) = child {
            if e.name == tag && e.attributes.get(attr).map_or(false, |v| v == val) {
                return Some(e);
            }
        }
    }
    None
}

fn find_child_mut<'a>(elem: &'a mut Element, name: &str) -> Option<&'a mut Element> {
    for child in &mut elem.children {
        match child {
            XMLNode::Element(ref mut e) if e.name == name => return Some(e),
            _ => {}
        }
    }
    None
}

fn find_or_create_child<'a>(elem: &'a mut Element, name: &str) -> &'a mut Element {
    let pos = elem.children.iter().position(|c| {
        matches!(c, XMLNode::Element(e) if e.name == name)
    });
    if pos.is_none() {
        elem.children.push(XMLNode::Element(Element::new(name)));
    }
    let p = elem.children.iter().position(|c| {
        matches!(c, XMLNode::Element(e) if e.name == name)
    }).unwrap();
    match &mut elem.children[p] {
        XMLNode::Element(ref mut e) => e,
        _ => unreachable!(),
    }
}

fn find_or_create_child_with_attr<'a>(elem: &'a mut Element, tag: &str, attr: &str, val: &str) -> &'a mut Element {
    let pos = elem.children.iter().position(|c| {
        matches!(c, XMLNode::Element(e) if e.name == tag && e.attributes.get(attr).map_or(false, |v| v == val))
    });
    if pos.is_none() {
        let mut new_elem = Element::new(tag);
        new_elem.attributes.insert(attr.to_string(), val.to_string());
        elem.children.push(XMLNode::Element(new_elem));
    }
    let p = elem.children.iter().position(|c| {
        matches!(c, XMLNode::Element(e) if e.name == tag && e.attributes.get(attr).map_or(false, |v| v == val))
    }).unwrap();
    match &mut elem.children[p] {
        XMLNode::Element(ref mut e) => e,
        _ => unreachable!(),
    }
}

// ── Filesystem helpers ────────────────────────────────────────────────────────

fn walk_xml_count(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() { count += walk_xml_count(&p); }
            else if p.extension().and_then(|e| e.to_str()) == Some("xml") { count += 1; }
        }
    }
    count
}

fn collect_xml_pairs(src_base: &Path, dst_base: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();
    collect_xml_pairs_recursive(src_base, src_base, dst_base, &mut pairs);
    pairs
}

fn collect_xml_pairs_recursive(base: &Path, dir: &Path, dst_base: &Path, out: &mut Vec<(PathBuf, PathBuf)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_xml_pairs_recursive(base, &path, dst_base, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("xml") {
                let rel = path.strip_prefix(base).unwrap();
                out.push((path.clone(), dst_base.join(rel)));
            }
        }
    }
}

fn collect_other_xml_pairs(backup: &Path, audio: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();
    collect_other_recursive(backup, backup, audio, &mut pairs);
    pairs
}

fn collect_other_recursive(base: &Path, dir: &Path, audio: &Path, out: &mut Vec<(PathBuf, PathBuf)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_other_recursive(base, &path, audio, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("xml") {
                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                // Skip ModularCars engine files (handled separately)
                let in_mc = path.to_string_lossy().contains("ModularCars");
                let is_engine = fname.ends_with("-Engine.xml");
                if in_mc && is_engine { continue; }
                let rel = path.strip_prefix(base).unwrap();
                out.push((path.clone(), audio.join(rel)));
            }
        }
    }
}
