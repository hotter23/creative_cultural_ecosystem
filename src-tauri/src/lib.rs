// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "256"]

mod clients;
mod commands;
mod db;
mod mcp;

use db::Database;
use mcp::McpServer;
use std::sync::Arc;
use tauri::Manager;
use commands::mcp::McpHttpServerState;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::io::Write;
use std::fs::OpenOptions;
use std::path::PathBuf;

/// 全局日志文件句柄
static LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 自定义日志宏，同时输出到控制台和日志文件
/// 使用方式：log!("消息 {}", args);
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        let msg = format!($($arg)*);
        println!("{}", msg);
        crate::write_to_log(&msg);
    });
}

/// 警告日志宏
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => ({
        let msg = format!("[警告] {}", format!($($arg)*));
        eprintln!("{}", msg);
        crate::write_to_log(&msg);
    });
}

/// 错误日志宏
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => ({
        let msg = format!("[错误] {}", format!($($arg)*));
        eprintln!("{}", msg);
        crate::write_to_log(&msg);
    });
}

/// 调试日志宏
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        {
            let msg = format!("[调试] {}", format!($($arg)*));
            println!("{}", msg);
            crate::write_to_log(&msg);
        }
    });
}

/// 写入日志到文件
fn write_to_log(msg: &str) {
    if let Ok(guard) = LOG_FILE.lock() {
        if let Some(ref log_path) = *guard {
            // 获取今天的日期
            let today = chrono::Local::now().format("%Y%m%d").to_string();
            let log_file = log_path.join(format!("short_video_{}.log", today));
            
            // 以追加模式打开文件
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
            {
                // 添加时间戳
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let log_line = format!("[{}] {}\n", timestamp, msg);
                
                // 写入文件并立即刷新
                if let Err(e) = file.write_all(log_line.as_bytes()) {
                    log_error!("写入日志失败: {}", e);
                }
                let _ = file.flush();
            }
        }
    }
}

/// 初始化日志系统，将日志输出到文件
fn init_logging() {
    // 获取日志目录路径
    let log_dir = get_log_dir();
    
    // 创建日志目录
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        log_error!("创建日志目录失败: {}", e);
        return;
    }
    
    // 保存日志目录路径
    {
        let mut guard = LOG_FILE.lock().unwrap();
        *guard = Some(log_dir.clone());
    }
    
    // 写入初始化日志
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let log_file_path = log_dir.join(format!("short_video_{}.log", today));
    
    let init_msg = format!(
        "[{}] === 短剧AI系统启动 ===\n[日志] 日志文件保存在: {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        log_file_path.display()
    );
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        let _ = file.write_all(init_msg.as_bytes());
        let _ = file.flush();
    }
    
    log!("[日志] 日志系统已初始化");
    log!("[日志] 日志文件保存在: {}", log_file_path.display());
}

/// 获取日志目录路径
fn get_log_dir() -> PathBuf {
    // 优先使用 exe 所在目录（打包后应该在这里）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("logs");
        }
    }
    
    // fallback: 使用当前目录的 logs 文件夹
    PathBuf::from("logs")
}

