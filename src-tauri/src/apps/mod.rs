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

#[cfg(windows)]
mod windows;

pub fn scan_apps() -> Vec<AppRecord> {
    #[cfg(windows)]
    {
        return windows::scan_apps_windows();
    }

    #[cfg(not(windows))]
    Vec::new()
}

pub fn scan_apps_stream<FProgress, FRecord>(on_progress: FProgress, on_record: FRecord)
where
    FProgress: FnMut(ScanProgress),
    FRecord: FnMut(AppRecord),
{
    #[cfg(windows)]
    {
        windows::scan_apps_stream_windows(on_progress, on_record);
        return;
    }

    #[cfg(not(windows))]
    {
        let _ = (on_progress, on_record);
    }
}

pub fn audit_overview() -> AuditOverview {
    #[cfg(windows)]
    {
        return windows::audit_overview_windows();
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
        return windows::measure_folder_size_windows(kind, folder);
    }

    #[cfg(not(windows))]
    {
        let _ = kind;
        let _ = folder;
        0
    }
}
