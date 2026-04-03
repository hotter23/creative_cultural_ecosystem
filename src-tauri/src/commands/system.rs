//! 系统相关 Tauri 命令

use std::sync::Arc;
use tauri::{command, State};

use crate::db::{Database, DbError};

// 保存视频配置
#[command]
pub async fn save_video_config(
    db: State<'_, Arc<Database>>,
    resolution: String,
    fps: i32,
    ffmpeg_path: Option<String>,
) -> Result<bool, String> {
    let fps_str = fps.to_string();
    
    let configs = vec![
        ("video_resolution", resolution.as_str(), "video"),
        ("video_fps", fps_str.as_str(), "video"),
    ];
    
    // 保存 FFmpeg 路径（如果提供）
    if let Some(path) = ffmpeg_path {
        if !path.is_empty() {
            db.set_config("video_ffmpeg_path", &path, Some("video"))
                .map_err(|e: DbError| e.to_string())?;
        }
    }
    
    db.set_config_batch(&configs)
        .map_err(|e: DbError| e.to_string())?;
    
    Ok(true)
}

// 获取视频配置
#[command]
pub async fn get_video_config(
    db: State<'_, Arc<Database>>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let configs = db.get_config_by_category("video")
        .map_err(|e: DbError| e.to_string())?;
    Ok(configs)
}

// 保存 Python 配置
#[command]
pub async fn save_python_config(
    db: State<'_, Arc<Database>>,
    python_path: Option<String>,
) -> Result<bool, String> {
    // 保存 Python 路径（如果提供）
    if let Some(path) = python_path {
        if !path.is_empty() {
            db.set_config("python_path", &path, Some("python"))
                .map_err(|e: DbError| e.to_string())?;
        }
    }
    
    Ok(true)
}

// 获取 Python 配置
#[command]
pub async fn get_python_config(
    db: State<'_, Arc<Database>>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let configs = db.get_config_by_category("python")
        .map_err(|e: DbError| e.to_string())?;
    Ok(configs)
}
