use serde::Deserialize;
use std::sync::atomic::Ordering;
use tauri::State;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardQuery {
    pub query: Option<String>,
    pub filters: Option<crate::database::Filters>,
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemUpdate {
    pub content: Option<String>,
    pub label: Option<String>,
}
fn db_error() -> String {
    "Clipboard+ could not access its local database.".into()
}
fn input_error() -> String {
    "That value is not valid.".into()
}
#[tauri::command]
pub fn list_clipboard_items(
    state: State<'_, crate::AppState>,
    query: ClipboardQuery,
) -> Result<Vec<crate::database::ClipboardItem>, String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .list(
            query.query.as_deref().unwrap_or(""),
            &query.filters.unwrap_or_default(),
            query.limit.unwrap_or(100).clamp(1, 500),
        )
        .map_err(|_| db_error())
}
#[tauri::command]
pub fn copy_clipboard_item(state: State<'_, crate::AppState>, id: String) -> Result<(), String> {
    let content = state
        .database
        .lock()
        .map_err(|_| db_error())?
        .content_by_id(&id)
        .map_err(|_| db_error())?
        .ok_or("Clipboard item not found.")?;
    crate::clipboard::write_text(&content)
}
#[tauri::command]
pub fn delete_clipboard_item(state: State<'_, crate::AppState>, id: String) -> Result<(), String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .delete(&id)
        .map_err(|_| db_error())
}
#[tauri::command]
pub fn clear_clipboard_history(state: State<'_, crate::AppState>) -> Result<(), String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .clear()
        .map_err(|_| db_error())
}
#[tauri::command]
pub fn set_item_pinned(
    state: State<'_, crate::AppState>,
    id: String,
    pinned: bool,
) -> Result<(), String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .pin(&id, pinned)
        .map_err(|_| db_error())
}
#[tauri::command]
pub fn assign_item_group(
    state: State<'_, crate::AppState>,
    id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .set_group(&id, group_id.as_deref())
        .map_err(|_| input_error())
}
#[tauri::command]
pub fn update_clipboard_item(
    state: State<'_, crate::AppState>,
    id: String,
    update: ItemUpdate,
) -> Result<(), String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .update(&id, update.content.as_deref(), update.label.as_deref())
        .map_err(|_| input_error())
}
#[tauri::command]
pub fn list_groups(
    state: State<'_, crate::AppState>,
) -> Result<Vec<crate::database::Group>, String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .groups()
        .map_err(|_| db_error())
}
#[tauri::command]
pub fn create_group(
    state: State<'_, crate::AppState>,
    name: String,
) -> Result<crate::database::Group, String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .create_group(&name)
        .map_err(|_| input_error())
}
#[tauri::command]
pub fn get_retention_settings(
    state: State<'_, crate::AppState>,
) -> Result<crate::database::RetentionSettings, String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .settings()
        .map_err(|_| db_error())
}
#[tauri::command]
pub fn update_retention_settings(
    state: State<'_, crate::AppState>,
    settings: crate::database::RetentionSettings,
) -> Result<(), String> {
    state
        .database
        .lock()
        .map_err(|_| db_error())?
        .save_settings(settings)
        .map_err(|_| input_error())
}
#[tauri::command]
pub fn set_monitoring_paused(state: State<'_, crate::AppState>, paused: bool) {
    state.monitoring_paused.store(paused, Ordering::Relaxed)
}
#[tauri::command]
pub fn is_monitoring_paused(state: State<'_, crate::AppState>) -> bool {
    state.monitoring_paused.load(Ordering::Relaxed)
}
