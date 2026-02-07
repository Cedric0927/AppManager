use std::collections::HashMap;
use std::path::PathBuf;

use super::matching::normalize_key;

#[derive(Clone)]
pub(super) struct UninstallEntry {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) publisher: Option<String>,
    pub(super) estimated_bytes: u64,
    pub(super) install_location: Option<String>,
    pub(super) display_icon: Option<String>,
}

pub(super) fn scan_uninstall_entries() -> Vec<UninstallEntry> {
    use winreg::enums::*;
    use winreg::RegKey;

    const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    const UNINSTALL_WOW6432_PATH: &str =
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";

    let mut out = Vec::new();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    out.extend(read_uninstall_key(
        &hklm,
        UNINSTALL_PATH,
        KEY_READ | KEY_WOW64_64KEY,
        "hklm64",
    ));
    out.extend(read_uninstall_key(
        &hklm,
        UNINSTALL_WOW6432_PATH,
        KEY_READ | KEY_WOW64_32KEY,
        "hklm32",
    ));

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    out.extend(read_uninstall_key(&hkcu, UNINSTALL_PATH, KEY_READ, "hkcu"));

    out
}

pub(super) fn dedupe_uninstall_entries(entries: Vec<UninstallEntry>) -> Vec<UninstallEntry> {
    let mut map: HashMap<String, UninstallEntry> = HashMap::new();

    for e in entries {
        let name_key = normalize_key(&strip_version_suffix(&e.name));
        let publisher_key = e
            .publisher
            .as_ref()
            .map(|p| normalize_key(p))
            .unwrap_or_default();
        let install_key = get_install_dir_hint(&e)
            .map(|p| p.to_string_lossy().to_string().to_ascii_lowercase())
            .unwrap_or_default();

        let base_key = format!("{}|{}", name_key, publisher_key);
        let key = if install_key.is_empty() {
            base_key.clone()
        } else {
            format!("{base_key}|{install_key}")
        };

        if !install_key.is_empty() && !map.contains_key(&key) && map.contains_key(&base_key) {
            if let Some(mut existing) = map.remove(&base_key) {
                merge_uninstall_entry(&mut existing, &e);
                map.insert(key, existing);
            } else {
                map.insert(key, e);
            }
            continue;
        }

        match map.get_mut(&key) {
            Some(existing) => merge_uninstall_entry(existing, &e),
            None => {
                map.insert(key, e);
            }
        }
    }

    map.into_values().collect()
}

fn merge_uninstall_entry(existing: &mut UninstallEntry, incoming: &UninstallEntry) {
    if incoming.estimated_bytes > existing.estimated_bytes {
        existing.estimated_bytes = incoming.estimated_bytes;
    }
    if existing.install_location.is_none() {
        existing.install_location = incoming.install_location.clone();
    }
    if existing.display_icon.is_none() {
        existing.display_icon = incoming.display_icon.clone();
    }
    if entry_quality(incoming) > entry_quality(existing) {
        existing.name = incoming.name.clone();
        existing.publisher = incoming.publisher.clone();
    }
}

fn strip_version_suffix(name: &str) -> String {
    let s = name.trim();
    if s.is_empty() {
        return String::new();
    }

    let mut out = s.to_string();

    if let Some((left, right)) = out.rsplit_once('(') {
        let right = right.trim_end_matches(')').trim();
        if is_version_like(right) {
            out = left.trim().to_string();
        }
    }

    if let Some((left, right)) = out.rsplit_once('[') {
        let right = right.trim_end_matches(']').trim();
        if is_version_like(right) {
            out = left.trim().to_string();
        }
    }

    let parts: Vec<&str> = out.split_whitespace().collect();
    if parts.len() >= 2 {
        let last = parts[parts.len() - 1];
        if is_version_like(last) {
            out = parts[..parts.len() - 1].join(" ");
        } else if parts.len() >= 3 {
            let prev = parts[parts.len() - 2].to_lowercase();
            if (prev == "v" || prev == "ver" || prev == "version") && is_version_like(last) {
                out = parts[..parts.len() - 2].join(" ");
            }
        }
    }

    out.trim().to_string()
}

fn is_version_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let s = s.strip_prefix('v').or_else(|| s.strip_prefix('V')).unwrap_or(s);
    let mut has_digit = false;
    let mut has_dot = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            has_digit = true;
            continue;
        }
        if c == '.' || c == '_' || c == '-' {
            if c == '.' {
                has_dot = true;
            }
            continue;
        }
        return false;
    }
    has_digit && has_dot
}

fn entry_quality(e: &UninstallEntry) -> i32 {
    let mut score = 0i32;
    if e.install_location.is_some() {
        score += 1000;
    }
    if e.display_icon.is_some() {
        score += 200;
    }
    if e.estimated_bytes > 0 {
        score += 50;
    }
    score += (e.name.len().min(64)) as i32;
    score
}

pub(super) fn get_install_dir_hint(uninstall: &UninstallEntry) -> Option<PathBuf> {
    if let Some(s) = uninstall.install_location.as_deref() {
        let p = PathBuf::from(s);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Some(icon) = uninstall.display_icon.as_deref() {
        return parse_display_icon_to_dir(icon);
    }
    None
}

fn parse_display_icon_to_dir(display_icon: &str) -> Option<PathBuf> {
    let mut s = display_icon.trim().to_string();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s = s[1..s.len() - 1].to_string();
    }

    if let Some((left, _right)) = s.rsplit_once(',') {
        let candidate = left.trim();
        if candidate.contains(":\\") {
            s = candidate.to_string();
        }
    }

    let p = PathBuf::from(s.trim());
    if p.is_dir() {
        return Some(p);
    }
    if p.is_file() {
        return p.parent().map(|d| d.to_path_buf());
    }
    None
}

fn read_uninstall_key(
    root: &winreg::RegKey,
    subkey_path: &str,
    flags: u32,
    id_prefix: &str,
) -> Vec<UninstallEntry> {
    let key = match root.open_subkey_with_flags(subkey_path, flags) {
        Ok(k) => k,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();

    for subkey_name in key.enum_keys().flatten() {
        let sub = match key.open_subkey_with_flags(&subkey_name, flags) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let name: String = match sub.get_value("DisplayName") {
            Ok(v) => v,
            Err(_) => continue,
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        let system_component: u32 = sub.get_value("SystemComponent").unwrap_or(0);
        if system_component == 1 {
            continue;
        }

        let release_type: Option<String> = sub.get_value("ReleaseType").ok();
        if matches!(
            release_type.as_deref(),
            Some("Update") | Some("Hotfix") | Some("Security Update")
        ) {
            continue;
        }

        let parent_key_name: Option<String> = sub.get_value("ParentKeyName").ok();
        let parent_display_name: Option<String> = sub.get_value("ParentDisplayName").ok();
        if parent_key_name.is_some() || parent_display_name.is_some() {
            continue;
        }

        let publisher: Option<String> = sub
            .get_value::<String, _>("Publisher")
            .ok()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty());

        let estimated_kb: Option<u32> = sub.get_value("EstimatedSize").ok();
        let estimated_bytes = estimated_kb.map(|kb| kb as u64 * 1024).unwrap_or(0);

        let install_location: Option<String> = sub
            .get_value::<String, _>("InstallLocation")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let display_icon: Option<String> = sub
            .get_value::<String, _>("DisplayIcon")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let id = format!("{id_prefix}:{subkey_name}");

        out.push(UninstallEntry {
            id,
            name,
            publisher,
            estimated_bytes,
            install_location,
            display_icon,
        });
    }

    out
}
