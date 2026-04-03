//! 日志相关命令

use crate::log;

/// 写入前端日志到文件
/// 前端可以通过 invoke('log_to_file', { message: '...' }) 调用
#[tauri::command]
pub fn log_to_file(message: String) {
    log!("[前端] {}", message);
}

/// 写入前端错误日志到文件
#[tauri::command]
pub fn log_error_to_file(message: String) {
    log!("[前端错误] {}", message);
}
