//! MCP (Model Control Protocol) 服务端模块
//! 提供标准化的工具调用接口，供 Agent 进行自然语言交互

pub mod error;
pub mod protocol;
pub mod server;
pub mod tools;
pub mod discovery;

pub use protocol::*;
pub use server::McpServer;
pub use discovery::{
    McpHttpServer, 
    McpServiceConfig, 
    register_mcp_service,
    unregister_mcp_service,
    get_mcp_config_dir,
};
