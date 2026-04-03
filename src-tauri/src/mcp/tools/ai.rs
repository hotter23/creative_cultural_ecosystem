//! AI 功能 MCP 工具

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::clients::minimax::{MinMaxClient, MinMaxConfig};
use crate::db::Database;
use super::super::{error::McpResult, protocol::ToolDescription};
use super::McpTool;

/// 加载 MinMax 客户端（内部使用）
fn load_minimax_client(db: &Arc<Database>) -> Result<MinMaxClient, String> {
    let config_map = db.get_config_by_category("minimax")
        .map_err(|e| e.to_string())?;
    
    let enabled = config_map.get("minimax_enabled")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    
    if !enabled {
        return Err("MinMax 服务未启用，请先在设置中配置并启用 MinMax".to_string());
    }
    
    let api_key = config_map.get("minimax_api_key")
        .cloned()
        .unwrap_or_default();
    
    if api_key.is_empty() {
        return Err("MinMax API Key 未配置，请先在设置中配置 API Key".to_string());
    }
    
    let config = MinMaxConfig {
        api_key,
        base_url: config_map.get("minimax_base_url")
            .cloned()
            .unwrap_or_else(|| "https://api.minimaxi.com".to_string()),
        default_model: config_map.get("minimax_default_model")
            .cloned()
            .unwrap_or_else(|| "abab6.5s-chat".to_string()),
        group_id: config_map.get("minimax_group_id").cloned(),
        enabled: true,
    };
    
    Ok(MinMaxClient::new(config))
}

/// AI 生成网文内容工具
pub struct AiGenerateNovelContentTool;

impl AiGenerateNovelContentTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for AiGenerateNovelContentTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "ai_generate_novel_content".to_string(),
            description: "使用 AI 根据主题或开篇自动生成网文内容。支持指定题材风格和字数要求。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "内容生成的提示词，可以是主题、开篇、或创意描述"
                    },
                    "genre": {
                        "type": "string",
                        "description": "小说题材类型，如：玄幻、都市、科幻、武侠、言情、历史等",
                        "default": "通用"
                    },
                    "word_count": {
                        "type": "integer",
                        "description": "期望生成的字数，建议范围：100-3000",
                        "minimum": 100,
                        "maximum": 5000,
                        "default": 500
                    },
                    "novel_id": {
                        "type": "integer",
                        "description": "关联的小说ID（可选，用于上下文关联）"
                    }
                },
                "required": ["prompt"]
            }),
            return_description: Some("返回 AI 生成的小说内容文本".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let prompt = params
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 prompt 参数".to_string()))?;
        
        let genre = params.get("genre").and_then(|v| v.as_str()).unwrap_or("通用");
        let word_count = params.get("word_count").and_then(|v| v.as_u64()).unwrap_or(500) as u32;
        
        let client = load_minimax_client(db)
            .map_err(super::super::error::McpError::InternalError)?;
        
        let result = client.generate_novel_content(
            prompt,
            genre,
            word_count,
            None,
        ).await
        .map_err(super::super::error::McpError::InternalError)?;
        
        Ok(json!({
            "content": result,
            "genre": genre,
            "word_count": result.chars().count()
        }))
    }
}

/// AI 续写网文内容工具
pub struct AiContinueNovelContentTool;

impl AiContinueNovelContentTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for AiContinueNovelContentTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "ai_continue_novel_content".to_string(),
            description: "基于已有内容，使用 AI 续写网文。AI 会保持风格连贯，推动情节发展。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prefix_content": {
                        "type": "string",
                        "description": "已有内容的前缀，用于上下文参考（建议最后1000-3000字）"
                    },
                    "genre": {
                        "type": "string",
                        "description": "小说题材类型，如：玄幻、都市、科幻等",
                        "default": "通用"
                    },
                    "word_count": {
                        "type": "integer",
                        "description": "期望续写的字数",
                        "minimum": 100,
                        "maximum": 5000,
                        "default": 500
                    },
                    "chapter_id": {
                        "type": "integer",
                        "description": "当前章节ID（可选）"
                    }
                },
                "required": ["prefix_content"]
            }),
            return_description: Some("返回 AI 续写的内容文本".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let prefix_content = params
            .get("prefix_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 prefix_content 参数".to_string()))?;
        
        let genre = params.get("genre").and_then(|v| v.as_str()).unwrap_or("通用");
        let word_count = params.get("word_count").and_then(|v| v.as_u64()).unwrap_or(500) as u32;
        
        let client = load_minimax_client(db)
            .map_err(super::super::error::McpError::InternalError)?;
        
        let result = client.continue_novel_content(
            prefix_content,
            genre,
            word_count,
        ).await
        .map_err(super::super::error::McpError::InternalError)?;
        
        Ok(json!({
            "content": result,
            "genre": genre,
            "word_count": result.chars().count()
        }))
    }
}

/// AI 润色内容工具
pub struct AiPolishContentTool;

