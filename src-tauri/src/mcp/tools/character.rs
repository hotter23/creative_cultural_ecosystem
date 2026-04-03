//! 角色管理 MCP 工具

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{Database, models::Character};
use crate::commands::audio::VOICE_CATEGORIES;
use super::super::{error::McpResult, protocol::ToolDescription};
use super::McpTool;

/// 列出小说角色工具
pub struct ListCharactersTool;

impl ListCharactersTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for ListCharactersTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "list_characters".to_string(),
            description: "获取指定小说的所有角色列表，可用于了解小说中的人物设定".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "小说的唯一标识符ID"
                    }
                },
                "required": ["novel_id"]
            }),
            return_description: Some("返回角色列表数组，包含角色的详细信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;

        let characters = db.get_characters_by_novel_id(novel_id as i64)?;
        Ok(json!(characters))
    }
}

/// 创建角色工具
pub struct CreateCharacterTool;

impl CreateCharacterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for CreateCharacterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "create_character".to_string(),
            description: "为小说创建一个新角色，可以在写作过程中动态添加人物".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "小说的唯一标识符ID"
                    },
                    "name": {
                        "type": "string",
                        "description": "角色的姓名"
                    },
                    "aliases": {
                        "type": "string",
                        "description": "角色的别名或绰号，多个用逗号分隔"
                    },
                    "gender": {
                        "type": "string",
                        "description": "角色的性别，可选值：male, female, other",
                        "enum": ["male", "female", "other"]
                    },
                    "role": {
                        "type": "string",
                        "description": "角色在故事中的定位，可选值：protagonist, supporting, antagonist, guest",
                        "enum": ["protagonist", "supporting", "antagonist", "guest"]
                    },
                    "description": {
                        "type": "string",
                        "description": "角色的详细描述，包括背景故事、性格特点、外貌特征等"
                    },
                    "appearance": {
                        "type": "string",
                        "description": "角色的外貌特征描述"
                    },
                    "personality": {
                        "type": "string",
                        "description": "角色的性格特点描述"
                    },
                    "tags": {
                        "type": "string",
                        "description": "角色标签，用于分类，多个用逗号分隔"
                    }
                },
                "required": ["novel_id", "name"]
            }),
            return_description: Some("返回创建成功的角色信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 name 参数".to_string()))?;

        let character = Character {
            id: 0,
            novel_id: novel_id as i64,
            name: name.to_string(),
            aliases: params.get("aliases").and_then(|v| v.as_str()).map(|s| s.to_string()),
            gender: params.get("gender").and_then(|v| v.as_str()).map(|s| s.to_string()),
            role: params.get("role").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: params.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            appearance: params.get("appearance").and_then(|v| v.as_str()).map(|s| s.to_string()),
            personality: params.get("personality").and_then(|v| v.as_str()).map(|s| s.to_string()),
            voice_id: None,
            tags: params.get("tags").and_then(|v| v.as_str()).map(|s| s.to_string()),
            created_at: chrono::Local::now().to_rfc3339(),
        };

        let char_id = db.create_character(&character)?;
        let result = Character {
            id: char_id,
            ..character
        };

        Ok(json!({
            "success": true,
            "message": "角色创建成功",
            "character": result
        }))
    }
}

/// 获取角色详情工具
pub struct GetCharacterTool;

impl GetCharacterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetCharacterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_character".to_string(),
            description: "获取指定角色的详细信息，包括角色的形象图片".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "角色的唯一标识符ID"
                    },
                    "include_images": {
                        "type": "boolean",
                        "description": "是否包含角色形象图片，默认为 false",
                        "default": false
                    }
                },
                "required": ["character_id"]
            }),
            return_description: Some("返回角色的详细信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let character_id = params
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 character_id 参数".to_string()))?;

        let include_images = params
            .get("include_images")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let character = db.get_character_by_id(character_id as i64)?
            .ok_or_else(|| super::super::error::McpError::InternalError(format!("角色 {} 不存在", character_id)))?;

        let mut result = json!(character);
        
        if include_images {
            let images = db.get_character_images(character_id as i64)?;
            if let Some(obj) = result.as_object_mut() {
                obj.insert("images".to_string(), json!(images));
            }
        }

        Ok(result)
    }
}

/// 更新角色工具
pub struct UpdateCharacterTool;

