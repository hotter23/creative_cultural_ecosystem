use std::sync::Arc;
use super::super::clients::minimax::{MinMaxClient, MinMaxConfig, ChatMessage};
use super::super::db::Database;
use tauri::{command, State};
use std::collections::HashMap;

// 获取 MinMax 配置
#[command]
pub async fn get_minimax_config(
    db: State<'_, Arc<Database>>,
) -> Result<HashMap<String, String>, String> {
    let config = db.get_config_by_category("minimax")
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

// 保存 MinMax 配置
#[command]
pub async fn save_minimax_config(
    db: State<'_, Arc<Database>>,
    enabled: bool,
    api_key: String,
    base_url: Option<String>,
    default_model: Option<String>,
    group_id: Option<String>,
) -> Result<bool, String> {
    let enabled_str = enabled.to_string();
    let base_url_str = base_url.unwrap_or_else(|| "https://api.minimaxi.com".to_string());
    let default_model_str = default_model.unwrap_or_else(|| "minimaxi-2.7".to_string());
    let group_id_str = group_id.unwrap_or_default();
    
    let configs = [
        ("minimax_enabled", enabled_str.as_str(), "minimax"),
        ("minimax_api_key", api_key.as_str(), "minimax"),
        ("minimax_base_url", base_url_str.as_str(), "minimax"),
        ("minimax_default_model", default_model_str.as_str(), "minimax"),
        ("minimax_group_id", group_id_str.as_str(), "minimax"),
    ];
    
    db.set_config_batch(&configs)
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

// 测试 MinMax 连接
#[command]
pub async fn test_minimax_connection(
    api_key: String,
    base_url: Option<String>,
    group_id: Option<String>,
) -> Result<bool, String> {
    let config = MinMaxConfig {
        api_key,
        base_url: base_url.unwrap_or_else(|| "https://api.minimaxi.com".to_string()),
        group_id,
        enabled: true,
        ..Default::default()
    };
    
    let client = MinMaxClient::new(config);
    client.test_connection().await
}

// 加载 MinMax 客户端（内部使用）
fn load_minimax_client(db: &Arc<Database>) -> Result<MinMaxClient, String> {
    let config_map = db.get_config_by_category("minimax")
        .map_err(|e| e.to_string())?;
    
    let enabled = config_map.get("minimax_enabled")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    
    if !enabled {
        return Err("MinMax 服务未启用".to_string());
    }
    
    let api_key = config_map.get("minimax_api_key")
        .cloned()
        .unwrap_or_default();
    
    if api_key.is_empty() {
        return Err("MinMax API Key 未配置".to_string());
    }
    
    let config = MinMaxConfig {
        api_key,
        base_url: config_map.get("minimax_base_url")
            .cloned()
            .unwrap_or_else(|| "https://api.minimaxi.com".to_string()),
        default_model: config_map.get("minimax_default_model")
            .cloned()
            .unwrap_or_else(|| "minimaxi-2.7".to_string()),
        group_id: config_map.get("minimax_group_id").cloned(),
        enabled: true,
    };
    
    Ok(MinMaxClient::new(config))
}

// 生成网文内容
#[command]
pub async fn ai_generate_novel_content(
    db: State<'_, Arc<Database>>,
    prompt: String,
    genre: Option<String>,
    word_count: Option<u32>,
) -> Result<String, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.generate_novel_content(
        &prompt,
        &genre.unwrap_or_else(|| "通用".to_string()),
        word_count.unwrap_or(500),
        None,
    ).await?;
    
    Ok(result)
}

// 续写网文内容
#[command]
pub async fn ai_continue_novel_content(
    db: State<'_, Arc<Database>>,
    prefix_content: String,
    genre: Option<String>,
    word_count: Option<u32>,
) -> Result<String, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.continue_novel_content(
        &prefix_content,
        &genre.unwrap_or_else(|| "通用".to_string()),
        word_count.unwrap_or(500),
    ).await?;
    
    Ok(result)
}

// 润色内容
#[command]
pub async fn ai_polish_content(
    db: State<'_, Arc<Database>>,
    content: String,
    style: Option<String>,
) -> Result<String, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.polish_content(
        &content,
        &style.unwrap_or_else(|| "润色优化，让文字更有张力".to_string()),
    ).await?;
    
    Ok(result)
}

