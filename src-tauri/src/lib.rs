// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Emitter;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub mod apps;

#[tauri::command]
fn scan_apps() -> Vec<apps::AppRecord> {
    apps::scan_apps()
}

#[tauri::command]
fn get_audit_overview() -> apps::AuditOverview {
    apps::audit_overview()
}

#[tauri::command]
fn measure_audit_folder_size(kind: String, folder: String) -> u64 {
    apps::measure_folder_size(&kind, &folder)
}

#[tauri::command]
async fn start_scan_apps(app: tauri::AppHandle) -> Result<(), String> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_apps,
            start_scan_apps,
            get_audit_overview,
            measure_audit_folder_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
