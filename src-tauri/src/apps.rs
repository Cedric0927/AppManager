use std::collections::HashMap;
use std::path::PathBuf;

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppBreakdownEntry {
    pub kind: String,
    pub label: String,
    pub bytes: u64,
    pub paths: Vec<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppRecord {
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub total_bytes: u64,
    pub breakdown: Vec<AppBreakdownEntry>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub phase: String,
    pub current: u32,
    pub total: u32,
    pub message: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuditRootSummary {
    pub kind: String,
    pub assigned_folders: u32,
    pub unassigned_folders: u32,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuditDuplicateInstallLocation {
    pub install_dir: String,
    pub apps: Vec<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuditUnassignedFolder {
    pub kind: String,
    pub folder: String,
    pub path: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuditOverview {
    pub app_count: u32,
    pub unknown_program_size_count: u32,
    pub roots: Vec<AuditRootSummary>,
    pub duplicate_install_locations: Vec<AuditDuplicateInstallLocation>,
    pub unassigned_folders: Vec<AuditUnassignedFolder>,
}

pub fn scan_apps() -> Vec<AppRecord> {
    #[cfg(windows)]
    {
        return scan_apps_windows();
    }

    #[cfg(not(windows))]
    Vec::new()
}

pub fn audit_overview() -> AuditOverview {
    #[cfg(windows)]
    {
        return audit_overview_windows();
    }

    #[cfg(not(windows))]
    AuditOverview {
        app_count: 0,
        unknown_program_size_count: 0,
        roots: Vec::new(),
        duplicate_install_locations: Vec::new(),
        unassigned_folders: Vec::new(),
    }
}

pub fn measure_folder_size(kind: &str, folder: &str) -> u64 {
    #[cfg(windows)]
    {
        return measure_folder_size_windows(kind, folder);
    }

    #[cfg(not(windows))]
    0
}

#[cfg(windows)]
#[derive(Clone)]
struct UninstallEntry {
    id: String,
    name: String,
    publisher: Option<String>,
    estimated_bytes: u64,
    install_location: Option<String>,
    display_icon: Option<String>,
}

#[cfg(windows)]
fn scan_apps_windows() -> Vec<AppRecord> {
    let mut out = Vec::new();
    scan_apps_stream_windows(|_| {}, |r| out.push(r));
    out
}

#[cfg(windows)]
pub fn scan_apps_stream<FProgress, FRecord>(on_progress: FProgress, on_record: FRecord)
where
    FProgress: FnMut(ScanProgress),
    FRecord: FnMut(AppRecord),
{
    scan_apps_stream_windows(on_progress, on_record);
}

#[cfg(windows)]
fn scan_apps_stream_windows<FProgress, FRecord>(mut on_progress: FProgress, mut on_record: FRecord)
where
    FProgress: FnMut(ScanProgress),
    FRecord: FnMut(AppRecord),
{
    let mut uninstall = scan_uninstall_entries();
    uninstall.sort_by(|a, b| a.name.cmp(&b.name));
    uninstall = dedupe_uninstall_entries(uninstall);

    on_progress(ScanProgress {
        phase: "uninstall".into(),
        current: 0,
        total: uninstall.len() as u32,
        message: "已读取已安装软件清单".into(),
    });

    let roots = build_roots();
    let app_tokens = build_app_tokens(&uninstall);
    let assigned = assign_folders(&roots, &app_tokens);
    let mut size_cache: HashMap<PathBuf, u64> = HashMap::new();

    let total = uninstall.len().max(1) as u32;
    for (i, u) in uninstall.into_iter().enumerate() {
        let record = enrich_with_breakdown(u, &assigned, &mut size_cache);
        on_record(record);
        on_progress(ScanProgress {
            phase: "scan".into(),
            current: (i as u32).saturating_add(1),
            total,
            message: "正在归因占用…".into(),
        });
    }

    on_progress(ScanProgress {
        phase: "done".into(),
        current: total,
        total,
        message: "扫描完成".into(),
    });
}

#[cfg(windows)]
#[derive(Clone)]
struct RootFolders {
    folders: HashMap<String, PathBuf>,
}

#[derive(Clone)]
struct Roots {
    local: Option<RootFolders>,
    roaming: Option<RootFolders>,
    local_low: Option<RootFolders>,
    program_data: Option<RootFolders>,
}

#[cfg(windows)]
fn build_roots() -> Roots {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(list_root_folders);
    let roaming = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(list_root_folders);
    let local_low = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|p| p.join(r"AppData\LocalLow"))
        .filter(|p| p.is_dir())
        .map(list_root_folders);
    let program_data = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(list_root_folders);

    Roots {
        local,
        roaming,
        local_low,
        program_data,
    }
}

#[cfg(windows)]
fn list_root_folders(root: PathBuf) -> RootFolders {
    let mut folders = HashMap::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            let key = name.to_lowercase();
            folders.insert(key, p);
        }
    }
    RootFolders { folders }
}

#[cfg(windows)]
fn audit_overview_windows() -> AuditOverview {
    let mut uninstall = scan_uninstall_entries();
    uninstall.sort_by(|a, b| a.name.cmp(&b.name));
    uninstall = dedupe_uninstall_entries(uninstall);

    let app_tokens = build_app_tokens(&uninstall);
    let roots = build_roots();

    let mut unknown_program_size_count = 0u32;
    let mut install_dir_to_apps: HashMap<String, Vec<String>> = HashMap::new();
    for u in &uninstall {
        if u.estimated_bytes == 0 && get_install_dir_hint(u).is_none() {
            unknown_program_size_count = unknown_program_size_count.saturating_add(1);
        }
        if let Some(p) = get_install_dir_hint(u) {
            let k = p.to_string_lossy().to_string().to_ascii_lowercase();
            install_dir_to_apps
                .entry(k)
                .or_default()
                .push(u.name.clone());
        }
    }

    let mut duplicate_install_locations: Vec<AuditDuplicateInstallLocation> = install_dir_to_apps
        .into_iter()
        .filter_map(|(dir, mut apps)| {
            if apps.len() <= 1 {
                return None;
            }
            apps.sort();
            apps.dedup();
            if apps.len() <= 1 {
                return None;
            }
            Some(AuditDuplicateInstallLocation {
                install_dir: dir,
                apps,
            })
        })
        .collect();
    duplicate_install_locations.sort_by(|a, b| b.apps.len().cmp(&a.apps.len()));

    let owners_local = build_owner_keys(roots.local.as_ref(), &app_tokens);
    let owners_roaming = build_owner_keys(roots.roaming.as_ref(), &app_tokens);
    let owners_local_low = build_owner_keys(roots.local_low.as_ref(), &app_tokens);
    let owners_program_data = build_owner_keys(roots.program_data.as_ref(), &app_tokens);

    let mut unassigned_folders = Vec::new();
    let mut root_summaries = Vec::new();

    if let Some(root) = roots.local.as_ref() {
        let assigned = owners_local.len() as u32;
        let unassigned = root
            .folders
            .keys()
            .filter(|k| !owners_local.contains_key(*k))
            .count() as u32;
        root_summaries.push(AuditRootSummary {
            kind: "appDataLocal".into(),
            assigned_folders: assigned,
            unassigned_folders: unassigned,
        });
        extend_unassigned_preview(&mut unassigned_folders, "appDataLocal", root, &owners_local);
    }

    if let Some(root) = roots.roaming.as_ref() {
        let assigned = owners_roaming.len() as u32;
        let unassigned = root
            .folders
            .keys()
            .filter(|k| !owners_roaming.contains_key(*k))
            .count() as u32;
        root_summaries.push(AuditRootSummary {
            kind: "appDataRoaming".into(),
            assigned_folders: assigned,
            unassigned_folders: unassigned,
        });
        extend_unassigned_preview(&mut unassigned_folders, "appDataRoaming", root, &owners_roaming);
    }

    if let Some(root) = roots.local_low.as_ref() {
        let assigned = owners_local_low.len() as u32;
        let unassigned = root
            .folders
            .keys()
            .filter(|k| !owners_local_low.contains_key(*k))
            .count() as u32;
        root_summaries.push(AuditRootSummary {
            kind: "appDataLocalLow".into(),
            assigned_folders: assigned,
            unassigned_folders: unassigned,
        });
        extend_unassigned_preview(
            &mut unassigned_folders,
            "appDataLocalLow",
            root,
            &owners_local_low,
        );
    }

    if let Some(root) = roots.program_data.as_ref() {
        let assigned = owners_program_data.len() as u32;
        let unassigned = root
            .folders
            .keys()
            .filter(|k| !owners_program_data.contains_key(*k))
            .count() as u32;
        root_summaries.push(AuditRootSummary {
            kind: "programData".into(),
            assigned_folders: assigned,
            unassigned_folders: unassigned,
        });
        extend_unassigned_preview(
            &mut unassigned_folders,
            "programData",
            root,
            &owners_program_data,
        );
    }

    unassigned_folders.sort_by(|a, b| a.path.cmp(&b.path));
    unassigned_folders.truncate(200);

    AuditOverview {
        app_count: uninstall.len() as u32,
        unknown_program_size_count,
        roots: root_summaries,
        duplicate_install_locations,
        unassigned_folders,
    }
}

#[cfg(windows)]
fn extend_unassigned_preview(
    out: &mut Vec<AuditUnassignedFolder>,
    kind: &str,
    root: &RootFolders,
    owners: &HashMap<String, String>,
) {
    let mut keys: Vec<&String> = root
        .folders
        .keys()
        .filter(|k| !owners.contains_key(*k))
        .collect();
    keys.sort();
    for k in keys.into_iter().take(80) {
        if let Some(p) = root.folders.get(k) {
            out.push(AuditUnassignedFolder {
                kind: kind.into(),
                folder: k.clone(),
                path: p.to_string_lossy().to_string(),
            });
        }
    }
}

#[cfg(windows)]
fn build_owner_keys(root: Option<&RootFolders>, tokens: &[AppTokens]) -> HashMap<String, String> {
    let Some(root) = root else {
        return HashMap::new();
    };

    let mut owners: HashMap<String, (i32, String)> = HashMap::new();

    for app in tokens {
        let candidates = build_candidate_folder_keys(app);
        for c in candidates {
            if !root.folders.contains_key(&c) {
                continue;
            }
            let score = score_folder(&c, app);
            if score <= 0 {
                continue;
            }
            match owners.get(&c) {
                Some((best_score, _)) if *best_score >= score => {}
                _ => {
                    owners.insert(c, (score, app.app_id.clone()));
                }
            }
        }
    }

    owners
        .into_iter()
        .map(|(k, (_s, id))| (k, id))
        .collect()
}

#[cfg(windows)]
fn measure_folder_size_windows(kind: &str, folder: &str) -> u64 {
    let roots = build_roots();
    let folder_key = folder.to_ascii_lowercase();

    let root = match kind {
        "appDataLocal" => roots.local.as_ref(),
        "appDataRoaming" => roots.roaming.as_ref(),
        "appDataLocalLow" => roots.local_low.as_ref(),
        "programData" => roots.program_data.as_ref(),
        _ => None,
    };

    let Some(root) = root else {
        return 0;
    };

    let Some(path) = root.folders.get(&folder_key) else {
        return 0;
    };

    directory_size(path)
}

#[cfg(windows)]
fn scan_uninstall_entries() -> Vec<UninstallEntry> {
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

#[cfg(windows)]
fn dedupe_uninstall_entries(entries: Vec<UninstallEntry>) -> Vec<UninstallEntry> {
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

#[cfg(windows)]
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

#[cfg(windows)]
#[derive(Clone)]
struct AppTokens {
    app_id: String,
    name_tokens: Vec<String>,
    publisher_tokens: Vec<String>,
    allow_publisher_only: bool,
}

#[cfg(windows)]
struct AssignedFolders {
    local: HashMap<String, Vec<PathBuf>>,
    roaming: HashMap<String, Vec<PathBuf>>,
    local_low: HashMap<String, Vec<PathBuf>>,
    program_data: HashMap<String, Vec<PathBuf>>,
}

#[cfg(windows)]
fn enrich_with_breakdown(
    uninstall: UninstallEntry,
    assigned: &AssignedFolders,
    size_cache: &mut HashMap<PathBuf, u64>,
) -> AppRecord {
    let mut breakdown = Vec::new();

    let (program_bytes, program_paths, program_label) = if uninstall.estimated_bytes > 0 {
        (
            uninstall.estimated_bytes,
            Vec::new(),
            "程序本身（注册表估算）".to_string(),
        )
    } else {
        let (bytes, paths) = compute_install_bytes(&uninstall, size_cache);
        (bytes, paths, "程序本身（安装目录扫描）".to_string())
    };

    breakdown.push(AppBreakdownEntry {
        kind: "program".into(),
        label: program_label,
        bytes: program_bytes,
        paths: program_paths,
    });

    if let Some(paths) = assigned.local.get(&uninstall.id) {
        let (bytes, shown) = sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "appDataLocal".into(),
                label: "数据（AppData\\Local）".into(),
                bytes,
                paths: shown,
            });
        }
    }

    if let Some(paths) = assigned.roaming.get(&uninstall.id) {
        let (bytes, shown) = sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "appDataRoaming".into(),
                label: "数据（AppData\\Roaming）".into(),
                bytes,
                paths: shown,
            });
        }
    }

    if let Some(paths) = assigned.local_low.get(&uninstall.id) {
        let (bytes, shown) = sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "appDataLocalLow".into(),
                label: "数据（AppData\\LocalLow）".into(),
                bytes,
                paths: shown,
            });
        }
    }

    if let Some(paths) = assigned.program_data.get(&uninstall.id) {
        let (bytes, shown) = sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "programData".into(),
                label: "数据（ProgramData）".into(),
                bytes,
                paths: shown,
            });
        }
    }

    let total_bytes = breakdown.iter().map(|b| b.bytes).sum();

    AppRecord {
        id: uninstall.id,
        name: uninstall.name,
        publisher: uninstall.publisher,
        total_bytes,
        breakdown,
    }
}