// 总结内容
#[command]
pub async fn ai_summarize_content(
    db: State<'_, Arc<Database>>,
    content: String,
    max_length: Option<u32>,
) -> Result<String, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.summarize_content(
        &content,
        max_length.unwrap_or(200),
    ).await?;
    
    Ok(result)
}

// 生成建议
#[command]
pub async fn ai_generate_suggestions(
    db: State<'_, Arc<Database>>,
    content: String,
    genre: Option<String>,
) -> Result<Vec<String>, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.generate_suggestions(
        &content,
        &genre.unwrap_or_else(|| "通用".to_string()),
    ).await?;
    
    Ok(result)
}

// 提取角色
#[command]
pub async fn ai_extract_characters(
    db: State<'_, Arc<Database>>,
    content: String,
) -> Result<Vec<HashMap<String, String>>, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.extract_characters(&content).await?;
    
    Ok(result)
}

// 通用聊天接口（支持自定义对话）
#[command]
pub async fn ai_chat_completion(
    db: State<'_, Arc<Database>>,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
) -> Result<String, String> {
    let client = load_minimax_client(&db)?;
    
    let response = client.chat_completion(messages, false, max_tokens).await?;
    
    if let Some(choice) = response.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("没有返回内容".to_string())
    }
}

// 获取 TTS 配置
#[command]
pub async fn get_tts_config(
    db: State<'_, Arc<Database>>,
) -> Result<HashMap<String, String>, String> {
    let config = db.get_config_by_category("tts")
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

// 保存 TTS 配置
#[command]
pub async fn save_tts_config(
    db: State<'_, Arc<Database>>,
    default_voice: String,
    speed: Option<f32>,
) -> Result<bool, String> {
    let speed_str = speed.unwrap_or(1.0).to_string();
    
    let configs = [
        ("tts_default_voice", default_voice.as_str(), "tts"),
        ("tts_speed", speed_str.as_str(), "tts"),
    ];
    
    db.set_config_batch(&configs)
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

// TTS 文本转语音
#[command]
pub async fn text_to_speech(
    db: State<'_, Arc<Database>>,
    text: String,
    voice_id: Option<String>,
    speed: Option<f32>,
) -> Result<Vec<u8>, String> {
    use super::super::clients::minimax::MinMaxTTSClient;
    
    let config_map = db.get_config_by_category("minimax")
        .map_err(|e| e.to_string())?;
    
    let enabled = config_map.get("minimax_enabled")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    
    if !enabled {
        return Err("MinMax 服务未启用".to_string());
    }
    
    let api_key = config_map.get("minimax_api_key")
        .cloned()
        .unwrap_or_default();
    
    if api_key.is_empty() {
        return Err("MinMax API Key 未配置".to_string());
    }
    
    let tts_config = db.get_config_by_category("tts")
        .map_err(|e| e.to_string())?;
    
    let config = MinMaxConfig {
        api_key,
        base_url: config_map.get("minimax_base_url")
            .cloned()
            .unwrap_or_else(|| "https://api.minimaxi.com".to_string()),
        default_model: String::new(),
        group_id: None,
        enabled: true,
    };
    
    let client = MinMaxTTSClient::new(config);
    
    let voice = voice_id.or_else(|| tts_config.get("tts_default_voice").cloned())
        .unwrap_or_else(|| "voice_female_01".to_string());
    
    let speech_speed = speed.or_else(|| {
        tts_config.get("tts_speed").and_then(|s| s.parse().ok())
    }).unwrap_or(1.0);
    
    client.text_to_speech(&text, &voice, speech_speed).await
}

// 情节建议
#[command]
pub async fn ai_suggest_plot(
    db: State<'_, Arc<Database>>,
    content: String,
) -> Result<String, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.generate_plot_suggestions(&content).await?;
    
    Ok(result)
}

// 简化的聊天接口（带上下文）
#[command]
pub async fn ai_chat(
    db: State<'_, Arc<Database>>,
    prompt: String,
    context: Option<String>,
) -> Result<String, String> {
    let client = load_minimax_client(&db)?;
    
    let result = client.chat_with_context(&prompt, context.as_deref()).await?;
    
    Ok(result)
}
