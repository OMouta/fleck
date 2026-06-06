#[tauri::command]
fn ownership_boundaries() -> Vec<(String, String)> {
    fleck_core::ownership_boundaries()
        .iter()
        .map(|boundary| {
            (
                boundary.owner.to_owned(),
                boundary.responsibility.to_owned(),
            )
        })
        .collect()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![ownership_boundaries])
        .run(tauri::generate_context!())
        .expect("failed to run Fleck desktop app");
}
