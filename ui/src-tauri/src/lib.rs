use backup_checker::{check_files, ChecksumGenerator};

#[tauri::command]
fn check_backup(
    old_folder: String,
    new_folder: String,
    max_depth: i16,
    generator: ChecksumGenerator,
) -> Vec<String> {
    check_files(&old_folder, &new_folder, max_depth, &generator, false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![check_backup])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
