//! 章节音频管理 MCP 工具

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::Database;
use super::super::{error::McpResult, protocol::ToolDescription};
use super::McpTool;

/// 获取章节音频详情工具
pub struct GetChapterAudioDetailTool;

impl GetChapterAudioDetailTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetChapterAudioDetailTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_chapter_audio_detail".to_string(),
            description: "获取章节的音频制作详情和进度信息，包括音频状态、完成度等".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节ID"
                    },
                    "chapterId": {
                        "type": "integer",
                        "description": "章节ID（驼峰命名，兼容旧版本）"
                    }
                },
                "required": []
            }),
            return_description: Some("返回章节音频详情信息，如果未创建音频则返回 null".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params
            .get("chapter_id")
            .and_then(|v| v.as_i64())
            .or_else(|| params.get("chapterId").and_then(|v| v.as_i64()));
        
        let Some(cid) = chapter_id else {
            return Ok(json!({
                "success": false,
                "error": "缺少 chapter_id 或 chapterId 参数"
            }));
        };

        match db.get_chapter_audio_by_chapter(cid) {
            Ok(Some(audio)) => {
                Ok(json!({
                    "success": true,
                    "data": {
                        "id": audio.id,
                        "novel_id": audio.novel_id,
                        "chapter_id": audio.chapter_id,
                        "status": audio.status,
                        "total_sentences": audio.total_sentences,
                        "completed_sentences": audio.completed_sentences,
                        "merged_audio_path": audio.merged_audio_path,
                        "created_at": audio.created_at
                    }
                }))
            }
            Ok(None) => {
                Ok(json!({
                    "success": true,
                    "data": null,
                    "message": "该章节尚未创建音频任务"
                }))
            }
            Err(e) => {
                Ok(json!({
                    "success": false,
                    "error": format!("查询失败: {}", e)
                }))
            }
        }
    }
}

/// 获取音频段落列表工具
pub struct GetAudioParagraphsTool;

impl GetAudioParagraphsTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetAudioParagraphsTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_audio_paragraphs".to_string(),
            description: "获取音频段落列表，通过章节ID优先获取已标注的段落信息，用于音频制作".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节ID（优先使用）"
                    },
                    "chapterId": {
                        "type": "integer",
                        "description": "章节ID（驼峰命名，兼容）"
                    },
                    "audio_id": {
                        "type": "integer",
                        "description": "音频ID（备选）"
                    },
                    "audioId": {
                        "type": "integer",
                        "description": "音频ID（驼峰命名，兼容）"
                    }
                },
                "required": []
            }),
            return_description: Some("返回段落列表，包含内容、类型、绑定角色、音频状态等信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params
            .get("chapter_id")
            .and_then(|v| v.as_i64())
            .or_else(|| params.get("chapterId").and_then(|v| v.as_i64()));
        
        let audio_id = params
            .get("audio_id")
            .and_then(|v| v.as_i64())
            .or_else(|| params.get("audioId").and_then(|v| v.as_i64()));

        // 优先通过 chapter_id 获取
        let paragraphs = if let Some(cid) = chapter_id {
            db.get_paragraphs_by_chapter_id(cid)
        } else if let Some(aid) = audio_id {
            db.get_paragraphs_by_audio_id(aid)
        } else {
            return Ok(json!({
                "success": false,
                "error": "需要提供 chapter_id/chapterId 或 audio_id/audioId 参数"
            }));
        };

        match paragraphs {
            Ok(list) => {
                let result: Vec<Value> = list.into_iter().map(|p| json!({
                    "id": p.id,
                    "chapter_id": p.chapter_id,
                    "paragraph_index": p.paragraph_index,
                    "content": p.content,
                    "type": p.r#type,
                    "character_id": p.character_id,
                    "audio_id": p.audio_id,
                    "speed": p.speed,
                    "pitch": p.pitch,
                    "volume": p.volume,
                    "emotion": p.emotion,
                    "audio_path": p.audio_path,
                    "duration": p.duration,
                    "status": p.status,
                    "error_msg": p.error_msg,
                    "task_id": p.task_id
                })).collect();

                Ok(json!({
                    "success": true,
                    "data": result,
                    "total": result.len(),
                    "completed_count": result.iter().filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("completed")).count()
                }))
            }
            Err(e) => {
                Ok(json!({
                    "success": false,
                    "error": format!("查询段落失败: {}", e)
                }))
            }
        }
    }
}

/// 生成章节音频任务工具
pub struct GenerateChapterAudioTool;

