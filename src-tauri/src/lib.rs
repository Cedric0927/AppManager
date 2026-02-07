pub mod apps;
mod commands;

use commands::{get_audit_overview, greet, measure_audit_folder_size, scan_apps, start_scan_apps};

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