#[cfg(windows)]
fn sum_paths(paths: &[PathBuf], size_cache: &mut HashMap<PathBuf, u64>) -> (u64, Vec<String>) {
    let mut items: Vec<(u64, String)> = Vec::new();
    let mut total = 0u64;

    for p in paths {
        let bytes = directory_size_cached(p, size_cache);
        total = total.saturating_add(bytes);
        items.push((bytes, p.to_string_lossy().to_string()));
    }

    items.sort_by(|a, b| b.0.cmp(&a.0));
    let shown = items.into_iter().take(5).map(|(_, p)| p).collect();
    (total, shown)
}

#[cfg(windows)]
fn build_app_tokens(uninstall: &[UninstallEntry]) -> Vec<AppTokens> {
    uninstall
        .iter()
        .map(|u| {
            let name_tokens = build_name_tokens(&u.name);
            let publisher_tokens = u
                .publisher
                .as_deref()
                .map(build_publisher_tokens)
                .unwrap_or_default();
            let allow_publisher_only = name_tokens.is_empty();

            AppTokens {
                app_id: u.id.clone(),
                name_tokens,
                publisher_tokens,
                allow_publisher_only,
            }
        })
        .collect()
}

#[cfg(windows)]
fn assign_folders(roots: &Roots, tokens: &[AppTokens]) -> AssignedFolders {
    AssignedFolders {
        local: assign_for_root(roots.local.as_ref(), tokens),
        roaming: assign_for_root(roots.roaming.as_ref(), tokens),
        local_low: assign_for_root(roots.local_low.as_ref(), tokens),
        program_data: assign_for_root(roots.program_data.as_ref(), tokens),
    }
}

