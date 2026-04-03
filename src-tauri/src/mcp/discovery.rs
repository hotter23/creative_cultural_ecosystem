//! MCP 服务发现模块
//! 实现标准的 MCP 服务发现协议，包括 HTTP 端点和 SSE 事件流

use crate::log;
use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse, IntoResponse, Json,
    },
    routing::{get, post},
    Router,
    http::StatusCode,
    serve,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::db::Database;
use super::{McpServer, McpRequest, McpResponse};
use super::protocol::ToolDescription;

/// MCP 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServiceConfig {
    /// 服务名称
    pub name: String,
    /// 服务版本
    pub version: String,
    /// 服务描述
    pub description: String,
    /// HTTP 服务地址
    pub http_endpoint: String,
    /// SSE 端点
    pub sse_endpoint: String,
    /// 支持的能力
    pub capabilities: Vec<String>,
}

impl Default for McpServiceConfig {
    fn default() -> Self {
        Self {
            name: "AI Content Creator MCP Service".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "AI 内容创作平台 MCP 服务 - 支持网文创作、音频生成、角色管理等功能".to_string(),
            http_endpoint: "http://localhost:8787/mcp".to_string(),
            sse_endpoint: "http://localhost:8787/mcp/sse".to_string(),
            capabilities: vec![
                "novel_management".to_string(),
                "chapter_management".to_string(),
                "ai_content_generation".to_string(),
                "text_to_speech".to_string(),
            ],
        }
    }
}

/// MCP 发现响应
#[derive(Debug, Clone, Serialize)]
pub struct McpDiscoveryResponse {
    /// 协议版本
    pub protocol_version: String,
    /// 服务信息
    pub service: McpServiceConfig,
    /// 认证方式
    pub auth_type: String,
    /// 工具列表端点
    pub tools_endpoint: String,
    /// 调用端点
    pub invoke_endpoint: String,
}

/// MCP 服务器运行时状态
#[derive(Clone)]
pub struct McpServerState {
    /// MCP 核心服务
    pub mcp_server: Arc<McpServer>,
    /// 数据库连接
    pub db: Arc<Database>,
    /// 事件广播器
    pub event_sender: broadcast::Sender<McpServerEvent>,
    /// 服务配置
    pub config: McpServiceConfig,
    /// 服务器是否在运行
    pub running: Arc<std::sync::atomic::AtomicBool>,
}

/// MCP 服务器事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum McpServerEvent {
    /// 服务启动
    ServiceStarted { timestamp: i64 },
    /// 服务停止
    ServiceStopped { timestamp: i64 },
    /// 工具调用开始
    ToolCallStarted { id: String, tool: String, timestamp: i64 },
    /// 工具调用完成
    ToolCallCompleted { id: String, tool: String, success: bool, timestamp: i64 },
    /// 心跳
    Heartbeat { timestamp: i64 },
}

/// 调用参数
#[derive(Debug, Deserialize)]
struct InvokeParams {
    name: String,
    arguments: Value,
    #[serde(default)]
    id: Option<String>,
}

/// MCP HTTP 服务器
pub struct McpHttpServer {
    /// MCP 服务器状态
    pub state: McpServerState,
    shutdown_tx: broadcast::Sender<()>,
}

impl McpHttpServer {
    /// 创建新的 MCP HTTP 服务器
    pub fn new(mcp_server: Arc<McpServer>, db: Arc<Database>, config: Option<McpServiceConfig>) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        let (shutdown_tx, _) = broadcast::channel(1);
        
