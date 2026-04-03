//! MCP 工具模块
//! 定义所有可通过 MCP 协议调用的工具

pub mod novel;
pub mod chapter;
pub mod ai;
pub mod audio;
pub mod character;
pub mod chapter_audio;
pub mod ambient;

use std::collections::HashMap;
use std::sync::Arc;

use crate::db::Database;
use super::error::McpResult;
use super::protocol::{ToolDescription, ToolCallParams};

/// 工具 trait
#[async_trait::async_trait]
pub trait McpTool: Send + Sync {
    /// 获取工具描述
    fn description(&self) -> ToolDescription;
    
    /// 调用工具
    async fn call(&self, db: &Arc<Database>, params: &serde_json::Value) -> McpResult<serde_json::Value>;
}

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn McpTool>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        registry.register_default_tools();
        registry
    }

    /// 注册工具
    pub fn register<T: McpTool + 'static>(&mut self, tool: T) {
        let desc = tool.description();
        self.tools.insert(desc.name, Box::new(tool));
    }

    /// 获取所有工具描述
    pub fn list_tools(&self) -> Vec<ToolDescription> {
        self.tools.values()
            .map(|tool| tool.description())
            .collect()
    }

    /// 调用工具
    pub async fn call_tool(
        &self,
        db: &Arc<Database>,
        params: &ToolCallParams,
    ) -> McpResult<serde_json::Value> {
        let tool = self.tools.get(&params.name)
            .ok_or_else(|| super::error::McpError::ToolNotFound(params.name.clone()))?;
        
        tool.call(db, &params.arguments).await
    }

    /// 注册默认工具
    fn register_default_tools(&mut self) {
        // 网文相关工具
        self.register(novel::ListNovelsTool::new());
        self.register(novel::GetNovelTool::new());
        self.register(novel::CreateNovelTool::new());
        self.register(novel::UpdateNovelTool::new());
        self.register(novel::DeleteNovelTool::new());

        // 章节相关工具
        self.register(chapter::ListChaptersTool::new());
        self.register(chapter::GetChapterTool::new());
        self.register(chapter::CreateChapterTool::new());
        self.register(chapter::UpdateChapterTool::new());
        self.register(chapter::DeleteChapterTool::new());
        self.register(chapter::ImportContentAsSingleParagraphTool::new());
        self.register(chapter::BatchMarkParagraphsTool::new());
        self.register(chapter::GetChapterParagraphsTool::new());
        self.register(chapter::GetParagraphVoiceConfigTool::new());

        // AI 相关工具
        self.register(ai::AiGenerateNovelContentTool::new());
        self.register(ai::AiContinueNovelContentTool::new());
        self.register(ai::AiPolishContentTool::new());
        self.register(ai::AiSummarizeContentTool::new());
        self.register(ai::AiSuggestPlotTool::new());
        self.register(ai::AiExtractCharactersTool::new());

        // 音频相关工具
        self.register(audio::TextToSpeechTool::new());
        
        // 章节音频管理工具
        self.register(chapter_audio::GetChapterAudioDetailTool::new());
        self.register(chapter_audio::GetAudioParagraphsTool::new());
        self.register(chapter_audio::GenerateChapterAudioTool::new());

        // 角色相关工具
        self.register(character::ListCharactersTool::new());
        self.register(character::GetCharacterTool::new());
        self.register(character::CreateCharacterTool::new());
        self.register(character::UpdateCharacterTool::new());
        self.register(character::DeleteCharacterTool::new());
        self.register(character::ListCharacterImagesTool::new());
        self.register(character::BindCharacterVoiceTool::new());
        self.register(character::GetVoiceListTool::new());

        // 环境音相关工具
        self.register(ambient::GenerateAmbientSoundTool::new());
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