#[cfg(windows)]
fn assign_for_root(root: Option<&RootFolders>, tokens: &[AppTokens]) -> HashMap<String, Vec<PathBuf>> {
    let Some(root) = root else {
        return HashMap::new();
    };

    let mut owners: HashMap<String, (i32, String)> = HashMap::new();

    for app in tokens {
        let candidates = build_candidate_folder_keys(app);
        for c in candidates {
            if !root.folders.contains_key(&c) {
                continue;
            }
            let score = score_folder(&c, app);
            if score <= 0 {
                continue;
            }
            match owners.get(&c) {
                Some((best_score, _)) if *best_score >= score => {}
                _ => {
                    owners.insert(c, (score, app.app_id.clone()));
                }
            }
        }
    }

    let mut assigned: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for (folder_key, (_score, app_id)) in owners {
        if let Some(p) = root.folders.get(&folder_key) {
            assigned.entry(app_id).or_default().push(p.clone());
        }
    }

    assigned
}

#[cfg(windows)]
fn build_candidate_folder_keys(tokens: &AppTokens) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    for t in &tokens.name_tokens {
        if t.len() >= 3 && t.len() <= 32 {
            out.push(t.to_string());
        }
    }

    for t in &tokens.publisher_tokens {
        if t.len() >= 3 && t.len() <= 32 {
            out.push(t.to_string());
        }
    }

    if tokens.publisher_tokens.len() >= 2 {
        let joined2 = format!("{}{}", tokens.publisher_tokens[0], tokens.publisher_tokens[1]);
        if joined2.len() >= 4 && joined2.len() <= 32 {
            out.push(joined2);
        }
    }
    if tokens.publisher_tokens.len() >= 3 {
        let joined3 = format!(
            "{}{}{}",
            tokens.publisher_tokens[0], tokens.publisher_tokens[1], tokens.publisher_tokens[2]
        );
        if joined3.len() >= 5 && joined3.len() <= 32 {
            out.push(joined3);
        }
    }

    out.sort();
    out.dedup();
    out
}

