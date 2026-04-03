//! MCP 服务 Tauri 命令接口
//! 提供前端调用 MCP 服务的 Tauri 命令

use crate::{log_error};
use std::sync::Arc;
use std::net::SocketAddr;
use tauri::{command, State};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::db::Database;
use crate::mcp::{
    McpServer, McpRequest, protocol,
    McpHttpServer, McpServiceConfig,
    register_mcp_service, unregister_mcp_service, get_mcp_config_dir,
};

/// MCP HTTP 服务器状态
#[derive(Default)]
pub struct McpHttpServerState {
    /// HTTP 服务器实例
    pub server: Mutex<Option<Arc<McpHttpServer>>>,
    /// 服务器任务句柄
    pub server_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

/// MCP 服务完整状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct McpServiceStatus {
    /// 核心服务状态: stopped, running, paused
    pub status: String,
    /// HTTP 服务器状态: stopped, running
    pub http_status: String,
    /// 已注册的工具数量
    pub tool_count: usize,
    /// 服务版本
    pub version: String,
    /// 服务是否已启用
    pub enabled: bool,
    /// HTTP 服务地址
    pub http_endpoint: Option<String>,
    /// SSE 端点地址
    pub sse_endpoint: Option<String>,
    /// 服务端口
    pub port: Option<u16>,
    /// 绑定地址
    pub bind_address: Option<String>,
}

/// 获取 MCP 服务状态
#[command]
pub async fn mcp_get_status(
    db: State<'_, Arc<Database>>,
    mcp_server: State<'_, Arc<McpServer>>,
    http_server_state: State<'_, Arc<McpHttpServerState>>,
) -> Result<McpServiceStatus, String> {
    let status = mcp_server.status().await;
    let tool_count = mcp_server.tool_registry().list_tools().len();
    
    // 检查 MCP 是否在配置中启用
    let enabled = match db.get_config("mcp_enabled") {
        Ok(Some(value)) => value == "true",
        _ => true, // 默认启用
    };

    // 从配置中读取端口
    let configured_port = match db.get_config("mcp_port") {
        Ok(Some(value)) => value.parse::<u16>().unwrap_or(8787),
        _ => 8787,
    };
    
    // 从配置中读取绑定地址
    let bind_address = match db.get_config("mcp_bind_address") {
        Ok(Some(value)) => Some(value),
        _ => Some("127.0.0.1".to_string()),
    };
    
    // 检查 HTTP 服务器状态
    let http_server = http_server_state.server.lock().await;
    let (http_status, http_endpoint, sse_endpoint, port) = if let Some(server) = http_server.as_ref() {
        if server.is_running() {
            let config = &server.state.config;
            (
                "running".to_string(),
                Some(config.http_endpoint.clone()),
                Some(config.sse_endpoint.clone()),
                Some(configured_port),
            )
        } else {
            ("stopped".to_string(), None, None, Some(configured_port))
        }
    } else {
        ("stopped".to_string(), None, None, Some(configured_port))
    };
    
    Ok(McpServiceStatus {
        status: format!("{:?}", status).to_lowercase(),
        http_status,
        tool_count,
        version: env!("CARGO_PKG_VERSION").to_string(),
        enabled,
        http_endpoint,
        sse_endpoint,
        port,
        bind_address,
    })
}

