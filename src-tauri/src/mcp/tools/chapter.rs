//! 章节管理 MCP 工具

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{Database, models::{Chapter, UpdateChapterRequest, ParagraphMarkRequest}};
use super::super::{error::McpResult, protocol::ToolDescription};
use super::McpTool;

/// 列出小说章节工具
pub struct ListChaptersTool;

impl ListChaptersTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for ListChaptersTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "list_chapters".to_string(),
            description: "获取指定小说的所有章节列表，按章节序号排序".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "小说的唯一标识符ID"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "返回结果的数量限制",
                        "minimum": 1
                    },
                    "offset": {
                        "type": "integer",
                        "description": "偏移量，用于分页",
                        "minimum": 0,
                        "default": 0
                    }
                },
                "required": ["novel_id"]
            }),
            return_description: Some("返回章节列表数组".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;

        let conn = db.get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, novel_id, title, content, plain_text, order_num, word_count, status, created_at 
             FROM chapters WHERE novel_id = ?1 ORDER BY order_num ASC"
        )?;
        
        let chapters: Vec<Chapter> = stmt
            .query_map([novel_id], |row| {
                Ok(Chapter {
                    id: row.get(0)?,
                    novel_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    plain_text: row.get(4)?,
                    order_num: row.get(5)?,
                    word_count: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(json!(chapters))
    }
}

/// 获取章节详情工具
pub struct GetChapterTool;

impl GetChapterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetChapterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_chapter".to_string(),
            description: "根据章节ID获取单个章节的详细内容".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节的唯一标识符ID"
                    }
                },
                "required": ["chapter_id"]
            }),
            return_description: Some("返回章节的详细信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params
            .get("chapter_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 chapter_id 参数".to_string()))?;

        let conn = db.get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, novel_id, title, content, plain_text, order_num, word_count, status, created_at 
             FROM chapters WHERE id = ?1"
        )?;
        
        let chapter: Chapter = stmt
            .query_row([chapter_id], |row| {
                Ok(Chapter {
                    id: row.get(0)?,
                    novel_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    plain_text: row.get(4)?,
                    order_num: row.get(5)?,
                    word_count: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?;
        
        Ok(json!(chapter))
    }
}

/// 创建章节工具
pub struct CreateChapterTool;

impl CreateChapterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for CreateChapterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "create_chapter".to_string(),
            description: "为指定小说创建一个新章节".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "要添加章节的小说ID"
                    },
                    "title": {
                        "type": "string",
                        "description": "章节标题"
                    },
                    "content": {
                        "type": "string",
                        "description": "章节内容（支持HTML格式）"
                    },
                    "order_num": {
                        "type": "integer",
                        "description": "章节排序号，默认为当前最大序号+1"
                    },
                    "status": {
                        "type": "string",
                        "description": "章节状态，可选值：draft(草稿), published(已发布)",
                        "default": "draft"
                    }
                },
                "required": ["novel_id", "title"]
            }),
            return_description: Some("返回新创建的章节信息".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;
        
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 title 参数".to_string()))?;
        
        let content = params.get("content").and_then(|v| v.as_str());
        let status = params.get("status").and_then(|v| v.as_str()).unwrap_or("draft");
        
        // 自动计算 order_num
        let conn = db.get_conn();
        let order_num = if let Some(num) = params.get("order_num").and_then(|v| v.as_i64()) {
            num as i32
        } else {
            let max_order: Result<i32, _> = conn
                .query_row("SELECT IFNULL(MAX(order_num), 0) + 1 FROM chapters WHERE novel_id = ?1", [novel_id], |row| row.get(0));
            max_order.unwrap_or(1)
        };

        let word_count = content.map(|c| c.chars().count()).unwrap_or(0) as i32;
        let plain_text = content.map(|c| html2text::from_read(c.as_bytes(), 10000));
        let now = chrono::Local::now().to_rfc3339();

        // 修复 Bug: 使用正确的参数数组形式，Option<T> 需要统一类型处理
        let content_str = content.unwrap_or("");
        let plain_text_str = plain_text.as_deref().unwrap_or("");

        conn.execute(
            "INSERT INTO chapters (novel_id, title, content, plain_text, order_num, word_count, status, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                novel_id,
                title,
                content_str,
                plain_text_str,
                order_num,
                word_count,
                status,
                &now,
            ),
        )?;
        
        let id = conn.last_insert_rowid();
        
        // 更新小说统计
        super::super::super::commands::chapter::update_novel_stats(&conn, novel_id)
            .map_err(|e| super::super::error::McpError::DatabaseError(e.to_string()))?;
        
        let chapter = Chapter {
            id,
            novel_id,
            title: title.to_string(),
            content: content.map(|s| s.to_string()),
            plain_text,
            order_num,
            word_count,
            status: status.to_string(),
            created_at: now,
        };
        
        Ok(json!(chapter))
    }
}