#[cfg(windows)]
fn score_folder(folder_key: &str, tokens: &AppTokens) -> i32 {
    let mut name_score = 0i32;
    for t in &tokens.name_tokens {
        if folder_key.contains(t) {
            name_score += t.len() as i32;
        }
    }

    let mut publisher_score = 0i32;
    for t in &tokens.publisher_tokens {
        if folder_key.contains(t) {
            publisher_score += t.len() as i32;
        }
    }

    if name_score > 0 {
        let total = name_score * 100 + publisher_score;
        if total >= 300 {
            return total;
        }
        return 0;
    }

    if tokens.allow_publisher_only && publisher_score > 0 {
        let total = publisher_score * 50;
        if total >= 200 {
            return total;
        }
    }

    0
}

#[cfg(windows)]
fn build_name_tokens(name: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    let key = normalize_key(name);
    if key.len() >= 3 {
        tokens.push(key);
    }

    for t in split_tokens(name) {
        if t.len() >= 3 && !is_stop_token_name(&t) && !t.chars().all(|c| c.is_ascii_digit()) {
            if !tokens.contains(&t) {
                tokens.push(t);
            }
        }
    }

    tokens
}

#[cfg(windows)]
fn build_publisher_tokens(publisher: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for t in split_tokens(publisher) {
        if t.len() >= 4
            && !is_stop_token_publisher(&t)
            && !t.chars().all(|c| c.is_ascii_digit())
        {
            if !tokens.contains(&t) {
                tokens.push(t);
            }
        }
    }
    tokens
}

