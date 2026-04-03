//! 环境音功能 MCP 工具

use crate::log;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::path::PathBuf;
use std::process::Command;

use crate::db::Database;
use super::super::{error::McpResult, error::McpError, protocol::ToolDescription};
use super::McpTool;

/// 获取 Stable Audio exe 路径
fn get_stable_audio_exe_path() -> Option<PathBuf> {
    let exe_dir = match std::env::current_exe() {
        Ok(p) => p.parent()?.to_path_buf(),
        Err(_) => return None,
    };

    let possible_paths = vec![
        // 打包后的 exe（安装目录下）
        exe_dir.join("resources/stable_audio/stable_audio_inference.exe"),
        exe_dir.join("stable_audio/stable_audio_inference.exe"),
        // 开发环境：exe 在 src-tauri 同级目录的 stable_audio/
        exe_dir.join("../../stable_audio/stable_audio_inference.exe"),
        exe_dir.join("../../../stable_audio/stable_audio_inference.exe"),
    ];

    for path in &possible_paths {
        if path.exists() {
            log!("[MCP-StableAudio] 找到推理程序: {}", path.display());
            return Some(path.clone());
        }
    }
    None
}

/// 生成环境音（使用 Stable Audio Open）
pub struct GenerateAmbientSoundTool;

impl GenerateAmbientSoundTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GenerateAmbientSoundTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "generate_ambient_sound".to_string(),
            description: "使用 AI（Stable Audio Open）生成环境音。支持生成雨声、海浪、森林、咖啡馆等场景的环境音。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "环境音描述提示词（英文效果更好）\n示例：\n- rainy cafe with soft background chatter（雨声咖啡馆）\n- ocean waves crashing on beach（海浪声）\n- forest with birds singing（森林鸟鸣）\n- crackling fireplace（壁炉噼啪声）\n- rain on window（雨打窗户）"
                    },
                    "duration": {
                        "type": "number",
                        "description": "生成音频的时长（秒），范围 2-30 秒",
                        "minimum": 2,
                        "maximum": 30,
                        "default": 10
                    },
                    "name": {
                        "type": "string",
                        "description": "环境音名称（可选）"
                    }
                },
                "required": ["prompt"]
            }),
            return_description: Some("返回生成的环境音信息，包括文件路径、时长等".to_string()),
        }
    }

    async fn call(&self, _db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let prompt = params
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParameter("缺少 prompt 参数".to_string()))?;
        
        let duration = params.get("duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0) as u32;
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // 准备输出路径（使用exe所在目录）
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        
        let storage_dir = exe_dir
            .join("data")
            .join("ambient");
        
        std::fs::create_dir_all(&storage_dir)
            .map_err(|e| McpError::InternalError(e.to_string()))?;
        
        let file_name = format!("ambient_{}.wav", uuid::Uuid::new_v4());
        let output_path = storage_dir.join(&file_name);
        let output_path_str = output_path.to_string_lossy().to_string();

        // 1. 使用 Stable Audio exe
        if let Some(exe_path) = get_stable_audio_exe_path() {
            log!("[MCP-StableAudio] 使用推理程序: {}", exe_path.display());
            
            let output = Command::new(&exe_path)
                .args([
                    "--prompt", prompt,
                    "--duration", &duration.to_string(),
                    "--output", &output_path_str,
                ])
                .env("PYTHONIOENCODING", "utf-8")
                .output()
                .map_err(|e| McpError::InternalError(e.to_string()))?;

            let stderr_output = String::from_utf8_lossy(&output.stderr);
            if !stderr_output.trim().is_empty() {
                log!("[MCP-StableAudio] 调试信息: {}", stderr_output);
            }

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                log!("[MCP-StableAudio] 输出: {}", stdout);
                
                let stdout_str = stdout.trim();
                let actual_duration = if stdout_str.starts_with('{') {
                    if let Ok(json_result) = serde_json::from_str::<serde_json::Value>(stdout_str) {
                        json_result.get("duration")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(duration as f64) as u32
                    } else {
                        duration
                    }
                } else {
                    duration
                };

                // 将生成的环境音记录到数据库
                let ambient_name = name.clone().unwrap_or_else(|| format!("环境音_{}", chrono::Local::now().format("%Y%m%d_%H%M%S")));
                let ambient_id = _db.create_ambient_sound(
                    &ambient_name,
                    Some(prompt),
                    "ai_generated",
                    Some(prompt),
                    &output_path_str,
                    actual_duration as f32,
                    0.5,
                    true,
                ).map_err(|e| McpError::InternalError(format!("数据库记录创建失败: {}", e)))?;

                return Ok(json!({
                    "success": true,
                    "file_path": output_path_str,
                    "duration": actual_duration,
                    "name": ambient_name,
                    "ambient_id": ambient_id,
                    "method": "stable_audio"
                }));
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let error_msg = if stderr.trim().starts_with('{') {
                    if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(stderr.trim()) {
                        error_json.get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未知错误")
                            .to_string()
                    } else {
                        stderr.to_string()
                    }
                } else {
                    stderr.to_string()
                };
                
                log!("[MCP-StableAudio] Stable Audio 执行失败: {}", error_msg);
            }
        }

        // 2. 回退到 FFmpeg 白噪音模拟
        log!("[MCP-StableAudio] 使用 FFmpeg 白噪音模拟");
        
        let filter = if prompt.to_lowercase().contains("雨") || prompt.to_lowercase().contains("rain") {
            "lowpass=f=800,volume=0.3"
        } else if prompt.to_lowercase().contains("海") || prompt.to_lowercase().contains("wave") {
            "lowpass=f=400,volume=0.25"
        } else if prompt.to_lowercase().contains("森林") || prompt.to_lowercase().contains("forest") {
            "bandpass=f=500:width_type=h:w=1500,volume=0.2"
        } else {
            "volume=0.2"
        };

        let output = Command::new("ffmpeg")
            .args([
                "-f", "lavfi",
                "-i", &format!("anoisesrc=d={}:c=2:color=brown", duration as f64),
                "-ar", "24000",
                "-filter:a", filter,
                "-y",
                &output_path_str,
            ])
            .output()
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::InternalError(
                format!("FFmpeg 生成失败: {}", stderr)
            ));
        }

        // 将生成的环境音记录到数据库
        let ambient_name = name.unwrap_or_else(|| format!("环境音_{}", chrono::Local::now().format("%Y%m%d_%H%M%S")));
        let ambient_id = _db.create_ambient_sound(
            &ambient_name,
            Some(prompt),
            "ai_generated",
            Some(prompt),
            &output_path_str,
            duration as f32,
            0.5,
            true,
        ).map_err(|e| McpError::InternalError(format!("数据库记录创建失败: {}", e)))?;

        Ok(json!({
            "success": true,
            "file_path": output_path_str,
            "duration": duration,
            "name": ambient_name,
            "ambient_id": ambient_id,
            "method": "ffmpeg_simulation",
            "note": "Stable Audio 未安装，使用 FFmpeg 白噪音模拟"
        }))
    }
}
