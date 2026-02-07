use std::collections::HashMap;

use super::matching::{build_app_tokens, build_candidate_folder_keys, score_folder, AppTokens};
use super::roots::{build_roots, RootFolders};
use super::sizing::directory_size;
use super::uninstall::{dedupe_uninstall_entries, get_install_dir_hint, scan_uninstall_entries};
use crate::apps::{AuditDuplicateInstallLocation, AuditOverview, AuditRootSummary, AuditUnassignedFolder};

pub(super) fn audit_overview_windows() -> AuditOverview {
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
            install_dir_to_apps.entry(k).or_default().push(u.name.clone());
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
            Some(AuditDuplicateInstallLocation { install_dir: dir, apps })
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

fn extend_unassigned_preview(
    out: &mut Vec<AuditUnassignedFolder>,
    kind: &str,
    root: &RootFolders,
    owners: &HashMap<String, String>,
) {
    let mut keys: Vec<&String> = root.folders.keys().filter(|k| !owners.contains_key(*k)).collect();
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

    owners.into_iter().map(|(k, (_s, id))| (k, id)).collect()
}

pub(super) fn measure_folder_size_windows(kind: &str, folder: &str) -> u64 {
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