impl UpdateCharacterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for UpdateCharacterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "update_character".to_string(),
            description: "更新已存在的角色信息，可以在写作过程中完善角色设定".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "角色的唯一标识符ID"
                    },
                    "name": {
                        "type": "string",
                        "description": "角色的姓名"
                    },
                    "aliases": {
                        "type": "string",
                        "description": "角色的别名或绰号"
                    },
                    "gender": {
                        "type": "string",
                        "description": "角色的性别"
                    },
                    "role": {
                        "type": "string",
                        "description": "角色在故事中的定位"
                    },
                    "description": {
                        "type": "string",
                        "description": "角色的详细描述"
                    },
                    "appearance": {
                        "type": "string",
                        "description": "角色的外貌特征"
                    },
                    "personality": {
                        "type": "string",
                        "description": "角色的性格特点"
                    },
                    "tags": {
                        "type": "string",
                        "description": "角色标签"
                    }
                },
                "required": ["character_id"]
            }),
            return_description: Some("返回更新后的角色信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let character_id = params
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 character_id 参数".to_string()))?;

        // 检查角色是否存在
        let mut existing = db.get_character_by_id(character_id as i64)?
            .ok_or_else(|| super::super::error::McpError::InternalError(format!("角色 {} 不存在", character_id)))?;

        // 合并更新字段
        if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
            existing.name = name.to_string();
        }
        if let Some(aliases) = params.get("aliases").and_then(|v| v.as_str()) {
            existing.aliases = Some(aliases.to_string());
        }
        if let Some(gender) = params.get("gender").and_then(|v| v.as_str()) {
            existing.gender = Some(gender.to_string());
        }
        if let Some(role) = params.get("role").and_then(|v| v.as_str()) {
            existing.role = Some(role.to_string());
        }
        if let Some(description) = params.get("description").and_then(|v| v.as_str()) {
            existing.description = Some(description.to_string());
        }
        if let Some(appearance) = params.get("appearance").and_then(|v| v.as_str()) {
            existing.appearance = Some(appearance.to_string());
        }
        if let Some(personality) = params.get("personality").and_then(|v| v.as_str()) {
            existing.personality = Some(personality.to_string());
        }
        if let Some(tags) = params.get("tags").and_then(|v| v.as_str()) {
            existing.tags = Some(tags.to_string());
        }

        db.update_character(character_id as i64, &existing)?;

        // 获取更新后的角色信息
        let updated = db.get_character_by_id(character_id as i64)?
            .ok_or_else(|| super::super::error::McpError::InternalError("更新后角色不存在".to_string()))?;

        Ok(json!({
            "success": true,
            "message": "角色更新成功",
            "character": updated
        }))
    }
}

/// 删除角色工具
pub struct DeleteCharacterTool;

impl DeleteCharacterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for DeleteCharacterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "delete_character".to_string(),
            description: "删除指定的角色，谨慎使用！".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "角色的唯一标识符ID"
                    }
                },
                "required": ["character_id"]
            }),
            return_description: Some("返回删除结果".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let character_id = params
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 character_id 参数".to_string()))?;

        // 检查角色是否存在
        let existing = db.get_character_by_id(character_id as i64)?
            .ok_or_else(|| super::super::error::McpError::InternalError(format!("角色 {} 不存在", character_id)))?;

        db.delete_character(character_id as i64)?;

        Ok(json!({
            "success": true,
            "message": format!("角色「{}」已删除", existing.name)
        }))
    }
}

/// 列出小说角色图片工具
pub struct ListCharacterImagesTool;

impl ListCharacterImagesTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for ListCharacterImagesTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "list_character_images".to_string(),
            description: "获取角色的所有生成图片列表，用于展示角色的不同形象".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "角色的唯一标识符ID"
                    }
                },
                "required": ["character_id"]
            }),
            return_description: Some("返回角色图片列表数组".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let character_id = params
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 character_id 参数".to_string()))?;

        let images = db.list_character_images(character_id)?;
        Ok(json!(images))
    }
}

/// 绑定角色音色工具
pub struct BindCharacterVoiceTool;

impl BindCharacterVoiceTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for BindCharacterVoiceTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "bind_character_voice".to_string(),
            description: "为角色绑定音频音色，用于后续音频生成时使用指定音色".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "character_id": {
                        "type": "integer",
                        "description": "角色的唯一标识符ID"
                    },
                    "voice_id": {
                        "type": "string",
                        "description": "音色ID，如 'male-qn-qingse'、'female-yujie' 等"
                    }
                },
                "required": ["character_id", "voice_id"]
            }),
            return_description: Some("返回绑定结果".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let character_id = params
            .get("character_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 character_id 参数".to_string()))?;

        let voice_id = params
            .get("voice_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 voice_id 参数".to_string()))?;

        db.bind_character_voice(character_id, voice_id)?;
        
        Ok(json!({
            "success": true,
            "message": format!("角色音色绑定成功：{}", voice_id),
            "character_id": character_id,
            "voice_id": voice_id
        }))
    }
}

/// 获取可用音色列表工具
pub struct GetVoiceListTool;

impl GetVoiceListTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetVoiceListTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_voice_list".to_string(),
            description: "获取所有可用的音色列表，按语言（普通话、粤语）分类".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
            return_description: Some("返回分类的音色列表".to_string()),
        }
    }

    async fn call(&self, _db: &Arc<Database>, _params: &Value) -> McpResult<Value> {
        let voice_categories: Vec<Value> = VOICE_CATEGORIES.iter()
            .map(|(label, voices)| {
                json!({
                    "label": label,
                    "options": voices.iter()
                        .map(|(id, name)| json!({ "id": id, "name": name }))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        
        Ok(json!({
            "success": true,
            "voice_categories": voice_categories
        }))
    }
}
