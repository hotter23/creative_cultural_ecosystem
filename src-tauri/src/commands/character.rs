//! 角色管理相关命令

use crate::log;
use crate::clients::minimax::{MinMaxConfig, MinMaxImageClient};
use crate::db::models::{Character, CharacterImage};
use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

// 创建角色请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCharacterRequest {
    pub novel_id: i64,
    pub name: String,
    pub aliases: Option<String>,
    pub gender: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub appearance: Option<String>,
    pub personality: Option<String>,
    pub voice_id: Option<String>,
    pub tags: Option<String>,
}

// 更新角色请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCharacterRequest {
    pub id: i64,
    pub name: Option<String>,
    pub aliases: Option<String>,
    pub gender: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub appearance: Option<String>,
    pub personality: Option<String>,
    pub voice_id: Option<String>,
    pub tags: Option<String>,
}

// AI 生成角色形象请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCharacterImageRequest {
    pub character_id: i64,
    pub prompt: Option<String>,      // 用户自定义提示词
    pub style: Option<String>,       // 风格: general, anime, photorealistic, painting
    pub size: Option<String>,        // 尺寸: 1024x1024, 768x1024, 1024x768
    pub pose: Option<String>,        // 姿态
    pub expression: Option<String>,  // 表情
}

// 生成的图像结果
#[derive(Debug, Serialize)]
pub struct GeneratedImageResult {
    pub id: String,
    pub url: String,
    pub local_path: Option<String>,
    pub prompt: String,
}

/// 获取小说的所有角色
#[tauri::command]
pub fn get_characters(
    db: State<'_, Arc<Database>>,
    novel_id: Option<i64>,
    #[allow(non_snake_case)] novelId: Option<i64>,
) -> Result<Vec<Character>, String> {
    log!("get_characters: 收到 novel_id = {:?}", novel_id);
    log!("get_characters: 收到 novelId = {:?}", novelId);
    
    let id = novel_id.or(novelId).unwrap_or(0);
    if id == 0 {
        log!("get_characters: 错误 - 缺少 novel_id 或 novelId 参数");
        return Err("缺少 novel_id 或 novelId 参数".to_string());
    }
    log!("get_characters: 使用 id = {}", id);
    db.get_characters_by_novel_id(id)
        .map_err(|e| e.to_string())
}

/// 获取单个角色详情
#[tauri::command]
pub async fn get_character(
    db: State<'_, Arc<Database>>,
    character_id: i64,
) -> Result<Option<Character>, String> {
    db.get_character_by_id(character_id)
        .map_err(|e| e.to_string())
}

/// 创建新角色
#[tauri::command]
pub async fn create_character(
    db: State<'_, Arc<Database>>,
    request: CreateCharacterRequest,
) -> Result<i64, String> {
    let now = chrono::Local::now().to_rfc3339();
    let character = Character {
        id: 0, // 由数据库自动生成
        novel_id: request.novel_id,
        name: request.name,
        aliases: request.aliases,
        gender: request.gender,
        role: request.role,
        description: request.description,
        appearance: request.appearance,
        personality: request.personality,
        voice_id: request.voice_id,
        tags: request.tags,
        created_at: now,
    };

    db.create_character(&character)
        .map_err(|e| e.to_string())
}

/// 更新角色信息
#[tauri::command]
pub async fn update_character(
    db: State<'_, Arc<Database>>,
    request: UpdateCharacterRequest,
) -> Result<(), String> {
    // 获取现有角色数据
    let existing = db.get_character_by_id(request.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "角色不存在".to_string())?;

    // 合并更新
    let updated = Character {
        id: request.id,
        novel_id: existing.novel_id,
        name: request.name.unwrap_or(existing.name),
        aliases: request.aliases.or(existing.aliases),
        gender: request.gender.or(existing.gender),
        role: request.role.or(existing.role),
        description: request.description.or(existing.description),
        appearance: request.appearance.or(existing.appearance),
        personality: request.personality.or(existing.personality),
        voice_id: request.voice_id.or(existing.voice_id),
        tags: request.tags.or(existing.tags),
        created_at: existing.created_at,
    };

    db.update_character(request.id, &updated)
        .map_err(|e| e.to_string())
}

/// 删除角色
#[tauri::command]
pub async fn delete_character(
    db: State<'_, Arc<Database>>,
    character_id: i64,
) -> Result<(), String> {
    db.delete_character(character_id)
        .map_err(|e| e.to_string())
}

/// 获取角色的所有形象图片
#[tauri::command]
pub async fn get_character_images(
    db: State<'_, Arc<Database>>,
    character_id: i64,
) -> Result<Vec<CharacterImage>, String> {
    db.get_character_images(character_id)
        .map_err(|e| e.to_string())
}

