//! 网文管理 MCP 工具

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{Database, models::{Novel, CreateNovelRequest, UpdateNovelRequest}};
use super::super::{error::McpResult, protocol::ToolDescription};
use super::McpTool;

/// 列出所有网文工具
pub struct ListNovelsTool;

impl ListNovelsTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for ListNovelsTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "list_novels".to_string(),
            description: "获取所有网文小说列表，支持分页查询".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "返回结果的数量限制，默认返回全部",
                        "minimum": 1
                    },
                    "offset": {
                        "type": "integer",
                        "description": "偏移量，用于分页",
                        "minimum": 0,
                        "default": 0
                    },
                    "status": {
                        "type": "string",
                        "description": "按状态筛选，可选值：draft(草稿), active(进行中), completed(已完成), paused(已暂停)"
                    }
                },
                "required": []
            }),
            return_description: Some("返回小说列表数组，包含小说ID、标题、描述、状态、章节数、字数等信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, _params: &Value) -> McpResult<Value> {
        let conn = db.get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, cover_path, status, current_stage, total_chapters, total_words, created_at, updated_at 
             FROM novels ORDER BY updated_at DESC"
        )?;
        
        let novels: Vec<Novel> = stmt
            .query_map([], |row| {
                Ok(Novel {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    cover_path: row.get(3)?,
                    status: row.get(4)?,
                    current_stage: row.get(5)?,
                    total_chapters: row.get(6)?,
                    total_words: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(json!(novels))
    }
}

/// 获取单个网文详情工具
pub struct GetNovelTool;

impl GetNovelTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetNovelTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_novel".to_string(),
            description: "根据小说ID获取单个小说的详细信息".to_string(),
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
            return_description: Some("返回小说的详细信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;

        let conn = db.get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, cover_path, status, current_stage, total_chapters, total_words, created_at, updated_at 
             FROM novels WHERE id = ?1"
        )?;
        
        let novel: Novel = stmt
            .query_row([novel_id], |row| {
                Ok(Novel {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    cover_path: row.get(3)?,
                    status: row.get(4)?,
                    current_stage: row.get(5)?,
                    total_chapters: row.get(6)?,
                    total_words: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?;
        
        Ok(json!(novel))
    }
}

/// 创建网文工具
pub struct CreateNovelTool;

impl CreateNovelTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for CreateNovelTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "create_novel".to_string(),
            description: "创建一部新的网文小说".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "小说的标题"
                    },
                    "description": {
                        "type": "string",
                        "description": "小说的简介描述"
                    },
                    "cover_path": {
                        "type": "string",
                        "description": "封面图片的本地路径或URL"
                    }
                },
                "required": ["title"]
            }),
            return_description: Some("返回新创建的小说的详细信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 title 参数".to_string()))?;
        
        let description = params.get("description").and_then(|v| v.as_str());
        let cover_path = params.get("cover_path").and_then(|v| v.as_str());

        let request = CreateNovelRequest {
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
        };

        let conn = db.get_conn();
        let now = chrono::Local::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO novels (title, description, cover_path, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &request.title,
                &request.description,
                &cover_path,
                &now,
                &now,
            ),
        )?;
        
        let id = conn.last_insert_rowid();
        
        let novel = Novel {
            id,
            title: request.title,
            description: request.description,
            cover_path: cover_path.map(|s| s.to_string()),
            status: "draft".to_string(),
            current_stage: "novel".to_string(),
            total_chapters: 0,
            total_words: 0,
            created_at: now.clone(),
            updated_at: now,
        };
        
        Ok(json!(novel))
    }
}

/// 更新网文信息工具
pub struct UpdateNovelTool;

impl UpdateNovelTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for UpdateNovelTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "update_novel".to_string(),
            description: "更新已有小说的信息，支持增量更新".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "要更新的小说的ID"
                    },
                    "title": {
                        "type": "string",
                        "description": "新的小说标题"
                    },
                    "description": {
                        "type": "string",
                        "description": "新的小说简介"
                    },
                    "cover_path": {
                        "type": "string",
                        "description": "新的封面路径"
                    },
                    "status": {
                        "type": "string",
                        "description": "新的状态，可选值：draft, active, completed, paused"
                    },
                    "current_stage": {
                        "type": "string",
                        "description": "当前创作阶段，可选值：novel, audio, character, video"
                    }
                },
                "required": ["novel_id"]
            }),
            return_description: Some("返回更新是否成功".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;

        let request = UpdateNovelRequest {
            title: params.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: params.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            cover_path: params.get("cover_path").and_then(|v| v.as_str()).map(|s| s.to_string()),
            status: params.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
            current_stage: params.get("current_stage").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };

        let mut conn = db.get_conn();
        
        // 提取所有字段值
        let title_val = &request.title;
        let desc_val = &request.description;
        let cover_val = &request.cover_path;
        let status_val = &request.status;
        let stage_val = &request.current_stage;
        
        let has_update = title_val.is_some() 
            || desc_val.is_some() 
            || cover_val.is_some()
            || status_val.is_some()
            || stage_val.is_some();
        
        if !has_update {
            return Ok(json!({ "success": true, "message": "没有需要更新的字段" }));
        }
        
        // 修复 Bug: 使用事务方式更新
        let tx = conn.transaction()?;
        
        if let Some(title) = title_val {
            tx.execute(
                "UPDATE novels SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                (title, novel_id)
            )?;
        }
        if let Some(description) = desc_val {
            tx.execute(
                "UPDATE novels SET description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                (description, novel_id)
            )?;
        }
        if let Some(cover_path) = cover_val {
            tx.execute(
                "UPDATE novels SET cover_path = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                (cover_path, novel_id)
            )?;
        }
        if let Some(status) = status_val {
            tx.execute(
                "UPDATE novels SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                (status, novel_id)
            )?;
        }
        if let Some(stage) = stage_val {
            tx.execute(
                "UPDATE novels SET current_stage = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                (stage, novel_id)
            )?;
        }
        
        tx.commit()?;
        
        Ok(json!({ "success": true, "message": "更新成功" }))
    }
}

/// 删除网文工具
pub struct DeleteNovelTool;

impl DeleteNovelTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for DeleteNovelTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "delete_novel".to_string(),
            description: "删除指定的小说及其所有关联的章节和内容. 注意: 此操作不可恢复!".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "要删除的小说的ID"
                    }
                },
                "required": ["novel_id"]
            }),
            return_description: Some("返回删除是否成功".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;

        let conn = db.get_conn();
        
        // 删除关联章节
        conn.execute("DELETE FROM chapters WHERE novel_id = ?1", [novel_id])?;
        
        // 删除小说
        conn.execute("DELETE FROM novels WHERE id = ?1", [novel_id])?;
        
        Ok(json!({ "success": true, "message": "删除成功" }))
    }
}
