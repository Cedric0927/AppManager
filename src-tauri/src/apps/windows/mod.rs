use std::collections::HashMap;
use std::path::PathBuf;

use super::{AppBreakdownEntry, AppRecord, ScanProgress};

mod audit;
mod matching;
mod roots;
mod sizing;
mod uninstall;

pub(super) fn get_disk_info_windows() -> Vec<super::DiskInfo> {
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .map(|d| super::DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_space: d.total_space(),
            available_space: d.available_space(),
            is_removable: d.is_removable(),
        })
        .collect()
}

pub(super) fn audit_overview_windows() -> super::AuditOverview {
    audit::audit_overview_windows()
}

pub(super) fn measure_folder_size_windows(kind: &str, folder: &str) -> u64 {
    audit::measure_folder_size_windows(kind, folder)
}

pub(super) fn scan_apps_windows() -> Vec<AppRecord> {
    let mut out = Vec::new();
    scan_apps_stream_windows(|_| {}, |r| out.push(r));
    out
}

pub(super) fn scan_apps_stream_windows<FProgress, FRecord>(
    mut on_progress: FProgress,
    mut on_record: FRecord,
) where
    FProgress: FnMut(ScanProgress),
    FRecord: FnMut(AppRecord),
{
    let mut uninstall = uninstall::scan_uninstall_entries();
    uninstall.sort_by(|a, b| a.name.cmp(&b.name));
    uninstall = uninstall::dedupe_uninstall_entries(uninstall);

    on_progress(ScanProgress {
        phase: "uninstall".into(),
        current: 0,
        total: uninstall.len() as u32,
        message: "已识别安装软件列表".into(),
    });

    let roots = roots::build_roots();
    let app_tokens = matching::build_app_tokens(&uninstall);
    let assigned = matching::assign_folders(&roots, &app_tokens);
    let mut size_cache: HashMap<PathBuf, u64> = HashMap::new();

    let total = uninstall.len().max(1) as u32;
    for (i, u) in uninstall.into_iter().enumerate() {
        let record = enrich_with_breakdown(u, &assigned, &mut size_cache);
        on_record(record);
        on_progress(ScanProgress {
            phase: "scan".into(),
            current: (i as u32).saturating_add(1),
            total,
            message: "正在分析占用细节…".into(),
        });
    }

    on_progress(ScanProgress {
        phase: "done".into(),
        current: total,
        total,
        message: "扫描完成".into(),
    });
}

fn enrich_with_breakdown(
    uninstall: uninstall::UninstallEntry,
    assigned: &matching::AssignedFolders,
    size_cache: &mut HashMap<PathBuf, u64>,
) -> AppRecord {
    let mut breakdown = Vec::new();

    let (program_bytes, program_paths, program_label) = if uninstall.estimated_bytes > 0 {
        (
            uninstall.estimated_bytes,
            Vec::new(),
            "软件程序 (系统估算)".to_string(),
        )
    } else {
        let (bytes, paths) = sizing::compute_install_bytes(&uninstall, size_cache);
        (bytes, paths, "软件程序 (目录扫描)".to_string())
    };

    breakdown.push(AppBreakdownEntry {
        kind: "program".into(),
        label: program_label,
        bytes: program_bytes,
        paths: program_paths,
    });

    if let Some(paths) = assigned.local.get(&uninstall.id) {
        let (bytes, shown) = sizing::sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "appDataLocal".into(),
                label: "应用数据 (AppData/Local)".into(),
                bytes,
                paths: shown,
            });
        }
    }

    if let Some(paths) = assigned.roaming.get(&uninstall.id) {
        let (bytes, shown) = sizing::sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "appDataRoaming".into(),
                label: "应用数据 (AppData/Roaming)".into(),
                bytes,
                paths: shown,
            });
        }
    }

    if let Some(paths) = assigned.local_low.get(&uninstall.id) {
        let (bytes, shown) = sizing::sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "appDataLocalLow".into(),
                label: "应用数据 (AppData/LocalLow)".into(),
                bytes,
                paths: shown,
            });
        }
    }

    if let Some(paths) = assigned.program_data.get(&uninstall.id) {
        let (bytes, shown) = sizing::sum_paths(paths, size_cache);
        if bytes > 0 {
            breakdown.push(AppBreakdownEntry {
                kind: "programData".into(),
                label: "共享数据 (ProgramData)".into(),
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