/// 更新章节工具
pub struct UpdateChapterTool;

impl UpdateChapterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for UpdateChapterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "update_chapter".to_string(),
            description: "更新章节的内容或元数据，支持增量更新".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "要更新的章节ID"
                    },
                    "title": {
                        "type": "string",
                        "description": "新的章节标题"
                    },
                    "content": {
                        "type": "string",
                        "description": "新的章节内容"
                    },
                    "order_num": {
                        "type": "integer",
                        "description": "新的排序号"
                    },
                    "status": {
                        "type": "string",
                        "description": "新的状态：draft, published"
                    }
                },
                "required": ["chapter_id"]
            }),
            return_description: Some("返回更新结果".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params
            .get("chapter_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 chapter_id 参数".to_string()))?;

        let request = UpdateChapterRequest {
            title: params.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
            content: params.get("content").and_then(|v| v.as_str()).map(|s| s.to_string()),
            plain_text: None,
            order_num: params.get("order_num").and_then(|v| v.as_i64()).map(|n| n as i32),
            status: params.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };

        let mut conn = db.get_conn();
        
        // 提取所有字段值
        let title_val = &request.title;
        let content_val = &request.content;
        let order_num_val = &request.order_num;
        let status_val = &request.status;
        
        let has_update = title_val.is_some() 
            || content_val.is_some() 
            || order_num_val.is_some()
            || status_val.is_some();
        
        if !has_update {
            return Ok(json!({ "success": true, "message": "没有需要更新的字段" }));
        }
        
        // 修复 Bug: rusqlite 不支持复杂的动态参数，改用多个独立 execute 或使用不同的参数策略
        // 这里改用更可靠的逐字段更新方式（在事务中）
        let tx = conn.transaction()?;
        
        if let Some(title) = title_val {
            tx.execute("UPDATE chapters SET title = ? WHERE id = ?", (title, chapter_id))?;
        }
        if let Some(content) = content_val {
            let plain_text = html2text::from_read(content.as_bytes(), 10000);
            let word_count = content.chars().count() as i32;
            tx.execute(
                "UPDATE chapters SET content = ?, plain_text = ?, word_count = ? WHERE id = ?",
                (content, &plain_text, word_count, chapter_id)
            )?;
        }
        if let Some(order_num) = order_num_val {
            tx.execute("UPDATE chapters SET order_num = ? WHERE id = ?", (order_num, chapter_id))?;
        }
        if let Some(status) = status_val {
            tx.execute("UPDATE chapters SET status = ? WHERE id = ?", (status, chapter_id))?;
        }
        
        tx.commit()?;
        
        // 更新小说统计
        let novel_id: i64 = conn
            .query_row("SELECT novel_id FROM chapters WHERE id = ?1", [chapter_id], |row| row.get(0))?;
        super::super::super::commands::chapter::update_novel_stats(&conn, novel_id)
            .map_err(|e| super::super::error::McpError::DatabaseError(e.to_string()))?;
        
        Ok(json!({ "success": true, "message": "更新成功" }))
    }
}

/// 删除章节工具
pub struct DeleteChapterTool;

