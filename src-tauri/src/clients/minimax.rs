use crate::log;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMaxConfig {
    pub api_key: String,
    pub group_id: Option<String>,
    pub base_url: String,
    pub default_model: String,
    pub enabled: bool,
}

impl Default for MinMaxConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            group_id: None,
            base_url: "https://api.minimaxi.com".to_string(),
            default_model: "abab6.5s-chat".to_string(),
            enabled: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct MinMaxChatResponse {
    #[allow(dead_code)]
    pub id: Option<String>,
    #[allow(dead_code)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[allow(dead_code)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Usage {
    #[allow(dead_code)]
    pub prompt_tokens: u32,
    #[allow(dead_code)]
    pub completion_tokens: u32,
    #[allow(dead_code)]
    pub total_tokens: u32,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct TTSRequest {
    pub text: String,
    pub voice_id: String,
    pub speed: f32,
    pub pitch: i32,
    pub volume: i32,
}

pub struct MinMaxClient {
    client: Client,
    config: MinMaxConfig,
}

impl MinMaxClient {
    pub fn new(config: MinMaxConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        
        if let Ok(auth_value) = header::HeaderValue::from_str(&format!("Bearer {}", config.api_key)) {
            headers.insert(header::AUTHORIZATION, auth_value);
        }
        
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, config }
    }

    // 测试连接
    pub async fn test_connection(&self) -> Result<bool, String> {
        if !self.config.enabled || self.config.api_key.is_empty() {
            return Err("MinMax 服务未启用或 API Key 为空".to_string());
        }

        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }
        ];

        match self.chat_completion(messages, false, Some(100)).await {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("连接测试失败: {}", e)),
        }
    }

    // 通用聊天补全接口
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        stream: bool,
        max_tokens: Option<u32>,
    ) -> Result<MinMaxChatResponse, String> {
        if !self.config.enabled {
            return Err("MinMax 服务未启用".to_string());
        }

        let mut payload = json!({
            "model": self.config.default_model,
            "messages": messages,
            "stream": stream,
            "temperature": 0.7,
            "top_p": 0.9,
        });

        if let Some(tokens) = max_tokens {
            payload["max_tokens"] = json!(tokens);
        }

        // 添加 group_id 如果存在
        if let Some(group_id) = &self.config.group_id {
            if !group_id.is_empty() {
                payload["group_id"] = json!(group_id);
            }
        }

        let response = self.client
            .post(format!("{}/v1/text/chatcompletion_v2", self.config.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API 返回错误: 状态码 {}, 内容: {}", status, error_text));
        }

        // 先获取文本内容以便调试
        let response_text = response.text().await.unwrap_or_default();
        
        let result = serde_json::from_str::<MinMaxChatResponse>(&response_text)
            .map_err(|e| format!("解析响应失败: {}, 响应内容: {}", e, response_text))?;

        Ok(result)
    }

    // 网文内容生成
    pub async fn generate_novel_content(
        &self,
        prompt: &str,
        genre: &str,
        word_count: u32,
        system_prompt: Option<&str>,
    ) -> Result<String, String> {
        let default_system = format!(
            "你是一个专业的网文作家，擅长创作{}类型的小说。请生成约{}字的内容，内容要吸引人，符合网文的节奏和特点。",
            genre, word_count
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.unwrap_or(&default_system).to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];

        let response = self.chat_completion(messages, false, Some(word_count * 2)).await?;
        
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("没有返回内容".to_string())
        }
    }

    // 内容续写
    pub async fn continue_novel_content(
        &self,
        prefix_content: &str,
        genre: &str,
        word_count: u32,
    ) -> Result<String, String> {
        let system_prompt = format!(
            "你是一个专业的网文作家，擅长创作{}类型的小说。请根据已有的内容继续续写，约{}字，保持风格和人物一致性，情节要有推进。",
            genre, word_count
        );

        let prompt = format!(
            "已有内容：\n{}\n\n请继续续写：",
            if prefix_content.len() > 1000 {
                &prefix_content[prefix_content.len() - 1000..]
            } else {
                prefix_content
            }
        );

        self.generate_novel_content(&prompt, genre, word_count, Some(&system_prompt)).await
    }

    // 内容润色
    pub async fn polish_content(
        &self,
        content: &str,
        style: &str,
    ) -> Result<String, String> {
        let system_prompt = format!(
            "请{}以下的网文内容，保持原意不变，但让文字更有吸引力。只返回修改后的内容。",
            style
        );

        let prompt = format!("需要修改的内容：\n{}", content);

        self.generate_novel_content(&prompt, "通用", content.len() as u32 / 2, Some(&system_prompt)).await
    }

    // 内容摘要
    pub async fn summarize_content(
        &self,
        content: &str,
        max_length: u32,
    ) -> Result<String, String> {
        let system_prompt = format!(
            "请总结以下网文内容的要点，不超过{}字。",
            max_length
        );

        let prompt = format!("需要总结的内容：\n{}", content);

        self.generate_novel_content(&prompt, "通用", max_length, Some(&system_prompt)).await
    }

    // 生成建议
    pub async fn generate_suggestions(
        &self,
        content: &str,
        genre: &str,
    ) -> Result<Vec<String>, String> {
        let system_prompt = format!(
            "你是一个网文创作顾问。基于当前的{}小说内容，给出3-5个后续情节发展的建议。每个建议用简洁的语言描述。",
            genre
        );

        let prompt = format!("当前内容：\n{}\n\n请给出后续情节发展建议：", content);

        let result = self.generate_novel_content(&prompt, genre, 500, Some(&system_prompt)).await?;
        
        // 分割建议
        let suggestions: Vec<String> = result
            .split('\n')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        Ok(suggestions)
    }

    // 提取角色
    pub async fn extract_characters(
        &self,
        content: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let system_prompt = "请从以下网文内容中提取出现的角色信息，包括姓名、性别、角色定位（主角、配角、反派等）。返回JSON格式的数组。";

        let prompt = format!("内容：\n{}", content);

        let result = self.generate_novel_content(&prompt, "通用", 500, Some(system_prompt)).await?;
        
        // 尝试解析 JSON
        match serde_json::from_str::<Vec<HashMap<String, String>>>(&result) {
            Ok(characters) => Ok(characters),
            Err(_) => {
                // 如果解析失败，返回简单的格式
                let mut chars = Vec::new();
                let mut map = HashMap::new();
                map.insert("raw".to_string(), result);
                chars.push(map);
                Ok(chars)
            }
        }
    }

    // 生成情节建议
    pub async fn generate_plot_suggestions(
        &self,
        content: &str,
    ) -> Result<String, String> {
        let system_prompt = "你是一个资深的网文编辑和创作顾问。基于当前的小说内容，请给出有建设性的后续情节发展建议。\
        建议要具体、有创意，能够推动故事发展，制造冲突和悬念。\
        请分点列出3-5个建议，每个建议要有明确的发展方向和可能的戏剧冲突。";

        let prompt = format!(
            "当前小说内容：\n{}\n\n请基于以上内容，给出后续情节发展的建议：",
            if content.len() > 2000 {
                &content[content.len() - 2000..]
            } else {
                content
            }
        );

        self.generate_novel_content(&prompt, "通用", 1000, Some(system_prompt)).await
    }

    // 带上下文的聊天接口
    pub async fn chat_with_context(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<String, String> {
        let mut messages = Vec::new();

        // 添加系统提示
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: "你是一个专业的网文创作助手，能够帮助作者进行创作、润色、构思情节等工作。\
            请根据作者的需求提供专业、有建设性的帮助。".to_string(),
        });

        // 如果有上下文，添加上下文
        if let Some(ctx) = context {
            if !ctx.is_empty() {
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: format!("当前小说内容参考：\n{}\n\n", if ctx.len() > 1500 {
                        &ctx[ctx.len() - 1500..]
                    } else {
                        ctx
                    }),
                });
            }
        }

        // 添加用户问题
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let response = self.chat_completion(messages, false, Some(2000)).await?;
        
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("没有返回内容".to_string())
        }
    }
}