/// 删除角色形象
#[tauri::command]
pub async fn delete_character_image(
    db: State<'_, Arc<Database>>,
    image_id: String,
) -> Result<(), String> {
    db.delete_character_image(&image_id)
        .map_err(|e| e.to_string())
}

/// 设置角色默认形象
#[tauri::command]
pub async fn set_default_character_image(
    db: State<'_, Arc<Database>>,
    character_id: i64,
    image_id: String,
) -> Result<(), String> {
    db.set_default_character_image(character_id, &image_id)
        .map_err(|e| e.to_string())
}

/// AI 生成角色形象（调用 MinMax image-01）
#[tauri::command]
pub async fn generate_character_image(
    db: State<'_, Arc<Database>>,
    request: GenerateCharacterImageRequest,
) -> Result<GeneratedImageResult, String> {
    // 获取角色信息
    let character = db.get_character_by_id(request.character_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "角色不存在".to_string())?;

    // 获取 MinMax 配置
    let config_map = db.get_config_by_category("minimax")
        .map_err(|e| e.to_string())?;
    
    let enabled = config_map.get("minimax_enabled")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    
    if !enabled {
        return Err("MinMax 服务未启用，请先在设置中配置并启用 MinMax".to_string());
    }

    let api_key = config_map.get("minimax_api_key")
        .cloned()
        .unwrap_or_default();
    
    if api_key.is_empty() {
        return Err("MinMax API Key 未配置".to_string());
    }

    let base_url = config_map.get("minimax_base_url")
        .cloned()
        .unwrap_or_else(|| "https://api.minimaxi.com".to_string());

    let group_id = config_map.get("minimax_group_id")
        .cloned()
        .filter(|s| !s.is_empty());

    let config = MinMaxConfig {
        api_key,
        group_id,
        base_url,
        default_model: "image-01".to_string(),
        enabled: true,
    };

    // 构建提示词（简化版，避免触发敏感词审核）
    let mut prompt_parts = Vec::new();
    
    // 角色名称
    prompt_parts.push(character.name.clone());
    
    // 性别转换为英文描述
    if let Some(gender) = &character.gender {
        let gender_desc = match gender.as_str() {
            "male" | "男" => "male",
            "female" | "女" => "female",
            _ => "",
        };
        if !gender_desc.is_empty() {
            prompt_parts.push(gender_desc.to_string());
        }
    }
    
    // 外貌特征
    if let Some(appearance) = &character.appearance {
        prompt_parts.push(appearance.clone());
    }
    
    // 性格特征
    if let Some(personality) = &character.personality {
        prompt_parts.push(personality.clone());
    }
    
    // 背景描述（只取前100字）
    if let Some(description) = &character.description {
        let short_desc = if description.chars().count() > 100 {
            description.chars().take(100).collect()
        } else {
            description.clone()
        };
        prompt_parts.push(short_desc);
    }
    
    // 姿态
    if let Some(pose) = &request.pose {
        prompt_parts.push(pose.clone());
    }
    
    // 表情
    if let Some(expression) = &request.expression {
        prompt_parts.push(expression.clone());
    }

    // 构建最终提示词 - 使用更简洁的格式避免审核
    let base_style = "anime character, high quality, detailed, vibrant colors, portrait";
    
    // 用户自定义提示词优先级最高
    let final_prompt = if let Some(custom_prompt) = request.prompt {
        if custom_prompt.is_empty() {
            format!("{}, {}, {}", base_style, prompt_parts.join(", "), character.name)
        } else {
            format!("{}, {}, {}", custom_prompt, base_style, prompt_parts.join(", "))
        }
    } else {
        format!("{}, {}, {}", base_style, prompt_parts.join(", "), character.name)
    };

    // 转换尺寸格式为 aspect_ratio (1024x1024 -> 1:1)
    let aspect_ratio = request.size.as_deref().map(|size| {
        match size {
            "1024x1024" => "1:1",
            "768x1024" => "3:4",
            "1024x768" => "4:3",
            _ => "1:1",
        }
    });

    // 输出最终提示词用于调试
    log!("生成图像的提示词: {}", final_prompt);

    // 调用图像生成 API
    let client = MinMaxImageClient::new(config);
    let images_result = client.generate_image(
        &final_prompt,
        aspect_ratio,
        request.style.as_deref(),
        None, // negative_prompt
        Some(1),
    ).await;

    let images = match images_result {
        Ok(imgs) => imgs,
        Err(e) => {
            // 如果触发敏感词，尝试使用更简单的提示词重试
            if e.contains("sensitive") || e.contains("1026") || e.contains("审核") {
                log!("触发内容审核，使用简化提示词重试...");
                let simple_prompt = format!("{}, anime character", character.name);
                log!("重试提示词: {}", simple_prompt);
                
                client.generate_image(
                    &simple_prompt,
                    aspect_ratio,
                    None,
                    None,
                    Some(1),
                ).await?
            } else {
                return Err(e);
            }
        }
    };

    let generated = images.first()
        .ok_or_else(|| "没有生成图像".to_string())?;

    // 保存到数据库
    let image_id = Uuid::new_v4().as_u64_pair().0 as i64;
    let character_image = CharacterImage {
        id: image_id,
        character_id: request.character_id,
        image_type: Some("ai_generated".to_string()),
        pose: request.pose.clone(),
        expression: request.expression.clone(),
        image_path: generated.url.clone(),
        prompt: Some(final_prompt.clone()),
        seed: None,
        is_default: false,
        created_at: chrono::Local::now().to_rfc3339(),
    };

    db.create_character_image(&character_image)
        .map_err(|e| e.to_string())?;

    Ok(GeneratedImageResult {
        id: image_id.to_string(),
        url: generated.url.clone(),
        local_path: None,
        prompt: final_prompt,
    })
}

