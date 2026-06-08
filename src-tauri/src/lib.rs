#[cfg(target_os = "macos")]
use tauri::{
    menu::{AboutMetadata, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    Emitter, Manager, TitleBarStyle,
};

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

#[cfg(target_os = "macos")]
fn build_macos_menu<R: tauri::Runtime, M: tauri::Manager<R>>(
    manager: &M,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let app_menu = SubmenuBuilder::new(manager, "Fleck")
        .item(&PredefinedMenuItem::about(
            manager,
            None,
            Some(AboutMetadata {
                name: Some("Fleck".to_owned()),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                ..Default::default()
            }),
        )?)
        .separator()
        .item(&PredefinedMenuItem::services(manager, None)?)
        .separator()
        .item(&PredefinedMenuItem::hide(manager, None)?)
        .item(&PredefinedMenuItem::hide_others(manager, None)?)
        .item(&PredefinedMenuItem::show_all(manager, None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(manager, None)?)
        .build()?;

    let file_menu = SubmenuBuilder::new(manager, "File")
        .item(
            &MenuItemBuilder::with_id("new-workspace", "New Workspace")
                .accelerator("CmdOrCtrl+N")
                .build(manager)?,
        )
        .item(
            &MenuItemBuilder::with_id("open-workspace", "Open Workspace...")
                .accelerator("CmdOrCtrl+O")
                .build(manager)?,
        )
        .item(&MenuItemBuilder::with_id("open-image", "Open Image...").build(manager)?)
        .item(&MenuItemBuilder::with_id("paste-image", "Paste Image").build(manager)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("save-workspace", "Save")
                .accelerator("CmdOrCtrl+S")
                .build(manager)?,
        )
        .item(
            &MenuItemBuilder::with_id("save-as", "Save As...")
                .accelerator("CmdOrCtrl+Shift+S")
                .build(manager)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("export-all", "Export All")
                .accelerator("CmdOrCtrl+E")
                .build(manager)?,
        )
        .item(&MenuItemBuilder::with_id("share-workspace", "Share .fleck File").build(manager)?)
        .separator()
        .item(&PredefinedMenuItem::close_window(manager, None)?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(manager, "Edit")
        .item(
            &MenuItemBuilder::with_id("undo", "Undo")
                .accelerator("CmdOrCtrl+Z")
                .build(manager)?,
        )
        .item(
            &MenuItemBuilder::with_id("redo", "Redo")
                .accelerator("CmdOrCtrl+Shift+Z")
                .build(manager)?,
        )
        .item(
            &MenuItemBuilder::with_id("repeat-last-command", "Repeat Last Command")
                .accelerator("CmdOrCtrl+.")
                .build(manager)?,
        )
        .separator()
        .item(&PredefinedMenuItem::cut(manager, None)?)
        .item(&PredefinedMenuItem::copy(manager, None)?)
        .item(&PredefinedMenuItem::paste(manager, None)?)
        .item(&PredefinedMenuItem::select_all(manager, None)?)
        .build()?;

    let view_menu = SubmenuBuilder::new(manager, "View")
        .item(
            &MenuItemBuilder::with_id("command-palette", "Command Palette")
                .accelerator("CmdOrCtrl+K")
                .build(manager)?,
        )
        .separator()
        .item(&PredefinedMenuItem::fullscreen(manager, None)?)
        .build()?;

    let window_menu = SubmenuBuilder::new(manager, "Window")
        .item(&PredefinedMenuItem::minimize(manager, None)?)
        .item(&PredefinedMenuItem::maximize(manager, None)?)
        .separator()
        .item(&PredefinedMenuItem::bring_all_to_front(manager, None)?)
        .build()?;

    MenuBuilder::new(manager)
        .item(&app_menu)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .build()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![ownership_boundaries])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_menu(build_macos_menu(app.handle())?)?;
                if let Some(window) = app.get_webview_window("main") {
                    window.set_decorations(true)?;
                    window.set_title_bar_style(TitleBarStyle::Overlay)?;
                }
            }
            Ok(())
        })
        .on_menu_event(|app, event| {
            #[cfg(target_os = "macos")]
            {
                let id = event.id().0.as_str();
                if matches!(
                    id,
                    "new-workspace"
                        | "open-workspace"
                        | "open-image"
                        | "paste-image"
                        | "save-workspace"
                        | "save-as"
                        | "export-all"
                        | "share-workspace"
                        | "undo"
                        | "redo"
                        | "repeat-last-command"
                        | "command-palette"
                ) {
                    let _ = app.emit("fleck://native-menu", id);
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (app, event);
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to run Fleck desktop app");
}
