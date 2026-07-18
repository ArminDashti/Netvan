use crate::state::AppState;
use netvan_core::ipc::{RpcRequest, RpcResponse};
use tauri::State;

#[tauri::command]
pub async fn rpc(state: State<'_, AppState>, request: RpcRequest) -> Result<RpcResponse, String> {
    Ok(state.engine.handle(request).await)
}

#[tauri::command]
pub fn get_data_dir() -> Result<String, String> {
    netvan_core::paths::ensure_data_dir()
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}