impl DeleteChapterTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for DeleteChapterTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "delete_chapter".to_string(),
            description: "删除指定的章节. 注意: 此操作不可恢复!".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "要删除的章节ID"
                    }
                },
                "required": ["chapter_id"]
            }),
            return_description: Some("返回删除结果".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params
            .get("chapter_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 chapter_id 参数".to_string()))?;

        let conn = db.get_conn();
        
        // 获取小说ID用于更新统计
        let novel_id: Result<i64, _> = conn
            .query_row("SELECT novel_id FROM chapters WHERE id = ?1", [chapter_id], |row| row.get(0));
        
        // 删除章节
        conn.execute("DELETE FROM chapters WHERE id = ?1", [chapter_id])?;
        
        // 更新小说统计
        if let Ok(novel_id) = novel_id {
            let _ = super::super::super::commands::chapter::update_novel_stats(&conn, novel_id);
        }
        
        Ok(json!({ "success": true, "message": "删除成功" }))
    }
}

// =============================================================================
// 段落分析与标注工具
// =============================================================================

/// 简单清理HTML标签
fn clean_html_tags(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    
    // 清理多余的空白字符
    let result = result
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");
    
    result
}

/// 手动标注段落类型工具（支持手动操作场景）
pub struct BatchMarkParagraphsTool;

impl BatchMarkParagraphsTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for BatchMarkParagraphsTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "batch_mark_paragraphs".to_string(),
            description: "手动标注段落类型（旁白/对话/环境音）和绑定角色，支持完整的手动操作流程".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节ID，用于关联保存标注结果"
                    },
                    "paragraphs": {
                        "type": "array",
                        "description": "段落列表及其标注，包含手动编辑后的内容",
                        "items": {
                            "type": "object",
                            "properties": {
                                "index": {
                                    "type": "integer",
                                    "description": "段落索引"
                                },
                                "content": {
                                    "type": "string",
                                    "description": "段落内容（手动拆分后的）"
                                },
                                "type": {
                                    "type": "string",
                                    "description": "段落类型：narration（旁白）、dialogue（对话）、environment（环境音）",
                                    "enum": ["narration", "dialogue", "environment"]
                                },
                                "character_id": {
                                    "type": "integer",
                                    "description": "角色ID（对话类型需要）"
                                }
                            },
                            "required": ["index", "content", "type"]
                        }
                    },
                    "save_to_db": {
                        "type": "boolean",
                        "description": "是否保存到数据库",
                        "default": false
                    }
                },
                "required": ["paragraphs"]
            }),
            return_description: Some("返回标注结果".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params.get("chapter_id").and_then(|v| v.as_i64());
        let paragraphs = params.get("paragraphs").and_then(|v| v.as_array());
        let save_to_db = params.get("save_to_db").and_then(|v| v.as_bool()).unwrap_or(false);

        let Some(para_list) = paragraphs else {
            return Ok(json!({
                "success": false,
                "message": "缺少 paragraphs 参数"
            }));
        };

        // 处理标注
        let mut marked_count = 0;
        let mut dialogue_count = 0;
        let mut narration_count = 0;
        let mut environment_count = 0;

        let mut result_paragraphs = Vec::new();

        for para in para_list {
            let content = para.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let p_type = para.get("type").and_then(|v| v.as_str()).unwrap_or("narration");
            let character_id = para.get("character_id").and_then(|v| v.as_i64());
            
            // 查询角色名称
            let character_name = if let Some(cid) = character_id {
                let conn = db.get_conn();
                let result: Result<String, _> = conn.query_row(
                    "SELECT name FROM characters WHERE id = ?1",
                    [cid],
                    |row| row.get(0)
                );
                result.ok()
            } else {
                None
            };

            // 统计
            match p_type {
                "dialogue" => {
                    dialogue_count += 1;
                    if character_id.is_some() {
                        marked_count += 1;
                    }
                },
                "narration" => {
                    narration_count += 1;
                    marked_count += 1;
                },
                "environment" => {
                    environment_count += 1;
                    marked_count += 1;
                },
                _ => {}
            }

            result_paragraphs.push(json!({
                "content": content,
                "type": p_type,
                "character_id": character_id,
                "character_name": character_name
            }));
        }

        // 如果需要保存到数据库
        if save_to_db && chapter_id.is_some() {
            let cid = chapter_id.unwrap();
            
            // 准备保存的数据
            let mut paragraphs_request = Vec::new();
            
            for para in para_list {
                let content = para.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let p_type = para.get("type").and_then(|v| v.as_str()).unwrap_or("narration");
                let character_id = para.get("character_id").and_then(|v| v.as_i64());
                let paragraph_index = para.get("index").and_then(|v| v.as_i64()).unwrap_or(0);
                
                paragraphs_request.push(ParagraphMarkRequest {
                    paragraph_index: paragraph_index as i32,
                    content: content.to_string(),
                    r#type: p_type.to_string(),
                    character_id
                });
            }
            
            // 保存到数据库
            match db.save_chapter_paragraphs(cid, paragraphs_request) {
                Ok(_) => (),
                Err(e) => {
                    return Ok(json!({
                        "success": false,
                        "message": format!("保存标注失败: {}", e)
                    }));
                }
            }
        }

        Ok(json!({
            "success": true,
            "message": format!("标注完成：共 {} 段，对话 {} 段，旁白 {} 段，环境音 {} 段", 
                result_paragraphs.len(), dialogue_count, narration_count, environment_count),
            "total_count": result_paragraphs.len(),
            "marked_count": marked_count,
            "dialogue_count": dialogue_count,
            "narration_count": narration_count,
            "environment_count": environment_count,
            "paragraphs": result_paragraphs
        }))
    }
}

