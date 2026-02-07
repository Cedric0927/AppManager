use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::uninstall::{get_install_dir_hint, UninstallEntry};

pub(super) fn sum_paths(paths: &[PathBuf], size_cache: &mut HashMap<PathBuf, u64>) -> (u64, Vec<String>) {
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

pub(super) fn compute_install_bytes(
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

pub(super) fn directory_size(root: &Path) -> u64 {
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

fn directory_size_cached(root: &Path, cache: &mut HashMap<PathBuf, u64>) -> u64 {
    let key = root.to_path_buf();
    if let Some(v) = cache.get(&key) {
        return *v;
    }
    let bytes = directory_size(root);
    cache.insert(key, bytes);
    bytes
}