impl AiPolishContentTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for AiPolishContentTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "ai_polish_content".to_string(),
            description: "使用 AI 润色优化网文内容，提升文笔质量，保持原意不变。支持多种润色风格。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "需要润色的原文内容"
                    },
                    "style": {
                        "type": "string",
                        "description": "润色风格：润色优化(默认)、更有张力、细腻描写、简洁明快、幽默风趣、古风典雅、现代都市",
                        "default": "润色优化，让文字更有张力"
                    },
                    "chapter_id": {
                        "type": "integer",
                        "description": "章节ID（可选）"
                    }
                },
                "required": ["content"]
            }),
            return_description: Some("返回润色后的内容文本".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 content 参数".to_string()))?;
        
        let style = params.get("style").and_then(|v| v.as_str()).unwrap_or("润色优化，让文字更有张力");
        
        let client = load_minimax_client(db)
            .map_err(super::super::error::McpError::InternalError)?;
        
        let result = client.polish_content(content, style).await
            .map_err(super::super::error::McpError::InternalError)?;
        
        Ok(json!({
            "original_content": content,
            "polished_content": result,
            "style": style,
            "change_count": 1
        }))
    }
}

/// AI 总结内容工具
pub struct AiSummarizeContentTool;

impl AiSummarizeContentTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for AiSummarizeContentTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "ai_summarize_content".to_string(),
            description: "使用 AI 对长文本进行智能摘要，提取核心情节和关键信息。可用于章节梗概、剧情回顾等。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "需要总结的内容文本（支持长文本）"
                    },
                    "max_length": {
                        "type": "integer",
                        "description": "摘要的最大长度（字数）",
                        "minimum": 50,
                        "maximum": 1000,
                        "default": 200
                    },
                    "summary_type": {
                        "type": "string",
                        "description": "摘要类型：general(通用摘要), plot(情节摘要), character(角色摘要), conflict(冲突点)",
                        "default": "general"
                    }
                },
                "required": ["content"]
            }),
            return_description: Some("返回生成的摘要内容".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 content 参数".to_string()))?;
        
        let max_length = params.get("max_length").and_then(|v| v.as_u64()).unwrap_or(200) as u32;
        
        let client = load_minimax_client(db)
            .map_err(super::super::error::McpError::InternalError)?;
        
        let result = client.summarize_content(content, max_length).await
            .map_err(super::super::error::McpError::InternalError)?;
        
        Ok(json!({
            "summary": result,
            "original_length": content.chars().count(),
            "summary_length": result.chars().count(),
            "max_length": max_length
        }))
    }
}

/// AI 情节建议工具
pub struct AiSuggestPlotTool;

impl AiSuggestPlotTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for AiSuggestPlotTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "ai_suggest_plot".to_string(),
            description: "基于当前内容，使用 AI 提供后续情节发展建议，帮助突破创作瓶颈。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "当前小说内容，用于上下文理解"
                    },
                    "genre": {
                        "type": "string",
                        "description": "小说题材类型",
                        "default": "通用"
                    },
                    "suggestion_count": {
                        "type": "integer",
                        "description": "建议的数量（1-5）",
                        "minimum": 1,
                        "maximum": 5,
                        "default": 3
                    }
                },
                "required": ["content"]
            }),
            return_description: Some("返回情节建议列表".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 content 参数".to_string()))?;
        
        let client = load_minimax_client(db)
            .map_err(super::super::error::McpError::InternalError)?;
        
        let result = client.generate_plot_suggestions(content).await
            .map_err(super::super::error::McpError::InternalError)?;
        
        // 将结果按换行分割成列表
        let suggestions: Vec<&str> = result
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();
        
        Ok(json!({
            "suggestions": suggestions,
            "raw_response": result
        }))
    }
}

/// AI 提取角色工具
pub struct AiExtractCharactersTool;

impl AiExtractCharactersTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl McpTool for AiExtractCharactersTool {
    fn description(&self) -> ToolDescription {
        ToolDescription {
            name: "ai_extract_characters".to_string(),
            description: "从网文内容中智能提取角色信息，包括角色名称、性别、身份定位、性格特点等。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "包含角色描写的小说内容"
                    },
                    "novel_id": {
                        "type": "integer",
                        "description": "关联的小说ID（可选）"
                    },
                    "auto_save": {
                        "type": "boolean",
                        "description": "是否自动保存提取的角色到数据库",
                        "default": false
                    }
                },
                "required": ["content"]
            }),
            return_description: Some("返回提取到的角色列表".to_string()),
        }
    }

    async fn call(&self, db: &Arc<Database>, params: &Value) -> McpResult<Value> {
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::super::error::McpError::InvalidParameter("缺少 content 参数".to_string()))?;
        
        let client = load_minimax_client(db)
            .map_err(super::super::error::McpError::InternalError)?;
        
        let result = client.extract_characters(content).await
            .map_err(super::super::error::McpError::InternalError)?;
        
        Ok(json!({
            "characters": result,
            "count": result.len()
        }))
    }
}