        Self {
            state: McpServerState {
                mcp_server,
                db,
                event_sender,
                config: config.unwrap_or_default(),
                running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            },
            shutdown_tx,
        }
    }

    /// 启动 HTTP 服务器
    pub async fn start(&self, addr: SocketAddr) -> Result<(), String> {
        if self.state.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("MCP HTTP 服务器已经在运行".to_string());
        }

        self.state.running.store(true, std::sync::atomic::Ordering::SeqCst);
        
        // 发送启动事件
        let _ = self.state.event_sender.send(McpServerEvent::ServiceStarted {
            timestamp: ::chrono::Utc::now().timestamp(),
        });

        // 启动心跳任务
        let event_sender = self.state.event_sender.clone();
        let running = self.state.running.clone();
        tokio::spawn(async move {
            while running.load(std::sync::atomic::Ordering::SeqCst) {
                let _ = event_sender.send(McpServerEvent::Heartbeat {
                    timestamp: ::chrono::Utc::now().timestamp(),
                });
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        let app = Self::create_router(self.state.clone());
        let shutdown_rx = self.shutdown_tx.subscribe();

        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| format!("绑定地址失败: {}", e))?;

        let server = serve(listener, app);

        let graceful = server.with_graceful_shutdown(async move {
            let mut rx = shutdown_rx;
            let _ = rx.recv().await;
        });

        if let Err(e) = graceful.await {
            return Err(format!("服务器错误: {}", e));
        }

        Ok(())
    }

    /// 停止 HTTP 服务器
    pub async fn stop(&self) -> Result<(), String> {
        if !self.state.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("MCP HTTP 服务器未运行".to_string());
        }

        self.state.running.store(false, std::sync::atomic::Ordering::SeqCst);
        
        // 发送停止事件
        let _ = self.state.event_sender.send(McpServerEvent::ServiceStopped {
            timestamp: ::chrono::Utc::now().timestamp(),
        });

        // 发送关闭信号
        let _ = self.shutdown_tx.send(());

        Ok(())
    }

    /// 检查服务器是否在运行
    pub fn is_running(&self) -> bool {
        self.state.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// 创建 Axum 路由
    fn create_router(state: McpServerState) -> Router {
        Router::new()
            // MCP 发现端点 - 标准的 .well-known 路径
            .route("/.well-known/mcp", get(Self::handle_discovery))
            .route("/.well-known/mcp.json", get(Self::handle_discovery))
            // MCP 服务根路径
            .route("/mcp", get(Self::handle_mcp_info))
            // 工具列表端点
            .route("/mcp/tools", get(Self::handle_list_tools))
            // 调用端点
            .route("/mcp/invoke", post(Self::handle_invoke))
            // JSON-RPC 端点
            .route("/mcp/rpc", post(Self::handle_rpc))
            // SSE 事件流端点
            .route("/mcp/sse", get(Self::handle_sse))
            // 健康检查
            .route("/mcp/health", get(Self::handle_health))
            .with_state(state)
    }

    /// 处理发现请求 - /.well-known/mcp
    async fn handle_discovery(
        State(state): State<McpServerState>,
    ) -> Json<McpDiscoveryResponse> {
        Json(McpDiscoveryResponse {
            protocol_version: "1.0".to_string(),
            service: state.config.clone(),
            auth_type: "none".to_string(),
            tools_endpoint: "/mcp/tools".to_string(),
            invoke_endpoint: "/mcp/invoke".to_string(),
        })
    }

    /// 处理 MCP 信息请求
    async fn handle_mcp_info(
        State(state): State<McpServerState>,
    ) -> Json<Value> {
        let status = state.mcp_server.status().await;
        let tool_count = state.mcp_server.tool_registry().list_tools().len();
        
        Json(serde_json::json!({
            "name": state.config.name,
            "version": state.config.version,
            "description": state.config.description,
            "status": format!("{:?}", status).to_lowercase(),
            "tool_count": tool_count,
            "capabilities": state.config.capabilities,
            "endpoints": {
                "tools": "/mcp/tools",
                "invoke": "/mcp/invoke",
                "rpc": "/mcp/rpc",
                "sse": "/mcp/sse",
                "health": "/mcp/health",
            }
        }))
    }

    /// 处理工具列表请求
    async fn handle_list_tools(
        State(state): State<McpServerState>,
    ) -> Json<Vec<ToolDescription>> {
        let tools = state.mcp_server.tool_registry().list_tools();
        Json(tools)
    }

    /// 处理工具调用
    async fn handle_invoke(
        State(state): State<McpServerState>,
        Json(params): Json<InvokeParams>,
    ) -> impl IntoResponse {
        let call_id = params.id.unwrap_or_else(|| ::uuid::Uuid::new_v4().to_string());

        // 发送调用开始事件
        let _ = state.event_sender.send(McpServerEvent::ToolCallStarted {
            id: call_id.clone(),
            tool: params.name.clone(),
            timestamp: ::chrono::Utc::now().timestamp(),
        });

        let tool_params = super::protocol::ToolCallParams {
            name: params.name,
            arguments: params.arguments,
        };

        let result = state.mcp_server.tool_registry().call_tool(
            &state.db,
            &tool_params,
        ).await;

        let success = result.is_ok();

        // 发送调用完成事件
        let _ = state.event_sender.send(McpServerEvent::ToolCallCompleted {
            id: call_id.clone(),
            tool: tool_params.name,
            success,
            timestamp: ::chrono::Utc::now().timestamp(),
        });

        match result {
            Ok(data) => (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": call_id,
                    "content": data,
                    "content_type": "application/json",
                    "success": true,
                }))
            ).into_response(),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "id": call_id,
                    "error": e.to_string(),
                    "success": false,
                }))
            ).into_response(),
        }
    }

    /// 处理 JSON-RPC 请求
    async fn handle_rpc(
        State(state): State<McpServerState>,
        Json(request): Json<McpRequest>,
    ) -> Json<McpResponse> {
        Json(state.mcp_server.handle_request(&state.db, request).await)
    }

    /// 处理 SSE 事件流
    async fn handle_sse(
        State(state): State<McpServerState>,
    ) -> Sse<impl tokio_stream::Stream<Item = Result<Event, axum::Error>>> {
        let rx = state.event_sender.subscribe();
        let stream = BroadcastStream::new(rx)
            .filter_map(|result| {
                match result {
                    Ok(event) => {
                        let event_type = match &event {
                            McpServerEvent::ServiceStarted { .. } => "service_started",
                            McpServerEvent::ServiceStopped { .. } => "service_stopped",
                            McpServerEvent::ToolCallStarted { .. } => "tool_call_started",
                            McpServerEvent::ToolCallCompleted { .. } => "tool_call_completed",
                            McpServerEvent::Heartbeat { .. } => "heartbeat",
                        };
                        
                        match serde_json::to_string(&event) {
                            Ok(data) => Some(Ok(Event::default().event(event_type).data(data))),
                            Err(_) => None,
                        }
                    },
                    Err(_) => None,
                }
            });

        Sse::new(stream)
            .keep_alive(KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive")
            )
    }

    /// 健康检查
    async fn handle_health(
        State(state): State<McpServerState>,
    ) -> impl IntoResponse {
        let status = if state.running.load(std::sync::atomic::Ordering::SeqCst) {
            "healthy"
        } else {
            "unhealthy"
        };

        Json(serde_json::json!({
            "status": status,
            "timestamp": ::chrono::Utc::now().timestamp(),
            "version": state.config.version,
        }))
    }
}

