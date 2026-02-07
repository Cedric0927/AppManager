use crate::apps;
use tauri::Emitter;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn scan_apps() -> Vec<apps::AppRecord> {
    apps::scan_apps()
}

#[tauri::command]
pub fn get_audit_overview() -> apps::AuditOverview {
    apps::audit_overview()
}

#[tauri::command]
pub fn measure_audit_folder_size(kind: String, folder: String) -> u64 {
    apps::measure_folder_size(&kind, &folder)
}

#[tauri::command]
pub async fn start_scan_apps(app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        apps::scan_apps_stream(
            |p| {
                let _ = app.emit("scan_progress", p);
            },
            |r| {
                let _ = app.emit("scan_result", r);
            },
        );
        let _ = app.emit("scan_done", ());
    });
    Ok(())
}
