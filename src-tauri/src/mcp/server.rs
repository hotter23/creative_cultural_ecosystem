//! MCP 服务器核心模块
//! 处理 MCP 请求的路由和执行逻辑

use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::db::Database;
use super::error::{McpError, McpResult};
use super::protocol::*;
use super::tools::ToolRegistry;

/// MCP 服务器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    /// 已停止
    Stopped,
    /// 运行中
    Running,
    /// 暂停中
    Paused,
}

/// MCP 服务器核心结构
pub struct McpServer {
    /// 工具注册表
    tool_registry: ToolRegistry,
    /// 服务器状态
    status: Mutex<ServerStatus>,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    /// 创建新的 MCP 服务器
    pub fn new() -> Self {
        Self {
            tool_registry: ToolRegistry::new(),
            status: Mutex::new(ServerStatus::Stopped),
        }
    }

    /// 获取服务器状态
    pub async fn status(&self) -> ServerStatus {
        *self.status.lock().await
    }

    /// 启动服务器
    pub async fn start(&self) -> McpResult<()> {
        let mut status = self.status.lock().await;
        *status = ServerStatus::Running;
        Ok(())
    }

    /// 停止服务器
    pub async fn stop(&self) -> McpResult<()> {
        let mut status = self.status.lock().await;
        *status = ServerStatus::Stopped;
        Ok(())
    }

    /// 暂停服务器
    pub async fn pause(&self) -> McpResult<()> {
        let mut status = self.status.lock().await;
        *status = ServerStatus::Paused;
        Ok(())
    }

    /// 恢复服务器
    pub async fn resume(&self) -> McpResult<()> {
        let mut status = self.status.lock().await;
        *status = ServerStatus::Running;
        Ok(())
    }

    /// 处理 MCP 请求
    pub async fn handle_request(&self, db: &Arc<Database>, request: McpRequest) -> McpResponse {
        let status = self.status.lock().await;
        if *status != ServerStatus::Running {
            return create_error_response(
                request.id,
                503,
                "MCP 服务未运行".to_string(),
            );
        }
        drop(status);

        match request.method.as_str() {
            // 列出所有可用工具
            "list_tools" => self.handle_list_tools(request).await,
            // 调用工具
            "call_tool" => self.handle_call_tool(db, request).await,
            // 获取服务器状态
            "get_status" => self.handle_get_status(request).await,
            // 未知方法
            _ => create_error_response(
                request.id,
                404,
                format!("未知方法: {}", request.method),
            ),
        }
    }

    /// 处理 list_tools 请求
    async fn handle_list_tools(&self, request: McpRequest) -> McpResponse {
        let tools = self.tool_registry.list_tools();
        let response = ToolListResponse { tools };
        
        match serde_json::to_value(response) {
            Ok(result) => create_success_response(request.id, result),
            Err(e) => create_error_response(
                request.id,
                500,
                format!("序列化工具列表失败: {}", e),
            ),
        }
    }

    /// 处理 call_tool 请求
    async fn handle_call_tool(&self, db: &Arc<Database>, request: McpRequest) -> McpResponse {
        // 解析参数
        let tool_params = match parse_tool_call_params(&request.params) {
            Ok(params) => params,
            Err(e) => return create_error_response(request.id, 400, e),
        };

        // 调用工具
        match self.tool_registry.call_tool(db, &tool_params).await {
            Ok(result) => {
                let call_response = ToolCallResponse {
                    content: result,
                    content_type: "application/json".to_string(),
                };
                
                match serde_json::to_value(call_response) {
                    Ok(value) => create_success_response(request.id, value),
                    Err(e) => create_error_response(
                        request.id,
                        500,
                        format!("序列化响应失败: {}", e),
                    ),
                }
            },
            Err(e) => {
                let (code, message) = match e {
                    McpError::ToolNotFound(_) => (404, e.to_string()),
                    McpError::InvalidParameter(_) => (400, e.to_string()),
                    McpError::PermissionDenied(_) => (403, e.to_string()),
                    McpError::DatabaseError(_) => (500, format!("数据库错误: {}", e)),
                    McpError::InternalError(_) => (500, format!("内部错误: {}", e)),
                    McpError::SerdeError(_) => (400, format!("参数解析错误: {}", e)),
                };
                
                create_error_response(request.id, code, message)
            }
        }
    }

    /// 处理 get_status 请求
    async fn handle_get_status(&self, request: McpRequest) -> McpResponse {
        let status = self.status.lock().await;
        let status_str = match *status {
            ServerStatus::Stopped => "stopped",
            ServerStatus::Running => "running",
            ServerStatus::Paused => "paused",
        };
        
        create_success_response(
            request.id,
            json!({
                "status": status_str,
                "tool_count": self.tool_registry.list_tools().len(),
                "version": env!("CARGO_PKG_VERSION"),
            }),
        )
    }

    /// 便捷方法：JSON 字符串请求处理
    pub async fn handle_request_str(&self, db: &Arc<Database>, request_str: &str) -> Result<String, String> {
        // 解析请求
        let request: McpRequest = serde_json::from_str(request_str)
            .map_err(|e| format!("请求解析失败: {}", e))?;
        
        // 处理请求
        let response = self.handle_request(db, request).await;
        
        // 序列化响应
        serde_json::to_string(&response)
            .map_err(|e| format!("响应序列化失败: {}", e))
    }

    /// 获取工具注册表引用
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }
}