/// 手动导入内容并设置为单一段落工具
pub struct ImportContentAsSingleParagraphTool;

impl ImportContentAsSingleParagraphTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for ImportContentAsSingleParagraphTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "import_content_as_single_paragraph".to_string(),
            description: "手动导入章节内容并作为一个单一段落处理，适用于后续手动细分操作".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节ID，如果提供则从数据库读取内容"
                    },
                    "content": {
                        "type": "string",
                        "description": "要导入的内容，如果提供则优先使用此内容"
                    },
                    "paragraph_type": {
                        "type": "string",
                        "description": "默认段落类型：narration（旁白）、dialogue（对话）、environment（环境音）",
                        "enum": ["narration", "dialogue", "environment"],
                        "default": "narration"
                    }
                },
                "required": []
            }),
            return_description: Some("返回导入的单一段落".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params.get("chapter_id").and_then(|v| v.as_i64());
        let content = params.get("content").and_then(|v| v.as_str());
        let paragraph_type = params.get("paragraph_type").and_then(|v| v.as_str()).unwrap_or("narration");

        // 获取要处理的内容
        let mut text = String::new();
        
        if let Some(content_str) = content {
            text = content_str.to_string();
        } else if let Some(cid) = chapter_id {
            let conn = db.get_conn();
            let result: Result<String, _> = conn.query_row(
                "SELECT plain_text FROM chapters WHERE id = ?1",
                [cid],
                |row| row.get(0)
            );
            if let Ok(plain_text) = result {
                text = plain_text;
            } else {
                return Ok(json!({
                    "success": false,
                    "message": "未找到章节内容"
                }));
            }
        }

        if text.is_empty() {
            return Ok(json!({
                "success": false,
                "message": "没有可导入的内容"
            }));
        }

        // 简单清理HTML标签
        let processed_text = clean_html_tags(&text)
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n\n", "\n")
            .trim().to_string();

        Ok(json!({
            "success": true,
            "message": "内容已导入，当前 1 个段落。请手动编辑调整。",
            "total_count": 1,
            "paragraphs": [
                {
                    "index": 0,
                    "content": processed_text,
                    "type": paragraph_type,
                    "character_id": serde_json::Value::Null,
                    "character_name": serde_json::Value::Null
                }
            ]
        }))
    }
}

/// 获取章节段落标注工具
pub struct GetChapterParagraphsTool;