/// 创建 MCP 服务配置文件，供 Agent 发现
pub fn create_mcp_config_file(
    output_path: &str,
    config: &McpServiceConfig,
) -> Result<(), String> {
    let discovery = McpDiscoveryResponse {
        protocol_version: "1.0".to_string(),
        service: config.clone(),
        auth_type: "none".to_string(),
        tools_endpoint: "/mcp/tools".to_string(),
        invoke_endpoint: "/mcp/invoke".to_string(),
    };

    let content = serde_json::to_string_pretty(&discovery)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(output_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

/// 获取标准的 MCP 配置目录路径
pub fn get_mcp_config_dir() -> Result<String, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?;
    
    let config_dir = home_dir.join(".mcp").join("services");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    Ok(config_dir.to_string_lossy().to_string())
}

/// 注册 MCP 服务到本地配置目录
pub fn register_mcp_service(config: &McpServiceConfig) -> Result<(), String> {
    let config_dir = get_mcp_config_dir()?;
    let file_name = format!("ai-content-creator-{}.json", config.name.to_lowercase().replace(" ", "-"));
    let file_path = format!("{}/{}", config_dir, file_name);
    
    create_mcp_config_file(&file_path, config)?;
    
    log!("MCP 服务配置已注册到: {}", file_path);
    
    Ok(())
}

/// 注销 MCP 服务
pub fn unregister_mcp_service(config_name: &str) -> Result<(), String> {
    let config_dir = get_mcp_config_dir()?;
    let file_name = format!("ai-content-creator-{}.json", config_name.to_lowercase().replace(" ", "-"));
    let file_path = format!("{}/{}", config_dir, file_name);
    
    if std::path::Path::new(&file_path).exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| format!("删除配置文件失败: {}", e))?;
        log!("MCP 服务配置已注销: {}", file_path);
    }
    
    Ok(())
}