// TTS 客户端（如果 MinMax 支持的话）
#[allow(dead_code)]
pub struct MinMaxTTSClient {
    client: Client,
    config: MinMaxConfig,
}

// API 基础响应包装
#[derive(Debug, Deserialize)]
struct BaseResp {
    status_code: i32,
    status_msg: String,
}

// TTS 任务创建响应 - task_id 可能是数字或字符串
#[derive(Debug, Deserialize)]
struct TtsTaskResponse {
    base_resp: BaseResp,
    task_id: Option<serde_json::Value>,  // 可能是数字或字符串
    #[allow(dead_code)]
    task_token: Option<String>,
    #[allow(dead_code)]
    file_id: Option<serde_json::Value>,  // 可能是数字或字符串
    #[allow(dead_code)]
    status: Option<String>,
}

// TTS 任务状态响应
#[derive(Debug, Deserialize)]
struct TtsStatusResponse {
    base_resp: BaseResp,
    status: Option<String>,
    #[allow(dead_code)]
    file_id: Option<serde_json::Value>,  // 可能是数字或字符串
    #[allow(dead_code)]
    duration: Option<f32>,
    error: Option<String>,
}

impl MinMaxTTSClient {
    pub fn new(config: MinMaxConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        
        // MiniMax API 使用 Authorization: Bearer {API_Key} 格式
        let auth_value = format!("Bearer {}", config.api_key);
        
        if let Ok(auth_header) = header::HeaderValue::from_str(&auth_value) {
            headers.insert(header::AUTHORIZATION, auth_header);
        }
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, config }
    }

    // 查询 TTS 任务状态
    async fn get_tts_status(&self, task_id: &str) -> Result<TtsStatusResponse, String> {
        // 构建查询 URL，task_id 作为查询参数
        let mut url = format!("{}/v1/query/t2a_async_query_v2?task_id={}", self.config.base_url, task_id);
        
        if let Some(group_id) = &self.config.group_id {
            if !group_id.is_empty() {
                url.push_str(&format!("&group_id={}", group_id));
            }
        }

        // 添加详细的调试信息
        log!("=== 查询 TTS 任务状态 ===");
        log!("请求 URL: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("查询 TTS 状态失败: {}", e))?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;

        // 打印响应详细信息
        log!("响应状态码: {}", status);
        log!("响应完整内容: {}", response_text);

        if !status.is_success() {
            return Err(format!("API 返回错误: 状态码 {}, 内容: {}", status, response_text));
        }

        // 检查 API 响应状态
        let result: TtsStatusResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("解析响应失败: {}, 响应内容: {}", e, response_text))?;

        if result.base_resp.status_code != 200 && result.base_resp.status_code != 0 {
            return Err(format!("API 错误: {} (状态码: {})", result.base_resp.status_msg, result.base_resp.status_code));
        }

        log!("查询成功，任务状态: {:?}", result);

        Ok(result)
    }

    // 创建 TTS 任务
    async fn create_tts_task(
        &self,
        text: &str,
        voice_id: &str,
        speed: f32,
    ) -> Result<(String, String), String> {  // 返回 (task_id, task_token)
        let payload = json!({
            "model": "speech-2.8-hd",
            "text": text,
            "voice_setting": {
                "voice_id": voice_id,
                "speed": speed,
                "vol": 1.0,
                "pitch": 0
            },
            "audio_setting": {
                "audio_sample_rate": 24000,
                "bitrate": 128000,
                "format": "mp3",
                "channel": 1
            }
        });

        // group_id 作为 URL 参数传递
        let mut url = format!("{}/v1/t2a_async_v2", self.config.base_url);
        
        if let Some(group_id) = &self.config.group_id {
            if !group_id.is_empty() {
                url.push_str(&format!("?group_id={}", group_id));
            }
        }

        log!("\n=== 发送 TTS 请求 ===");
        log!("URL: {}", url);
        log!("Payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("创建 TTS 任务失败: {}", e))?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;

        log!("响应状态: {}", status);
        log!("响应内容: {}", response_text);

        if !status.is_success() {
            return Err(format!("API 返回错误: 状态码 {}, 内容: {}", status, response_text));
        }

        let result: TtsTaskResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("解析响应失败: {}, 响应内容: {}", e, response_text))?;

        // 检查 API 响应状态
        if result.base_resp.status_code != 200 && result.base_resp.status_code != 0 {
            return Err(format!("API 错误: {} (状态码: {})", result.base_resp.status_msg, result.base_resp.status_code));
        }

        // 将 task_id 转换为字符串
        let task_id_str = match result.task_id {
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::String(s)) => s,
            _ => return Err(format!("无效的 task_id 格式: {}", response_text)),
        };

        // 获取 task_token
        let task_token = result.task_token.ok_or_else(|| format!("API 未返回 task_token: {}", response_text))?;

        Ok((task_id_str, task_token))
    }

    // 下载音频文件
    pub async fn download_audio(&self, task_id: &str, task_token: &str) -> Result<Vec<u8>, String> {
        // 下载 URL，注意域名可能是 api.minimax.chat，而不是 api.minimaxi.com
        let mut url = format!("https://api.minimax.chat/query/t2a_async_download?task_id={}&task_token={}", task_id, task_token);
        
        // 如果有 GroupId，添加到 URL 中
        if let Some(group_id) = &self.config.group_id {
            url.push_str(&format!("&GroupId={}", group_id));
        }

        log!("下载音频 URL: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("下载音频失败: {}", e))?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
            return Err(format!("API 返回错误: 状态码 {}, 内容: {}", status, error_text));
        }

        // 解析 JSON 响应获取 download_url
        let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
        
        #[derive(Debug, Deserialize)]
        struct DownloadResponse {
            download_url: String,
            #[allow(dead_code)]
            download_count: i32,
            #[allow(dead_code)]
            base_resp: BaseResp,
        }
        
        let download_response: DownloadResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("解析下载响应失败: {}, 响应内容: {}", e, response_text))?;

        log!("获取到下载链接: {}", download_response.download_url);
        
        // 下载压缩文件 - 创建一个新的请求，不包含不必要的请求头
        let tar_response = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?
            .get(&download_response.download_url)
            .send()
            .await
            .map_err(|e| format!("下载压缩文件失败: {}", e))?;

        let tar_status = tar_response.status();
        
        if !tar_status.is_success() {
            let error_text = tar_response.text().await.map_err(|e| format!("读取压缩文件响应失败: {}", e))?;
            return Err(format!("压缩文件下载 API 返回错误: 状态码 {}, 内容: {}", tar_status, error_text));
        }

        let tar_data = tar_response.bytes()
            .await
            .map_err(|e| format!("读取压缩文件数据失败: {}", e))?;

        log!("压缩文件下载成功，大小: {} bytes", tar_data.len());

        // 解压缩文件并获取 MP3
        let mp3_data = self.extract_mp3_from_tar(&tar_data).await?;

        Ok(mp3_data)
    }

    // 从 tar 文件中提取 MP3 文件
    async fn extract_mp3_from_tar(&self, tar_data: &[u8]) -> Result<Vec<u8>, String> {
        use tar::Archive;
        use std::io::{Cursor, Read};

        let cursor = Cursor::new(tar_data);
        let mut archive = Archive::new(cursor);

        // 遍历归档文件
        for entry in archive.entries().map_err(|e| format!("解压缩失败: {}", e))? {
            let mut entry = entry.map_err(|e| format!("读取归档条目失败: {}", e))?;
            
            let path = entry.path().map_err(|e| format!("获取文件路径失败: {}", e))?;
            
            // 检查是否是 MP3 文件
            if let Some(ext) = path.extension() {
                if ext == "mp3" {
                    log!("找到 MP3 文件: {:?}", path);
                    
                    let mut mp3_data = Vec::new();
                    entry.read_to_end(&mut mp3_data).map_err(|e| format!("读取 MP3 文件失败: {}", e))?;
                    
                    log!("MP3 文件大小: {} bytes", mp3_data.len());
                    return Ok(mp3_data);
                }
            }
        }

        Err("在压缩文件中未找到 MP3 文件".to_string())
    }

    // 创建异步语音合成任务（只创建任务，不自动下载）
    pub async fn create_tts_task_only(
        &self,
        text: &str,
        voice_id: &str,
        speed: f32,
    ) -> Result<(String, String), String> {
        let (task_id, task_token) = self.create_tts_task(text, voice_id, speed).await?;
        log!("异步 TTS 任务创建成功，task_id: {}, task_token: [token]", task_id);
        Ok((task_id, task_token))
    }

    // 查询异步语音合成状态
    #[allow(dead_code)]
    pub async fn query_tts_status(
        &self,
        task_id: &str,
    ) -> Result<String, String> {
        let status = self.get_tts_status(task_id).await?;
        
        let status_str = status.status.unwrap_or_else(|| "unknown".to_string());
        log!("任务状态: {}", status_str);
        
        match status_str.as_str() {
            "Success" => {
                return Ok("任务已完成".to_string());
            }
            "Failed" => {
                return Err(format!("TTS 任务失败: {:?}", status.error));
            }
            "Processing" => {
                return Ok("任务正在处理中...".to_string());
            }
            "Expired" => {
                return Err("任务已过期".to_string());
            }
            _ => {
                return Err(format!("未知任务状态: {}", status_str));
            }
        }
    }

    // 文本转语音（完整流程：创建任务 -> 轮询状态 -> 下载音频）
    pub async fn text_to_speech(
        &self,
        text: &str,
        voice_id: &str,
        speed: f32,
    ) -> Result<Vec<u8>, String> {
        let (task_id, task_token) = self.create_tts_task(text, voice_id, speed).await?;
        
        log!("TTS 任务创建成功，task_id: {}, task_token: [token]", task_id);
        
        // 轮询任务状态
        let mut attempts = 0;
        let max_attempts = 60;
        let poll_interval = std::time::Duration::from_secs(2);
        
        while attempts < max_attempts {
            let status = self.get_tts_status(&task_id).await?;
            
            let status_str = status.status.unwrap_or_else(|| "unknown".to_string());
            log!("任务状态: {}", status_str);
            
            match status_str.as_str() {
                "Success" => {
                    log!("任务已成功完成，开始下载音频...");
                    let audio_data = self.download_audio(&task_id, &task_token).await?;
                    return Ok(audio_data);
                }
                "Failed" => {
                    return Err(format!("TTS 任务失败: {:?}", status.error));
                }
                "Processing" => {
                    log!("任务正在处理中...");
                    tokio::time::sleep(poll_interval).await;
                    attempts += 1;
                }
                "Expired" => {
                    return Err("任务已过期".to_string());
                }
                _ => {
                    return Err(format!("未知任务状态: {}", status_str));
                }
            }
        }

        Err("TTS 任务超时".to_string())
    }
}