pub fn run() {
    // 初始化日志系统
    init_logging();
    
    log!("=== 短剧AI系统启动 ===");
    
    tauri::Builder::default()
        .setup(|app| {
            // 初始化数据库
            let db = Arc::new(Database::new(app.handle())?);
            app.manage(db.clone());
            
            // 初始化 MCP 服务器
            let mcp_server = Arc::new(McpServer::new());
            app.manage(mcp_server.clone());
            
            // 初始化 MCP HTTP 服务器状态
            let http_server_state = Arc::new(McpHttpServerState::default());
            app.manage(http_server_state.clone());
            
            // 自动启动 MCP 服务（根据配置）
            tauri::async_runtime::spawn(async move {
                if let Err(e) = auto_start_mcp_services(db, mcp_server, http_server_state).await {
                    log_error!("MCP 自动启动失败: {}", e);
                }
            });
            
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            // 网文命令
            commands::novel::get_novels,
            commands::novel::get_novel,
            commands::novel::create_novel,
            commands::novel::update_novel,
            commands::novel::delete_novel,
            
            // 章节命令
            commands::chapter::get_chapters,
            commands::chapter::get_chapter,
            commands::chapter::create_chapter,
            commands::chapter::update_chapter,
            commands::chapter::delete_chapter,
            commands::chapter::save_chapter_paragraphs,
            commands::chapter::get_chapter_paragraphs,
            commands::chapter::delete_chapter_paragraphs,
            
            // AI 相关命令
            commands::ai::get_minimax_config,
            commands::ai::save_minimax_config,
            commands::ai::test_minimax_connection,
            commands::ai::ai_generate_novel_content,
            commands::ai::ai_continue_novel_content,
            commands::ai::ai_polish_content,
            commands::ai::ai_summarize_content,
            commands::ai::ai_generate_suggestions,
            commands::ai::ai_extract_characters,
            commands::ai::ai_chat_completion,
            commands::ai::ai_suggest_plot,
            commands::ai::ai_chat,
            commands::ai::get_tts_config,
            commands::ai::save_tts_config,
            commands::ai::text_to_speech,
            
            // MCP 服务命令
            commands::mcp::mcp_get_status,
            commands::mcp::mcp_start,
            commands::mcp::mcp_stop,
            commands::mcp::mcp_pause,
            commands::mcp::mcp_resume,
            commands::mcp::mcp_list_tools,
            commands::mcp::mcp_call_tool,
            commands::mcp::mcp_handle_request,
            commands::mcp::mcp_handle_request_str,
            commands::mcp::mcp_get_config,
            commands::mcp::mcp_save_config,
            
            // MCP HTTP 服务器命令
            commands::mcp::mcp_http_server_start,
            commands::mcp::mcp_http_server_stop,
            commands::mcp::mcp_register_service,
            commands::mcp::mcp_unregister_service,
            commands::mcp::mcp_get_config_dir,
            
            // 角色管理命令
            commands::character::get_characters,
            commands::character::get_character,
            commands::character::create_character,
            commands::character::update_character,
            commands::character::delete_character,
            commands::character::get_character_images,
            commands::character::delete_character_image,
            commands::character::set_default_character_image,
            commands::character::generate_character_image,
            commands::character::extract_characters_from_content,
            
            // 音频命令
            commands::audio::get_voice_list,
            commands::audio::get_novel_audios,
            commands::audio::get_chapter_audio_detail,
            commands::audio::get_audio_paragraphs,
            commands::audio::generate_chapter_audio,
            commands::audio::update_audio_progress,
            commands::audio::mark_audio_completed,
            commands::audio::delete_chapter_audio,
            commands::audio::regenerate_paragraph_audio,
            commands::audio::update_paragraph_params,
            commands::audio::update_paragraph_ambient_sound,
            commands::audio::copy_ambient_to_paragraph,
            commands::audio::get_audio_storage_dir,
            commands::audio::query_audio_status,
            commands::audio::download_paragraph_audio,
            commands::audio::get_audio_stream,
            commands::audio::merge_chapter_audio,
            
            // 系统相关命令
            commands::system::save_video_config,
            commands::system::get_video_config,
            commands::system::save_python_config,
            commands::system::get_python_config,
            // 环境音命令
            commands::ambient::get_ambient_sounds,
            commands::ambient::get_chapter_ambient_config,
            commands::ambient::save_chapter_ambient_config,
            commands::ambient::delete_chapter_ambient_config,
            commands::ambient::generate_ambient_sound,
            commands::ambient::generate_ambient_sound_stable_audio,
            commands::ambient::delete_ambient_sound,
            commands::ambient::mix_voice_with_ambient,
            
            // 日志命令
            commands::logger::log_to_file,
            commands::logger::log_error_to_file,
            
            // 混音命令
            commands::mixer::get_paragraph_ambient_configs,
            commands::mixer::save_paragraph_ambient_config,
            commands::mixer::batch_save_paragraph_ambient_configs,
            commands::mixer::delete_paragraph_ambient_config,
            commands::mixer::get_chapter_paragraph_ambient_configs,
            commands::mixer::get_mix_presets,
            commands::mixer::save_mix_preset,
            commands::mixer::delete_mix_preset,
            commands::mixer::mix_paragraph_audio,
            commands::mixer::batch_mix_paragraphs,
            commands::mixer::mix_chapter_audio,
            commands::mixer::mix_chapter_with_same_ambient,
            commands::mixer::get_chapter_mix_status,
            commands::mixer::get_chapter_paragraphs_for_mix,
            commands::mixer::clear_chapter_mix,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 根据配置自动启动 MCP 服务
async fn auto_start_mcp_services(
    db: Arc<Database>,
    mcp_server: Arc<McpServer>,
    http_server_state: Arc<McpHttpServerState>,
) -> Result<(), String> {
    // 读取配置
    let enabled = match db.get_config("mcp_enabled") {
        Ok(Some(value)) => value == "true",
        _ => true, // 默认启用
    };
    
    let auto_start = match db.get_config("mcp_auto_start") {
        Ok(Some(value)) => value == "true",
        _ => false, // 默认不自动启动
    };
    
    let port = match db.get_config("mcp_port") {
        Ok(Some(value)) => value.parse::<u16>().unwrap_or(8787),
        _ => 8787,
    };
    
    let bind_address = match db.get_config("mcp_bind_address") {
        Ok(Some(value)) => value,
        _ => "127.0.0.1".to_string(),
    };
    
    // 解析绑定地址
    let ip = match bind_address.as_str() {
        "0.0.0.0" => std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        "127.0.0.1" => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        addr => addr.parse().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
    };
    
    if enabled && auto_start {
        log!("MCP 自动启动已启用，正在启动 MCP 服务...");
        
        // 启动 MCP 核心服务
        mcp_server.start().await
            .map_err(|e| format!("启动 MCP 核心服务失败: {}", e))?;
        log!("MCP 核心服务已启动");
        
        // 启动 HTTP 服务器
        let config = mcp::McpServiceConfig {
            http_endpoint: format!("http://{}:{}/mcp", bind_address, port),
            sse_endpoint: format!("http://{}:{}/mcp/sse", bind_address, port),
            ..mcp::McpServiceConfig::default()
        };
        
        let http_server = Arc::new(mcp::McpHttpServer::new(
            mcp_server.clone(),
            db.clone(),
            Some(config),
        ));
        
        let server_clone = http_server.clone();
        let addr = SocketAddr::from((ip, port));
        
        let server_task = tokio::spawn(async move {
            if let Err(e) = server_clone.start(addr).await {
                log_error!("MCP HTTP 服务器错误: {}", e);
            }
        });
        
        // 存储服务器状态
        let mut server_lock = http_server_state.server.lock().await;
        *server_lock = Some(http_server);
        
        let mut task_lock = http_server_state.server_task.lock().await;
        *task_lock = Some(server_task);
        
        // 等待服务器启动
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        log!("MCP HTTP 服务器已启动在 {}:{}", bind_address, port);
        log!("MCP 服务端点: http://{}:{}/mcp", bind_address, port);
    } else {
        log!("MCP 自动启动已禁用（enabled={}, auto_start={})", enabled, auto_start);
        log!("可以通过应用设置或调用 Tauri 命令手动启动 MCP 服务");
        log!("可用的 MCP 端点:");
        log!("  GET http://{}:{}/.well-known/mcp", bind_address, port);
        log!("  GET http://{}:{}/mcp/tools", bind_address, port);
        log!("  POST http://{}:{}/mcp/invoke", bind_address, port);
    }
    
    Ok(())
}