/// 从小说内容中提取角色
#[tauri::command]
pub async fn extract_characters_from_content(
    db: State<'_, Arc<Database>>,
    novel_id: i64,
) -> Result<Vec<Character>, String> {
    // 步骤1: 读取配置和章节内容（同步操作）
    let (config_map, chapters) = {
        // 获取 MinMax 配置
        let config_map = db.get_config_by_category("minimax")
            .map_err(|e| e.to_string())?;
        
        let enabled = config_map.get("minimax_enabled")
            .and_then(|v| v.parse().ok())
            .unwrap_or(false);
        
        if !enabled {
            return Err("MinMax 服务未启用，请先在设置中配置并启用 MinMax".to_string());
        }

        // 获取小说章节内容
        let conn = db.get_conn();
        let mut stmt = conn
            .prepare("SELECT plain_text FROM chapters WHERE novel_id = ?1 ORDER BY order_num ASC LIMIT 3")
            .map_err(|e| e.to_string())?;
        
        let content_rows = stmt
            .query_map([novel_id], |row| {
                let text: Option<String> = row.get(0)?;
                Ok(text)
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        (config_map, content_rows)
    };

    // 收集所有内容
    let mut all_content = String::new();
    for content_opt in chapters {
        if let Some(content) = content_opt {
            all_content.push_str(&content);
            all_content.push('\n');
        }
    }

    if all_content.is_empty() {
        return Ok(Vec::new());
    }

    // 限制内容长度
    let content_sample = if all_content.len() > 3000 {
        &all_content[0..3000]
    } else {
        &all_content
    };

    // 步骤2: 调用 AI 提取角色（异步操作，不持有数据库锁）
    let api_key = config_map.get("minimax_api_key")
        .cloned()
        .unwrap_or_default();
    
    let group_id = config_map.get("minimax_group_id")
        .cloned()
        .filter(|s| !s.is_empty());

    let base_url = config_map.get("minimax_base_url")
        .cloned()
        .unwrap_or_else(|| "https://api.minimaxi.com".to_string());

    let config = MinMaxConfig {
        api_key,
        group_id,
        base_url,
        default_model: "abab6.5s-chat".to_string(),
        enabled: true,
    };

    let client = crate::clients::minimax::MinMaxClient::new(config);
    let extracted = client.extract_characters(content_sample).await?;

    // 步骤3: 保存结果到数据库（同步操作）
    let characters = {
        let mut characters = Vec::new();
        for char_info in extracted {
            // 跳过只有 raw 数据的结果
            if char_info.contains_key("raw") && char_info.len() == 1 {
                continue;
            }

            let name = char_info.get("name")
                .or_else(|| char_info.get("姓名"))
                .cloned()
                .unwrap_or_else(|| "未知角色".to_string());

            let gender = char_info.get("gender")
                .or_else(|| char_info.get("性别"))
                .cloned();

            let role = char_info.get("role")
                .or_else(|| char_info.get("角色"))
                .or_else(|| char_info.get("角色定位"))
                .cloned();

            let description = char_info.get("description")
                .or_else(|| char_info.get("描述"))
                .or_else(|| char_info.get("介绍"))
                .cloned();

            let now = chrono::Local::now().to_rfc3339();
            let character = Character {
                id: 0,
                novel_id,
                name,
                aliases: None,
                gender,
                role,
                description,
                appearance: None,
                personality: None,
                voice_id: None,
                tags: None,
                created_at: now.clone(),
            };

            // 保存到数据库
            let char_id = db.create_character(&character)
                .map_err(|e| e.to_string())?;

            characters.push(Character {
                id: char_id,
                ..character
            });
        }
        characters
    };

    Ok(characters)
}