// ==================== 图像生成相关 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratedImage {
    pub url: String,
    pub revised_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageGenerationData {
    pub image_base64: Option<Vec<String>>,
    #[allow(dead_code)]
    pub url: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ImageGenerationResponse {
    #[allow(dead_code)]
    pub created: Option<i64>,
    pub data: ImageGenerationData,
}

/// MinMax 图像生成客户端（用于 image-01 模型）
pub struct MinMaxImageClient {
    client: Client,
    config: MinMaxConfig,
}

impl MinMaxImageClient {
    pub fn new(config: MinMaxConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        
        if let Ok(auth_value) = header::HeaderValue::from_str(&format!("Bearer {}", config.api_key)) {
            headers.insert(header::AUTHORIZATION, auth_value);
        }
        
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, config }
    }

    /// 生成图像
    pub async fn generate_image(
        &self,
        prompt: &str,
        size: Option<&str>,
        _style: Option<&str>,
        _negative_prompt: Option<&str>,
        _n: Option<i32>,
    ) -> Result<Vec<GeneratedImage>, String> {
        if !self.config.enabled {
            return Err("MinMax 服务未启用".to_string());
        }

        if self.config.api_key.is_empty() {
            return Err("API Key 为空".to_string());
        }

        let mut payload = json!({
            "model": "image-01",
            "prompt": prompt,
            "response_format": "base64",
        });

        // 添加可选参数 - 注意：Image-01 模型不支持 style, negative_prompt 和 n 参数
        if let Some(s) = size {
            payload["aspect_ratio"] = json!(s);
        }

        // 添加 group_id 如果存在
        if let Some(group_id) = &self.config.group_id {
            if !group_id.is_empty() {
                payload["group_id"] = json!(group_id);
            }
        }

        let response = self.client
            .post(format!("{}/v1/image_generation", self.config.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("图像生成请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API 返回错误: 状态码 {}, 内容: {}", status, error_text));
        }

        let response_text = response.text().await.unwrap_or_default();
        
        let result = serde_json::from_str::<ImageGenerationResponse>(&response_text)
            .map_err(|e| format!("解析响应失败: {}, 响应内容: {}", e, response_text))?;

        // 转换为 GeneratedImage 格式（使用 base64 数据创建 data URL）
        let images = result.data.image_base64.unwrap_or_default().into_iter()
            .map(|b64_data| GeneratedImage {
                url: format!("data:image/jpeg;base64,{}", b64_data),
                revised_prompt: None,
            })
            .collect();

        Ok(images)
    }

    /// 下载图像并保存到本地（预留功能，暂未使用）
    #[allow(dead_code)]
    pub async fn download_image(&self, url: &str, save_path: &std::path::Path) -> Result<(), String> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("下载图像失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("下载图像失败，状态码: {}", response.status()));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("读取图像数据失败: {}", e))?;

        std::fs::write(save_path, bytes)
            .map_err(|e| format!("保存图像失败: {}", e))?;

        Ok(())
    }
}
