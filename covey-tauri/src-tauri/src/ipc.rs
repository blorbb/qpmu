use color_eyre::eyre::Result;
use covey::config::GlobalConfig;
use covey_manifest::PluginManifest;
use covey_tauri_types::{Event, ListItemId};
use tauri::{ipc::Channel, Manager, State, WebviewWindow, WebviewWindowBuilder};

use crate::state::{AppState, EventChannel};

#[tauri::command]
pub fn setup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    events: Channel<Event>,
) -> Result<(), String> {
    setup_impl(app, state, events).map_err(|e| format!("{e:#}"))
}

fn setup_impl(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    channel: Channel<Event>,
) -> Result<()> {
    let frontend = EventChannel { channel, app };
    state.init(frontend)?;
    Ok(())
}

#[tauri::command]
pub fn query(state: State<'_, AppState>, text: String) {
    tokio::spawn(state.host().query(text));
}

#[tauri::command]
pub fn activate(state: State<'_, AppState>, list_item_id: ListItemId, command_name: String) {
    let state = &state;
    let id = list_item_id;
    let item = state.find_list_item(&id);

    if let Some(item) = item {
        tokio::spawn(state.host().activate(item, command_name));
    } else {
        tracing::warn!("list item with id {id:?} not found")
    }
}

#[tauri::command]
pub fn show_settings_window(app: tauri::AppHandle) {
    let window = app.get_webview_window("settings").unwrap_or_else(|| {
        let settings_window = app
            .config()
            .app
            .windows
            .iter()
            .find(|window| window.label == "settings")
            .expect("app config should have settings window");

        WebviewWindowBuilder::from_config(&app, settings_window)
            .unwrap()
            .build()
            .unwrap()
    });

    window.show().unwrap();
    window.set_focus().unwrap();
}

#[tauri::command]
pub fn get_global_config(state: State<'_, AppState>) -> GlobalConfig {
    state.host().config()
}

#[tauri::command]
pub fn set_global_config(state: State<'_, AppState>, config: GlobalConfig) {
    tracing::debug!("received global config {config:#?}");
    state.host().reload(config)
}

#[tauri::command]
pub fn get_manifest(state: State<'_, AppState>, plugin_name: String) -> Option<PluginManifest> {
    state
        .host()
        .plugins()
        .get(&*plugin_name)
        .map(|plugin| plugin.manifest())
        .cloned()
}
