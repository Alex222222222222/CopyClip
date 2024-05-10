#[tauri::command]
pub async fn trace(msg: String) -> Result<(), String> {
    log::trace!("{}", msg);
    Ok(())
}
#[tauri::command]
pub async fn debug(msg: String) -> Result<(), String> {
    log::debug!("{}", msg);
    Ok(())
}

#[tauri::command]
pub async fn info(msg: String) -> Result<(), String> {
    log::info!("{}", msg);
    Ok(())
}

#[tauri::command]
pub async fn warn(msg: String) -> Result<(), String> {
    log::warn!("{}", msg);
    Ok(())
}
#[tauri::command]
pub async fn error(msg: String) -> Result<(), String> {
    log::error!("{}", msg);
    Ok(())
}
