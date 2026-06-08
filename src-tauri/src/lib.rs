mod desktop_commands;

use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri::{
    menu::{AboutMetadata, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    Emitter, TitleBarStyle,
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

#[cfg(test)]
mod tests {
    use super::desktop_commands::REGISTERED_TAURI_COMMANDS;
    use std::collections::BTreeSet;

    #[test]
    fn frontend_bridge_commands_are_registered_in_tauri() {
        let api_source = include_str!("../../src/lib/api.ts");
        let lib_source = include_str!("lib.rs");
        let invoked = bridge_commands(api_source);
        let registered = REGISTERED_TAURI_COMMANDS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let handler = handler_commands(lib_source);
        let missing = invoked.difference(&registered).copied().collect::<Vec<_>>();
        let not_in_handler = registered.difference(&handler).copied().collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "frontend bridge commands missing from Tauri registration: {missing:?}"
        );
        assert!(
            not_in_handler.is_empty(),
            "registered Tauri commands missing from generate_handler!: {not_in_handler:?}"
        );
    }

    fn bridge_commands(source: &str) -> BTreeSet<&str> {
        source
            .match_indices("bridge(\"")
            .filter_map(|(index, marker)| {
                let start = index + marker.len();
                let rest = &source[start..];
                let end = rest.find('"')?;
                Some(&rest[..end])
            })
            .collect()
    }

    fn handler_commands(source: &str) -> BTreeSet<&str> {
        let invoke_start = source
            .rfind(".invoke_handler(")
            .expect("invoke_handler block exists");
        let source = &source[invoke_start..];
        let start = source
            .find("tauri::generate_handler![")
            .expect("generate_handler block exists");
        let rest = &source[start..];
        let open = rest.find('[').expect("generate_handler opening bracket");
        let close = rest[open..]
            .find(']')
            .expect("generate_handler closing bracket");
        rest[open + 1..open + close]
            .split(',')
            .filter_map(|entry| {
                let name = entry.trim();
                if name.is_empty() {
                    return None;
                }
                Some(name.rsplit("::").next().unwrap_or(name))
            })
            .collect()
    }
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
        .manage(desktop_commands::DesktopState::default())
        .invoke_handler(tauri::generate_handler![
            ownership_boundaries,
            desktop_commands::get_workspace_meta,
            desktop_commands::get_layers,
            desktop_commands::get_image_objects,
            desktop_commands::get_export_areas,
            desktop_commands::get_history,
            desktop_commands::get_commands,
            desktop_commands::new_workspace,
            desktop_commands::open_workspace,
            desktop_commands::open_workspace_path,
            desktop_commands::save_workspace,
            desktop_commands::save_workspace_as,
            desktop_commands::get_recent_files,
            desktop_commands::pick_image_file,
            desktop_commands::acquire_clipboard_asset,
            desktop_commands::acquire_dropped_asset,
            desktop_commands::acquire_replacement_asset,
            desktop_commands::reveal_image_source,
            desktop_commands::relink_asset,
            desktop_commands::get_render_model,
            desktop_commands::get_viewport_focus,
            desktop_commands::create_export_area,
            desktop_commands::export_area,
            desktop_commands::export_all,
            desktop_commands::reveal_exported_file,
            desktop_commands::copy_export_result,
            desktop_commands::run_command,
            desktop_commands::undo,
            desktop_commands::redo,
            desktop_commands::jump_to_history,
            desktop_commands::supports_history_jump
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_menu(build_macos_menu(app.handle())?)?;
                if let Some(window) = app.get_webview_window("main") {
                    window.set_title_bar_style(TitleBarStyle::Overlay)?;
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_decorations(false)?;
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