#[cfg(windows)]
fn split_tokens(s: &str) -> Vec<String> {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .collect()
}

#[cfg(windows)]
fn normalize_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

#[cfg(windows)]
fn is_stop_token_name(token: &str) -> bool {
    matches!(
        token,
        "windows"
            | "update"
            | "installer"
            | "setup"
            | "runtime"
            | "redistributable"
            | "driver"
            | "tool"
            | "tools"
            | "plugin"
            | "service"
            | "sdk"
            | "for"
            | "and"
            | "the"
            | "app"
    )
}

#[cfg(windows)]
fn is_stop_token_publisher(token: &str) -> bool {
    matches!(
        token,
        "microsoft"
            | "nvidia"
            | "corporation"
            | "corp"
            | "inc"
            | "ltd"
            | "llc"
            | "co"
            | "company"
            | "limited"
            | "gmbh"
            | "sarl"
            | "pty"
            | "plc"
            | "software"
            | "systems"
            | "system"
            | "technologies"
            | "technology"
            | "solution"
            | "solutions"
    )
}

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(windows)]
fn get_install_dir_hint(uninstall: &UninstallEntry) -> Option<PathBuf> {
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

#[cfg(windows)]
fn compute_install_bytes(
    uninstall: &UninstallEntry,
    size_cache: &mut HashMap<PathBuf, u64>,
) -> (u64, Vec<String>) {
    let Some(dir) = get_install_dir_hint(uninstall) else {
        return (0, Vec::new());
    };

    let bytes = directory_size_cached(&dir, size_cache);
    let paths = vec![dir.to_string_lossy().to_string()];
    (bytes, paths)
}

#[cfg(windows)]
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

#[cfg(windows)]
fn directory_size(root: &std::path::Path) -> u64 {
    use jwalk::WalkDir;

    let mut total = 0u64;
    for entry in WalkDir::new(root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let size = entry
            .metadata()
            .map(|m| m.len())
            .or_else(|_| std::fs::metadata(entry.path()).map(|m| m.len()))
            .unwrap_or(0);
        total = total.saturating_add(size);
    }
    total
}

#[cfg(windows)]
fn directory_size_cached(root: &std::path::Path, cache: &mut HashMap<PathBuf, u64>) -> u64 {
    let key = root.to_path_buf();
    if let Some(v) = cache.get(&key) {
        return *v;
    }
    let bytes = directory_size(root);
    cache.insert(key, bytes);
    bytes
}

#[cfg(windows)]
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