impl GetChapterParagraphsTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetChapterParagraphsTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_chapter_paragraphs".to_string(),
            description: "获取章节的所有段落标注信息".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节ID"
                    }
                },
                "required": ["chapter_id"]
            }),
            return_description: Some("返回章节的段落标注列表".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let chapter_id = params.get("chapter_id").and_then(|v| v.as_i64());
        
        let Some(cid) = chapter_id else {
            return Ok(json!({
                "success": false,
                "message": "缺少 chapter_id 参数"
            }));
        };
        
        // 从数据库获取段落标注
        match db.get_chapter_paragraphs(cid) {
            Ok(chapter_paragraphs) => {
                // 转换为前端需要的格式
                let mut paragraphs_response = Vec::new();
                
                for para in chapter_paragraphs {
                    paragraphs_response.push(json!({
                        "index": para.paragraph_index,
                        "content": para.content,
                        "type": para.r#type,
                        "character_id": para.character_id,
                        "character_name": serde_json::Value::Null
                    }));
                }
                
                Ok(json!({
                    "success": true,
                    "total_count": paragraphs_response.len(),
                    "paragraphs": paragraphs_response
                }))
            },
            Err(e) => {
                Ok(json!({
                    "success": false,
                    "message": format!("获取段落标注失败: {}", e)
                }))
            }
        }
    }
}

/// 获取角色对话的音色配置工具
pub struct GetParagraphVoiceConfigTool;

impl GetParagraphVoiceConfigTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for GetParagraphVoiceConfigTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "get_paragraph_voice_config".to_string(),
            description: "获取段落对应的音色配置，用于生成音频".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "novel_id": {
                        "type": "integer",
                        "description": "小说ID"
                    },
                    "paragraphs": {
                        "type": "array",
                        "description": "段落列表（需包含type和character_id）",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "description": "段落类型"
                                },
                                "character_id": {
                                    "type": "integer",
                                    "description": "角色ID（对话类型需要）"
                                }
                            }
                        }
                    }
                },
                "required": ["novel_id", "paragraphs"]
            }),
            return_description: Some("返回每个段落的音色配置".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let novel_id = params
            .get("novel_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 novel_id 参数".to_string()))?;

        let paragraphs = params
            .get("paragraphs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少或无效的 paragraphs 参数".to_string()))?;

        // 获取所有角色的音色配置
        let conn = db.get_conn();
        let mut stmt = conn.prepare("SELECT id, name, voice_id FROM characters WHERE novel_id = ?1")?;
        let character_voices: std::collections::HashMap<i64, (String, Option<String>)> = stmt
            .query_map([novel_id], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let voice_id: Option<String> = row.get(2)?;
                Ok((id, (name, voice_id)))
            })?
            .collect::<Result<_, _>>()?;

        // 默认旁白音色
        let default_narration_voice = "female-yujie";
        let default_environment_voice = "male-qn-qingse";

        let mut result = Vec::new();
        for (index, para) in paragraphs.iter().enumerate() {
            let p_type = para.get("type").and_then(|v| v.as_str()).unwrap_or("narration");
            let character_id = para.get("character_id").and_then(|v| v.as_i64());

            let (voice_id, character_name, voice_type) = match p_type {
                "dialogue" => {
                    if let Some(cid) = character_id {
                        if let Some((name, voice)) = character_voices.get(&cid) {
                            (voice.as_deref().unwrap_or(default_narration_voice), 
                             Some(name.as_str()), "角色对话")
                        } else {
                            (default_narration_voice, None, "未知角色")
                        }
                    } else {
                        (default_narration_voice, None, "对话（未指定角色）")
                    }
                },
                "environment" => (default_environment_voice, None, "环境音"),
                _ => (default_narration_voice, None, "旁白"),
            };

            result.push(json!({
                "paragraph_index": index,
                "type": p_type,
                "character_id": character_id,
                "character_name": character_name,
                "voice_id": voice_id,
                "voice_type": voice_type
            }));
        }

        Ok(json!({
            "success": true,
            "voice_configs": result,
            "character_count": character_voices.len(),
            "default_voices": {
                "narration": default_narration_voice,
                "environment": default_environment_voice
            }
        }))
    }
}
