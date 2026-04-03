//! MCP 协议定义模块
//! 定义 MCP 的请求、响应、工具描述等核心数据结构

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP 请求消息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpRequest {
    /// 请求 ID，用于追踪
    pub id: String,
    /// 操作类型: list_tools, call_tool
    pub method: String,
    /// 参数
    pub params: Value,
}

/// MCP 响应消息
#[derive(Debug, Serialize, Clone)]
pub struct McpResponse {
    /// 响应 ID，与请求 ID 对应
    pub id: String,
    /// 是否成功
    pub success: bool,
    /// 结果数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpErrorInfo>,
}

/// 错误信息
#[derive(Debug, Serialize, Clone)]
pub struct McpErrorInfo {
    /// 错误码
    pub code: i32,
    /// 错误消息
    pub message: String,
}

/// 工具描述
#[derive(Debug, Serialize, Clone)]
pub struct ToolDescription {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数定义 (JSON Schema)
    pub parameters: Value,
    /// 返回值描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_description: Option<String>,
}

/// 工具列表响应
#[derive(Debug, Serialize, Clone)]
pub struct ToolListResponse {
    /// 工具列表
    pub tools: Vec<ToolDescription>,
}

/// 工具调用请求参数
#[derive(Debug, Deserialize, Clone)]
pub struct ToolCallParams {
    /// 工具名称
    pub name: String,
    /// 工具参数
    pub arguments: Value,
}

/// 工具调用响应
#[derive(Debug, Serialize, Clone)]
pub struct ToolCallResponse {
    /// 调用结果
    pub content: Value,
    /// 内容类型: text/plain, application/json 等
    pub content_type: String,
}

/// 创建成功响应
pub fn create_success_response(id: String, result: Value) -> McpResponse {
    McpResponse {
        id,
        success: true,
        result: Some(result),
        error: None,
    }
}

/// 创建错误响应
pub fn create_error_response(id: String, code: i32, message: String) -> McpResponse {
    McpResponse {
        id,
        success: false,
        result: None,
        error: Some(McpErrorInfo { code, message }),
    }
}

/// 解析工具调用参数
pub fn parse_tool_call_params(params: &Value) -> Result<ToolCallParams, String> {
    serde_json::from_value::<ToolCallParams>(params.clone())
        .map_err(|e| format!("参数解析失败: {}", e))
}