impl GenerateChapterAudioTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GenerateChapterAudioTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "generate_chapter_audio".to_string(),
            description: "为章节创建音频生成任务，基于已标注的段落开始生成音频".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "小说ID"
                    },
                    "novelId": {
                        "type": "integer",
                        "description": "小说ID（兼容）"
                    },
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节ID"
                    },
                    "chapterId": {
                        "type": "integer",
                        "description": "章节ID（兼容）"
                    },
                    "chapter_title": {
                        "type": "string",
                        "description": "章节标题"
                    },
                    "chapterTitle": {
                        "type": "string",
                        "description": "章节标题（兼容）"
                    },
                    "chapter_content": {
                        "type": "string",
                        "description": "章节内容（可选，优先从数据库读取）"
                    },
                    "chapterContent": {
                        "type": "string",
                        "description": "章节内容（兼容）"
                    },
                    "voice_id": {
                        "type": "string",
                        "description": "默认音色ID，用于未绑定角色的旁白段落"
                    },
                    "voiceId": {
                        "type": "string",
                        "description": "默认音色ID（兼容）"
                    },
                    "speed": {
                        "type": "number",
                        "description": "默认语速，0.5-2.0之间，默认1.0",
                        "default": 1.0
                    }
                },
                "required": ["chapter_id"]
            }),
            return_description: Some("返回生成的音频任务ID".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .or_else(|| params.get("novelId").and_then(|v| v.as_i64()));
        
        let chapter_id = params
            .get("chapter_id")
            .and_then(|v| v.as_i64())
            .or_else(|| params.get("chapterId").and_then(|v| v.as_i64()));
        
        let voice_id = params
            .get("voice_id")
            .and_then(|v| v.as_str())
            .or_else(|| params.get("voiceId").and_then(|v| v.as_str()));
        
        let speed = params
            .get("speed")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        let Some(cid) = chapter_id else {
            return Ok(json!({
                "success": false,
                "error": "缺少 chapter_id 或 chapterId 参数"
            }));
        };

        // 获取小说ID（优先使用传入的，否则从段落信息中推断）
        let nid = if let Some(id) = novel_id {
            id
        } else {
            // 从段落中获取小说ID
            match db.get_paragraphs_by_chapter_id(cid) {
                Ok(paragraphs) => {
                    if let Some(_first_para) = paragraphs.first() {
                        // 尝试从章节中获取小说ID - 需要先获取章节
                        // 这里简化处理：从novel_id参数或要求必须传入
                        return Ok(json!({
                            "success": false,
                            "error": "请提供 novel_id 参数以便创建音频任务"
                        }));
                    } else {
                        return Ok(json!({
                            "success": false,
                            "error": "该章节暂无标注段落，请先进行段落标注再创建音频任务"
                        }));
                    }
                }
                Err(_) => {
                    return Ok(json!({
                        "success": false,
                        "error": "无法获取章节信息，请确保提供正确的 chapter_id 和 novel_id"
                    }));
                }
            }
        };

        // 检查是否已有音频任务
        match db.get_chapter_audio_by_chapter(cid) {
            Ok(Some(_audio)) => {
                return Ok(json!({
                    "success": false,
                    "error": "该章节已有音频任务，请使用 regenerate_chapter_audio 重新生成"
                }));
            }
            Ok(None) => {},
            Err(e) => {
                return Ok(json!({
                    "success": false,
                    "error": format!("检查现有任务失败: {}", e)
                }));
            }
        }

        // 检查是否有段落标注
        match db.get_paragraphs_by_chapter_id(cid) {
            Ok(paragraphs) => {
                if paragraphs.is_empty() {
                    return Ok(json!({
                        "success": false,
                        "error": "该章节暂无手动标注的段落，请先在章节编辑中进行标注"
                    }));
                }
            }
            Err(e) => {
                return Ok(json!({
                    "success": false,
                    "error": format!("获取段落列表失败: {}", e)
                }));
            }
        }

        // 创建音频任务
        let default_voice = voice_id.unwrap_or("female-tianmei");
        match db.create_chapter_audio(nid, Some(cid)) {
            Ok(audio_id) => {
                Ok(json!({
                    "success": true,
                    "audio_id": audio_id,
                    "chapter_id": cid,
                    "novel_id": nid,
                    "default_voice": default_voice,
                    "speed": speed,
                    "message": "音频任务创建成功，请等待后台生成"
                }))
            }
            Err(e) => {
                Ok(json!({
                    "success": false,
                    "error": format!("创建音频任务失败: {}", e)
                }))
            }
        }
    }
}
