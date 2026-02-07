use std::collections::HashMap;
use std::path::PathBuf;

use super::roots::{RootFolders, Roots};
use super::uninstall::UninstallEntry;

#[derive(Clone)]
pub(super) struct AppTokens {
    pub(super) app_id: String,
    pub(super) name_tokens: Vec<String>,
    pub(super) publisher_tokens: Vec<String>,
    pub(super) allow_publisher_only: bool,
}

pub(super) struct AssignedFolders {
    pub(super) local: HashMap<String, Vec<PathBuf>>,
    pub(super) roaming: HashMap<String, Vec<PathBuf>>,
    pub(super) local_low: HashMap<String, Vec<PathBuf>>,
    pub(super) program_data: HashMap<String, Vec<PathBuf>>,
}

pub(super) fn build_app_tokens(uninstall: &[UninstallEntry]) -> Vec<AppTokens> {
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

pub(super) fn assign_folders(roots: &Roots, tokens: &[AppTokens]) -> AssignedFolders {
    AssignedFolders {
        local: assign_for_root(roots.local.as_ref(), tokens),
        roaming: assign_for_root(roots.roaming.as_ref(), tokens),
        local_low: assign_for_root(roots.local_low.as_ref(), tokens),
        program_data: assign_for_root(roots.program_data.as_ref(), tokens),
    }
}

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

pub(super) fn build_candidate_folder_keys(tokens: &AppTokens) -> Vec<String> {
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

pub(super) fn score_folder(folder_key: &str, tokens: &AppTokens) -> i32 {
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

fn split_tokens(s: &str) -> Vec<String> {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .collect()
}

pub(super) fn normalize_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

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
