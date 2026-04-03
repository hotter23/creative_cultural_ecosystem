//! 音频管理 MCP 工具

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::clients::minimax::{MinMaxConfig, MinMaxTTSClient};
use crate::db::Database;
use super::super::{error::McpResult, protocol::ToolDescription};
use super::McpTool;

/// 文本转语音工具
pub struct TextToSpeechTool;

impl TextToSpeechTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for TextToSpeechTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "text_to_speech".to_string(),
            description: "将文本转换为语音音频，支持多种音色和语速调节。用于有声书制作和角色配音。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "要转换为语音的文本内容（建议每段不超过500字）"
                    },
                    "voice_id": {
                        "type": "string",
                        "description": "音色ID，支持：voice_female_01(女声1), voice_female_02(女声2), voice_male_01(男声1), voice_male_02(男声2), voice_child(儿童声), voice_elderly(老年声)"
                    },
                    "speed": {
                        "type": "number",
                        "description": "语速，范围：0.5（慢速）- 2.0（快速），1.0为正常速度",
                        "minimum": 0.5,
                        "maximum": 2.0,
                        "default": 1.0
                    },
                    "pitch": {
                        "type": "integer",
                        "description": "音调，范围：-10到10，0为正常音调",
                        "minimum": -10,
                        "maximum": 10,
                        "default": 0
                    },
                    "volume": {
                        "type": "integer",
                        "description": "音量，范围：0（静音）- 100（最大）",
                        "minimum": 0,
                        "maximum": 100,
                        "default": 80
                    },
                    "chapter_id": {
                        "type": "integer",
                        "description": "关联的章节ID（可选，用于自动保存音频）"
                    },
                    "save_to_file": {
                        "type": "boolean",
                        "description": "是否保存为文件",
                        "default": true
                    }
                },
                "required": ["text"]
            }),
            return_description: Some("返回音频文件路径或音频数据信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 text 参数".to_string()))?;
        
        if text.is_empty() {
            return Err(super::super::error::McpError::InvalidParameter("text 参数不能为空".to_string()));
        }

        let voice_id = params.get("voice_id").and_then(|v| v.as_str()).unwrap_or("voice_female_01");
        let speed = params.get("speed").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        let _pitch = params.get("pitch").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let _volume = params.get("volume").and_then(|v| v.as_i64()).unwrap_or(80) as i32;

        // 加载配置
        let config_map = db.get_config_by_category("minimax")
            .map_err(|e| super::super::error::McpError::DatabaseError(e.to_string()))?;
        
        let enabled = config_map.get("minimax_enabled")
            .and_then(|v| v.parse().ok())
            .unwrap_or(false);
        
        if !enabled {
            return Err(super::super::error::McpError::PermissionDenied(
                "MinMax 服务未启用，请先在设置中配置并启用 MinMax".to_string()
            ));
        }
        
        let api_key = config_map.get("minimax_api_key")
            .cloned()
            .unwrap_or_default();
        
        if api_key.is_empty() {
            return Err(super::super::error::McpError::PermissionDenied(
                "MinMax API Key 未配置，请先在设置中配置 API Key".to_string()
            ));
        }

        let config = MinMaxConfig {
            api_key,
            base_url: config_map.get("minimax_base_url")
                .cloned()
                .unwrap_or_else(|| "https://api.minimaxi.com".to_string()),
            default_model: String::new(),
            group_id: config_map.get("minimax_group_id").cloned(),
            enabled: true,
        };
        
        let client = MinMaxTTSClient::new(config);
        
        // 调用 TTS 服务
        let audio_data = client.text_to_speech(text, voice_id, speed).await
            .map_err(super::super::error::McpError::InternalError)?;
        
        // 生成文件名
        let file_name = format!("tts_{}.mp3", uuid::Uuid::new_v4());
        
        // 确保音频目录存在
        let audio_dir = std::path::Path::new("audio");
        if !audio_dir.exists() {
            std::fs::create_dir_all(audio_dir)
                .map_err(|e| super::super::error::McpError::InternalError(format!("创建目录失败: {}", e)))?;
        }
        
        let file_path = audio_dir.join(&file_name);
        
        // 保存音频文件
        std::fs::write(&file_path, &audio_data)
            .map_err(|e| super::super::error::McpError::InternalError(format!("保存文件失败: {}", e)))?;
        
        Ok(json!({
            "success": true,
            "message": "语音生成成功",
            "file_path": file_path.to_str().unwrap_or(&file_name),
            "file_name": file_name,
            "text_length": text.len(),
            "voice_id": voice_id,
            "speed": speed,
            "audio_size_bytes": audio_data.len()
        }))
    }
}
