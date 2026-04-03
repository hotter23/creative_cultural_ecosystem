//! MCP 错误处理模块

use std::fmt;

#[derive(Debug)]
pub enum McpError {
    /// 工具未找到
    ToolNotFound(String),
    /// 参数错误
    InvalidParameter(String),
    /// 内部错误
    InternalError(String),
    /// 数据库错误
    DatabaseError(String),
    /// 权限错误
    PermissionDenied(String),
    /// 序列化/反序列化错误
    SerdeError(String),
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::ToolNotFound(name) => write!(f, "工具未找到: {}", name),
            McpError::InvalidParameter(msg) => write!(f, "参数错误: {}", msg),
            McpError::InternalError(msg) => write!(f, "内部错误: {}", msg),
            McpError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            McpError::PermissionDenied(msg) => write!(f, "权限错误: {}", msg),
            McpError::SerdeError(msg) => write!(f, "序列化错误: {}", msg),
        }
    }
}

impl std::error::Error for McpError {}

pub type McpResult<T> = Result<T, McpError>;

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::SerdeError(err.to_string())
    }
}

impl From<crate::db::DbError> for McpError {
    fn from(err: crate::db::DbError) -> Self {
        McpError::DatabaseError(err.to_string())
    }
}

impl From<rusqlite::Error> for McpError {
    fn from(err: rusqlite::Error) -> Self {
        McpError::DatabaseError(err.to_string())
    }
}