/// 启动 MCP 核心服务
#[command]
pub async fn mcp_start(
    mcp_server: State<'_, Arc<McpServer>>,
) -> Result<bool, String> {
    mcp_server.start().await
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

/// 停止 MCP 核心服务
#[command]
pub async fn mcp_stop(
    mcp_server: State<'_, Arc<McpServer>>,
) -> Result<bool, String> {
    mcp_server.stop().await
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

/// 暂停 MCP 核心服务
#[command]
pub async fn mcp_pause(
    mcp_server: State<'_, Arc<McpServer>>,
) -> Result<bool, String> {
    mcp_server.pause().await
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

/// 恢复 MCP 核心服务
#[command]
pub async fn mcp_resume(
    mcp_server: State<'_, Arc<McpServer>>,
) -> Result<bool, String> {
    mcp_server.resume().await
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

/// 启动 MCP HTTP 服务器
#[command]
pub async fn mcp_http_server_start(
    db: State<'_, Arc<Database>>,
    mcp_server: State<'_, Arc<McpServer>>,
    http_server_state: State<'_, Arc<McpHttpServerState>>,
    port: Option<u16>,
    bind_address: Option<String>,
) -> Result<bool, String> {
    let mut server_lock = http_server_state.server.lock().await;
    
    if let Some(existing_server) = server_lock.as_ref() {
        if existing_server.is_running() {
            return Err("MCP HTTP 服务器已经在运行".to_string());
        }
    }

    // 创建服务配置
    let port = port.unwrap_or(8787);
    let bind_addr = bind_address.unwrap_or_else(|| "127.0.0.1".to_string());
    
    // 解析绑定地址
    let ip = match bind_addr.as_str() {
        "0.0.0.0" => std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        "127.0.0.1" => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        addr => addr.parse().map_err(|_| format!("无效的IP地址: {}", addr))?,
    };
    
    let config = McpServiceConfig {
        http_endpoint: format!("http://{}:{}/mcp", bind_addr, port),
        sse_endpoint: format!("http://{}:{}/mcp/sse", bind_addr, port),
        ..McpServiceConfig::default()
    };

    // 创建 HTTP 服务器
    let http_server = Arc::new(McpHttpServer::new(
        mcp_server.inner().clone(),
        db.inner().clone(),
        Some(config),
    ));

    let server_clone = http_server.clone();
    let addr = SocketAddr::from((ip, port));
    
    // 启动服务器任务
    let server_task = tokio::spawn(async move {
        if let Err(e) = server_clone.start(addr).await {
            log_error!("MCP HTTP 服务器错误: {}", e);
        }
    });

    // 等待服务器启动
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    *server_lock = Some(http_server);
    
    let mut task_lock = http_server_state.server_task.lock().await;
    *task_lock = Some(server_task);

    Ok(true)
}

/// 停止 MCP HTTP 服务器
#[command]
pub async fn mcp_http_server_stop(
    http_server_state: State<'_, Arc<McpHttpServerState>>,
) -> Result<bool, String> {
    let mut server_lock = http_server_state.server.lock().await;
    
    if let Some(server) = server_lock.as_ref() {
        server.stop().await
            .map_err(|e| e.to_string())?;
    } else {
        return Err("MCP HTTP 服务器未运行".to_string());
    }

    // 取消任务
    let mut task_lock = http_server_state.server_task.lock().await;
    if let Some(task) = task_lock.take() {
        task.abort();
    }

    *server_lock = None;

    Ok(true)
}

/// 注册 MCP 服务到本地配置目录（供 Agent 发现）
#[command]
pub async fn mcp_register_service(
    port: Option<u16>,
    bind_address: Option<String>,
) -> Result<bool, String> {
    let port = port.unwrap_or(8787);
    let bind_addr = bind_address.unwrap_or_else(|| "127.0.0.1".to_string());
    let config = McpServiceConfig {
        http_endpoint: format!("http://{}:{}/mcp", bind_addr, port),
        sse_endpoint: format!("http://{}:{}/mcp/sse", bind_addr, port),
        ..McpServiceConfig::default()
    };

    register_mcp_service(&config)?;

    Ok(true)
}

/// 注销 MCP 服务
#[command]
pub async fn mcp_unregister_service() -> Result<bool, String> {
    unregister_mcp_service("ai-content-creator")?;
    Ok(true)
}

/// 获取 MCP 配置目录路径
#[command]
pub async fn mcp_get_config_dir() -> Result<String, String> {
    get_mcp_config_dir()
}

/// 获取所有可用工具列表
#[command]
pub async fn mcp_list_tools(
    mcp_server: State<'_, Arc<McpServer>>,
) -> Result<Value, String> {
    let tools = mcp_server.tool_registry().list_tools();
    
    serde_json::to_value(tools)
        .map_err(|e| format!("序列化工具列表失败: {}", e))
}

/// 调用 MCP 工具
#[command]
pub async fn mcp_call_tool(
    db: State<'_, Arc<Database>>,
    mcp_server: State<'_, Arc<McpServer>>,
    tool_name: String,
    arguments: Value,
) -> Result<Value, String> {
    let params = protocol::ToolCallParams {
        name: tool_name,
        arguments,
    };
    
    let result = mcp_server.tool_registry().call_tool(&db, &params).await
        .map_err(|e| e.to_string())?;
    
    Ok(result)
}

/// 处理 MCP 请求（JSON RPC 风格）
#[command]
pub async fn mcp_handle_request(
    db: State<'_, Arc<Database>>,
    mcp_server: State<'_, Arc<McpServer>>,
    request: Value,
) -> Result<Value, String> {
    let mcp_request: McpRequest = serde_json::from_value(request)
        .map_err(|e| format!("请求解析失败: {}", e))?;
    
    let response = mcp_server.handle_request(&db, mcp_request).await;
    
    serde_json::to_value(response)
        .map_err(|e| format!("响应序列化失败: {}", e))
}

/// 处理 MCP 请求（字符串版本）
#[command]
pub async fn mcp_handle_request_str(
    db: State<'_, Arc<Database>>,
    mcp_server: State<'_, Arc<McpServer>>,
    request_str: String,
) -> Result<String, String> {
    mcp_server.handle_request_str(&db, &request_str).await
}

/// 获取 MCP 配置
#[command]
pub async fn mcp_get_config(
    db: State<'_, Arc<Database>>,
) -> Result<Value, String> {
    let config = db.get_config_by_category("mcp")
        .map_err(|e| e.to_string())?;
    
    serde_json::to_value(config)
        .map_err(|e| format!("序列化配置失败: {}", e))
}

/// 保存 MCP 配置
#[command]
pub async fn mcp_save_config(
    db: State<'_, Arc<Database>>,
    enabled: bool,
    auto_start: bool,
    port: i32,
    bind_address: String,
) -> Result<bool, String> {
    let enabled_str = enabled.to_string();
    let auto_start_str = auto_start.to_string();
    let port_str = port.to_string();
    
    let configs = [
        ("mcp_enabled", enabled_str.as_str(), "mcp"),
        ("mcp_auto_start", auto_start_str.as_str(), "mcp"),
        ("mcp_port", port_str.as_str(), "mcp"),
        ("mcp_bind_address", bind_address.as_str(), "mcp"),
    ];
    
    db.set_config_batch(&configs)
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}
