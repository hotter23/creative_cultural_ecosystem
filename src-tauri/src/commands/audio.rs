//! 音频相关 Tauri 命令

use crate::log;
use std::{sync::Arc, path::PathBuf};
use tauri::State;
use serde_json::{json, Value};

use crate::{
    clients::minimax::{MinMaxConfig, MinMaxTTSClient},
    db::{models::*, Database},
    db::DbError,
};

// MiniMax 系统音色列表（统一配置）
pub const VOICE_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    ("中文（普通话）", &[
        ("male-qn-qingse", "青涩青年"),
        ("male-qn-jingying", "精英青年"),
        ("male-qn-badao", "霸道青年"),
        ("male-qn-daxuesheng", "青年大学生"),
        ("female-shaonv", "少女"),
        ("female-yujie", "御姐"),
        ("female-chengshu", "成熟女性"),
        ("female-tianmei", "甜美女性"),
        ("male-qn-qingse-jingpin", "青涩青年(精)"),
        ("male-qn-jingying-jingpin", "精英青年(精)"),
        ("male-qn-badao-jingpin", "霸道青年(精)"),
        ("male-qn-daxuesheng-jingpin", "大学生(精)"),
        ("female-shaonv-jingpin", "少女(精)"),
        ("female-yujie-jingpin", "御姐(精)"),
        ("female-chengshu-jingpin", "成熟女性(精)"),
        ("female-tianmei-jingpin", "甜美女性(精)"),
        ("clever_boy", "聪明男童"),
        ("cute_boy", "可爱男童"),
        ("lovely_girl", "萌萌女童"),
        ("cartoon_pig", "卡通猪小琪"),
        ("bingjiao_didi", "病娇弟弟"),
        ("junlang_nanyou", "俊朗男友"),
        ("chunzhen_xuedi", "纯真学弟"),
        ("lengdan_xiongzhang", "冷淡学长"),
        ("badao_shaoye", "霸道少爷"),
        ("tianxin_xiaoling", "甜心小玲"),
        ("qiaopi_mengmei", "俏皮萌妹"),
        ("wumei_yujie", "妩媚御姐"),
        ("diadia_xuemei", "嗲嗲学妹"),
        ("danya_xuejie", "淡雅学姐"),
        ("Chinese (Mandarin)_Reliable_Executive", "沉稳高管"),
        ("Chinese (Mandarin)_News_Anchor", "新闻女声"),
        ("Chinese (Mandarin)_Mature_Woman", "傲娇御姐"),
        ("Chinese (Mandarin)_Unrestrained_Young_Man", "不羁青年"),
        ("Chinese (Mandarin)_Arrogant_Miss", "嚣张小姐"),
        ("Chinese (Mandarin)_Robot_Armor", "机械战甲"),
        ("Chinese (Mandarin)_Kind-hearted_Antie", "热心大婶"),
        ("Chinese (Mandarin)_HK_Flight_Attendant", "港普空姐"),
        ("Chinese (Mandarin)_Humorous_Elder", "搞笑大爷"),
        ("Chinese (Mandarin)_Gentleman", "温润男声"),
        ("Chinese (Mandarin)_Warm_Bestie", "温暖闺蜜"),
        ("Chinese (Mandarin)_Male_Announcer", "播报男声"),
        ("Chinese (Mandarin)_Sweet_Lady", "甜美女声"),
        ("Chinese (Mandarin)_Southern_Young_Man", "南方小哥"),
        ("Chinese (Mandarin)_Wise_Women", "阅历姐姐"),
        ("Chinese (Mandarin)_Gentle_Youth", "温润青年"),
        ("Chinese (Mandarin)_Warm_Girl", "温暖少女"),
        ("Chinese (Mandarin)_Kind-hearted_Elder", "花甲奶奶"),
        ("Chinese (Mandarin)_Cute_Spirit", "憨憨萌兽"),
        ("Chinese (Mandarin)_Radio_Host", "电台男主播"),
        ("Chinese (Mandarin)_Lyrical_Voice", "抒情男声"),
        ("Chinese (Mandarin)_Straightforward_Boy", "率真弟弟"),
        ("Chinese (Mandarin)_Sincere_Adult", "真诚青年"),
        ("Chinese (Mandarin)_Gentle_Senior", "温柔学姐"),
        ("Chinese (Mandarin)_Stubborn_Friend", "嘴硬竹马"),
        ("Chinese (Mandarin)_Crisp_Girl", "清脆少女"),
        ("Chinese (Mandarin)_Pure-hearted_Boy", "清澈邻家弟弟"),
        ("Chinese (Mandarin)_Soft_Girl", "柔和少女"),
    ]),
    ("中文（粤语）", &[
        ("Cantonese_ProfessionalHost（F)", "专业女主持"),
        ("Cantonese_GentleLady", "温柔女声"),
        ("Cantonese_ProfessionalHost（M)", "专业男主持"),
        ("Cantonese_PlayfulMan", "活泼男声"),
        ("Cantonese_CuteGirl", "可爱女孩"),
        ("Cantonese_KindWoman", "善良女声"),
    ]),
    ("英文", &[
        ("Santa_Claus", "Santa Claus"),
        ("Grinch", "Grinch"),
        ("Rudolph", "Rudolph"),
        ("Arnold", "Arnold"),
        ("Charming_Santa", "Charming Santa"),
        ("Charming_Lady", "Charming Lady"),
        ("Sweet_Girl", "Sweet Girl"),
        ("Cute_Elf", "Cute Elf"),
        ("Attractive_Girl", "Attractive Girl"),
        ("Serene_Woman", "Serene Woman"),
        ("English_Trustworthy_Man", "Trustworthy Man"),
        ("English_Graceful_Lady", "Graceful Lady"),
        ("English_Aussie_Bloke", "Aussie Bloke"),
        ("English_Whispering_girl", "Whispering girl"),
        ("English_Diligent_Man", "Diligent Man"),
        ("English_Gentle-voiced_man", "Gentle-voiced man"),
    ]),
    ("日文", &[
        ("Japanese_IntellectualSenior", "Intellectual Senior"),
        ("Japanese_DecisivePrincess", "Decisive Princess"),
        ("Japanese_LoyalKnight", "Loyal Knight"),
        ("Japanese_DominantMan", "Dominant Man"),
        ("Japanese_SeriousCommander", "Serious Commander"),
        ("Japanese_ColdQueen", "Cold Queen"),
        ("Japanese_DependableWoman", "Dependable Woman"),
        ("Japanese_GentleButler", "Gentle Butler"),
        ("Japanese_KindLady", "Kind Lady"),
        ("Japanese_CalmLady", "Calm Lady"),
        ("Japanese_OptimisticYouth", "Optimistic Youth"),
        ("Japanese_GenerousIzakayaOwner", "Generous Izakaya Owner"),
        ("Japanese_SportyStudent", "Sporty Student"),
        ("Japanese_InnocentBoy", "Innocent Boy"),
        ("Japanese_GracefulMaiden", "Graceful Maiden"),
    ]),
    ("韩文", &[
        ("Korean_SweetGirl", "Sweet Girl"),
        ("Korean_CheerfulBoyfriend", "Cheerful Boyfriend"),
        ("Korean_EnchantingSister", "Enchanting Sister"),
        ("Korean_ShyGirl", "Shy Girl"),
        ("Korean_ReliableSister", "Reliable Sister"),
        ("Korean_StrictBoss", "Strict Boss"),
        ("Korean_SassyGirl", "Sassy Girl"),
    ]),
];

/// 获取应用数据目录（统一的音频文件存储位置）
/// 优先使用 exe 安装目录下的 data 文件夹
/// 在 Windows 上通常为: 安装目录/data
/// 在 macOS 上通常为: 安装目录/data
/// 在 Linux 上通常为: 安装目录/data
pub fn get_app_data_dir() -> PathBuf {
    // 优先使用 exe 所在目录（打包后应该在这里）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let path = exe_dir.join("data");
            std::fs::create_dir_all(&path).ok();
            return path;
        }
    }
    
    // fallback: 使用当前目录的 data 文件夹
    PathBuf::from("data")
}

/// 获取音频文件存储目录（内部函数）
 fn get_audio_storage_dir_internal() -> PathBuf {
     let dir = get_app_data_dir().join("audio");
     std::fs::create_dir_all(&dir).ok();
     dir
 }

#[tauri::command]
pub fn get_voice_list() -> Value {
    let categories: Vec<Value> = VOICE_CATEGORIES.iter()
        .map(|(label, voices)| {
            json!({
                "label": label,
                "options": voices.iter()
                    .map(|(id, name)| json!({ "id": id, "name": name }))
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    
    json!({
        "success": true,
        "voice_categories": categories
    })
}

// 获取打包的 FFmpeg 路径（开箱即用支持）
pub fn get_bundled_ffmpeg_path() -> Option<PathBuf> {
    // 获取当前 exe 所在目录
    let exe_dir = match std::env::current_exe() {
        Ok(exe_path) => exe_path.parent()?.to_path_buf(),
        Err(_) => return None,
    };
    
    // 按优先级查找 FFmpeg 可能的位置
    // Tauri 2.0 resources 会放在 resources/ 目录下
    let possible_paths = vec![
        // Tauri 2.0 标准位置（resources 目录）
        exe_dir.join("resources/bin/ffmpeg.exe"),
        exe_dir.join("resources/ffmpeg.exe"),
        // 打包后直接放在应用目录下（便携模式）
        exe_dir.join("bin/ffmpeg.exe"),
        exe_dir.join("ffmpeg.exe"),
        // 开发环境：项目根目录的 bin 文件夹
        exe_dir.join("../../bin/ffmpeg.exe"),
        exe_dir.join("../bin/ffmpeg.exe"),
    ];
    
    for path in possible_paths {
        if path.exists() {
            log!("[FFmpeg] 找到 FFmpeg: {}", path.display());
            return Some(path);
        }
    }
    
    log!("[FFmpeg] 未找到打包的 FFmpeg，将使用系统 PATH");
    None
}

// 智能查找 FFmpeg 可执行文件路径
fn find_ffmpeg_path(db: &Database) -> String {
    // 1. 最高优先级：用户在设置中配置的自定义路径
    if let Ok(Some(path)) = db.get_config("video_ffmpeg_path") {
        if !path.is_empty() && std::path::Path::new(&path).exists() {
            return path;
        }
    }
    
    // 2. 使用打包在安装包中的 FFmpeg（开箱即用）
    if let Some(path) = get_bundled_ffmpeg_path() {
        return path.to_string_lossy().to_string();
    }
    
    // 3.  fallback: 尝试使用系统 PATH 中的 FFmpeg（用户自行安装的情况）
    "ffmpeg".to_string()
}

// 获取小说的音频列表
#[tauri::command]
pub async fn get_novel_audios(
    db: State<'_, Arc<Database>>,
    #[allow(unused_variables)] novel_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] novelId: Option<i64>,
) -> Result<Vec<ChapterAudio>, String> {
    let id = novel_id.or(novelId).unwrap_or(0);
    if id == 0 {
        return Err("缺少 novel_id 或 novelId 参数".to_string());
    }
    let audios = db
        .get_chapter_audios_by_novel(id)
        .map_err(|e: DbError| e.to_string())?;
    Ok(audios)
}

// 获取章节的音频详情
#[tauri::command]
pub async fn get_chapter_audio_detail(
    db: State<'_, Arc<Database>>,
    #[allow(unused_variables)] chapter_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] chapterId: Option<i64>,
) -> Result<Option<ChapterAudio>, String> {
    let id = chapter_id.or(chapterId).unwrap_or(0);
    if id == 0 {
        return Err("缺少 chapter_id 或 chapterId 参数".to_string());
    }
    let audio = db
        .get_chapter_audio_by_chapter(id)
        .map_err(|e: DbError| e.to_string())?;
    Ok(audio)
}

// 获取音频的所有段落（优先通过 chapter_id 获取）
#[tauri::command]
pub async fn get_audio_paragraphs(
    db: State<'_, Arc<Database>>,
    #[allow(unused_variables)] audio_id: Option<i64>,
    #[allow(unused_variables)] chapter_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] audioId: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] chapterId: Option<i64>,
) -> Result<Vec<ChapterParagraph>, String> {
    // 支持多种参数形式
    let aid = audio_id.or(audioId);
    let cid = chapter_id.or(chapterId);
    
    // 优先通过 chapter_id 获取段落
    if let Some(chap_id) = cid {
        let paragraphs = db
            .get_paragraphs_by_chapter_id(chap_id)
            .map_err(|e: DbError| e.to_string())?;
        return Ok(paragraphs);
    }
    
    // 其次通过 audio_id 获取段落
    if let Some(aud_id) = aid {
        let paragraphs = db
            .get_paragraphs_by_audio_id(aud_id)
            .map_err(|e: DbError| e.to_string())?;
        return Ok(paragraphs);
    }
    
    Err("需要提供 chapter_id 或 audio_id 参数".to_string())
}

// 查询音频生成状态
#[tauri::command]
pub async fn query_audio_status(
    db: State<'_, Arc<Database>>,
    task_id: String,
) -> Result<String, String> {
    let config_map = db.get_config_by_category("minimax")
        .map_err(|e: DbError| e.to_string())?;
    
    let api_key = config_map.get("minimax_api_key").cloned().ok_or("API 密钥未配置")?;
    let base_url = config_map.get("minimax_base_url").cloned().ok_or("API 基础地址未配置")?;
    let group_id = config_map.get("minimax_group_id").cloned().ok_or("Group ID 未配置")?;
    
    let config = MinMaxConfig {
        api_key,
        group_id: Some(group_id),
        base_url,
        default_model: "speech-01".to_string(),
        enabled: true,
    };
    
    let tts_client = MinMaxTTSClient::new(config);
    let status = tts_client.query_tts_status(&task_id).await?;
    
    Ok(status)
}

// 读取音频文件并返回字节数据
#[tauri::command]
pub async fn get_audio_stream(path: String) -> Result<Vec<u8>, String> {
    let path_obj = std::path::Path::new(&path);
    if !path_obj.exists() {
        return Err(format!("音频文件不存在: {}", path));
    }
    
    std::fs::read(&path)
        .map_err(|e| format!("读取音频文件失败: {} - {}", path, e))
}

// 生成章节音频
#[tauri::command]
pub async fn generate_chapter_audio(
    db: State<'_, Arc<Database>>,
    #[allow(unused_variables)] novel_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] novelId: Option<i64>,
    #[allow(unused_variables)] chapter_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] chapterId: Option<i64>,
    _chapter_title: String,
    _chapter_content: String,
    #[allow(unused_variables)] voice_id: Option<String>,
    #[allow(unused_variables)] #[allow(non_snake_case)] voiceId: Option<String>,
    speed: Option<f32>,
) -> Result<i64, String> {
    let n_id = novel_id.or(novelId).unwrap_or(0);
    if n_id == 0 {
        return Err("缺少 novel_id 或 novelId 参数".to_string());
    }
    let c_id = chapter_id.or(chapterId).unwrap_or(0);
    if c_id == 0 {
        return Err("缺少 chapter_id 或 chapterId 参数".to_string());
    }
    let v_id = voice_id.or(voiceId);
    
    // 检查是否已有音频
    if let Some(existing) = db
        .get_chapter_audio_by_chapter(c_id)
        .map_err(|e: DbError| e.to_string())?
    {
        // 如果存在，删除旧的
        db.delete_chapter_audio(existing.id)
            .map_err(|e: DbError| e.to_string())?;
    }

    // 创建新的音频记录
    let audio_id = db
        .create_chapter_audio(n_id, Some(c_id))
        .map_err(|e: DbError| e.to_string())?;

    // 获取该章节的手动标注段落
    let paragraphs = db.get_chapter_paragraphs(c_id).map_err(|e: DbError| e.to_string())?;
    
    if paragraphs.is_empty() {
        return Err("该章节暂无手动标注，请先在章节编辑中进行标注".to_string());
    }

    let total_paragraphs = paragraphs.len() as i32;

    // 更新状态为处理中
    db.update_chapter_audio_status(audio_id, "processing", Some(total_paragraphs), Some(0))
        .map_err(|e: DbError| e.to_string())?;

    // 获取角色列表，用于根据 character_id 获取 voice_id
    let characters = db.get_characters_by_novel_id(n_id).map_err(|e: DbError| e.to_string())?;
    let character_voice_map: std::collections::HashMap<i64, String> = characters
        .into_iter()
        .filter_map(|c| c.voice_id.map(|vid| (c.id, vid)))
        .collect();

    // 初始化段落的音频数据
    let default_voice = v_id.unwrap_or_else(|| "female-tianmei".to_string());
    let default_speed = speed.unwrap_or(1.0);
    
    db.init_paragraphs_audio(audio_id, c_id, &default_voice, default_speed)
        .map_err(|e: DbError| e.to_string())?;

    // 更新每个段落的 voice_id（根据绑定的角色）
    for para in paragraphs {
        if let Some(cid) = para.character_id {
            if let Some(vid) = character_voice_map.get(&cid) {
                let conn = db.get_conn();
                conn.execute(
                    "UPDATE chapter_paragraphs SET voice_id = ? WHERE id = ?",
                    (vid, para.id),
                ).map_err(DbError::from)?;
            }
        }
    }

    Ok(audio_id)
}

// 重新生成段落音频
#[tauri::command]
pub async fn regenerate_paragraph_audio(
    db: State<'_, Arc<Database>>,
    paragraph_id: i64,
    text: String,
    voice_id: String,
    speed: f32,
) -> Result<String, String> {
    log!("regenerate_paragraph_audio: 收到 paragraph_id = {}", paragraph_id);
    log!("regenerate_paragraph_audio: 收到 text = {}", text);
    log!("regenerate_paragraph_audio: 收到 voice_id = {}", voice_id);
    log!("regenerate_paragraph_audio: 收到 speed = {}", speed);
    let config_map = db
        .get_config_by_category("minimax")
        .map_err(|e: DbError| e.to_string())?;

    let api_key = config_map.get("minimax_api_key").cloned().unwrap_or_default();
    let base_url = config_map.get("minimax_base_url").cloned().unwrap_or_default();
    let group_id = config_map
        .get("minimax_group_id")
        .cloned()
        .filter(|s| !s.is_empty());

    let config = MinMaxConfig {
        api_key,
        group_id,
        base_url,
        default_model: "speech-01".to_string(),
        enabled: true,
    };

    // 调用 TTS API - 只创建任务，不等待完成
    let client = MinMaxTTSClient::new(config);
    
    let (task_id, task_token) = client.create_tts_task_only(&text, &voice_id, speed).await?;
    
    // 更新段落状态为 "processing"，表示正在生成，并保存 task_id 和 task_token
    db.update_paragraph_audio_result(
        paragraph_id,
        "processing",
        None,
        None,
        None,
        Some(&task_id),
        Some(&task_token),
    )
    .map_err(|e: DbError| e.to_string())?;

    Ok(task_id)
}

// 下载音频
#[tauri::command]
#[allow(dead_code)]
pub async fn download_paragraph_audio(
    db: State<'_, Arc<Database>>,
    paragraph_id: i64,
) -> Result<Vec<u8>, String> {
    // 获取段落信息
    let paragraph = db.get_paragraph_by_id(paragraph_id)
        .map_err(|e: DbError| e.to_string())?
        .ok_or("未找到该段落".to_string())?;
    
    // 检查是否有 task_id 和 task_token（用于下载音频）
    let (task_id, task_token) = match (&paragraph.task_id, &paragraph.task_token) {
        (Some(id), Some(token)) => (id, token),
        _ => return Err("该段落尚未生成完成，无法下载音频".to_string()),
    };
    
    // 获取配置
    let config_map = db.get_config_by_category("minimax")
        .map_err(|e: DbError| e.to_string())?;

    let api_key = config_map.get("minimax_api_key").cloned().unwrap_or_default();
    let base_url = config_map.get("minimax_base_url").cloned().unwrap_or_default();
    let group_id = config_map
        .get("minimax_group_id")
        .cloned()
        .filter(|s| !s.is_empty());

    let config = MinMaxConfig {
        api_key,
        group_id,
        base_url,
        default_model: "speech-01".to_string(),
        enabled: true,
    };

    let tts_client = MinMaxTTSClient::new(config);
    let audio_data = tts_client.download_audio(task_id, task_token).await?;
    
    // 保存音频到本地并更新段落状态
    // 使用统一的路径获取函数，确保在 dev 和打包模式下路径一致
    let app_dir = get_audio_storage_dir_internal();
    let file_name = format!("para_{}.mp3", paragraph_id);
    let file_path = app_dir.join(file_name);
    let file_path_str = file_path.to_string_lossy().to_string();
    
    log!("[download_paragraph_audio] 保存音频到: {}", file_path_str);

    std::fs::write(&file_path, &audio_data).map_err(|e: std::io::Error| e.to_string())?;

    // 更新段落状态
    db.update_paragraph_audio_result(
        paragraph_id,
        "completed",
        Some(&file_path_str),
        None,
        None,
        paragraph.task_id.as_deref(),
        paragraph.task_token.as_deref(),
    )
    .map_err(|e: DbError| e.to_string())?;
    
    Ok(audio_data)
}

// 更新段落音频参数（仅速度）
// 注意：voice_id 已移除，音色通过 character_id 从角色获取
#[tauri::command]
pub async fn update_paragraph_params(
    db: State<'_, Arc<Database>>,
    paragraph_id: i64,
    speed: Option<f32>,
) -> Result<(), String> {
    db.update_paragraph_audio_params(paragraph_id, speed)
        .map_err(|e: DbError| e.to_string())?;
    Ok(())
}

// 更新段落环境音
#[tauri::command]
pub async fn update_paragraph_ambient_sound(
    db: State<'_, Arc<Database>>,
    paragraph_id: i64,
    ambient_sound_id: Option<i64>,
) -> Result<(), String> {
    db.update_paragraph_ambient_sound(paragraph_id, ambient_sound_id)
        .map_err(|e: DbError| e.to_string())?;
    Ok(())
}

// 将环境音复制到段落
#[tauri::command]
pub async fn copy_ambient_to_paragraph(
    db: State<'_, Arc<Database>>,
    paragraph_id: i64,
    ambient_sound_id: i64,
) -> Result<String, String> {
    let ambient = db.get_ambient_sound_by_id(ambient_sound_id)
        .map_err(|e: DbError| e.to_string())?
        .ok_or("环境音不存在")?;
    
    let exe_path = std::env::current_exe()
        .map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent()
        .ok_or_else(|| "无法获取执行目录".to_string())?;
    let storage_dir = exe_dir.join("data").join("paragraphs");
    
    std::fs::create_dir_all(&storage_dir)
        .map_err(|e| e.to_string())?;
    
    let dest_path = storage_dir.join(format!("para_{}_{}.wav", paragraph_id, ambient_sound_id));
    
    std::fs::copy(&ambient.file_path, &dest_path)
        .map_err(|e| format!("复制文件失败: {}", e))?;
    
    db.update_paragraph_audio_path(paragraph_id, &dest_path.to_string_lossy())
        .map_err(|e: DbError| e.to_string())?;
    
    Ok(dest_path.to_string_lossy().to_string())
}

// 更新音频完成进度
#[tauri::command]
pub async fn update_audio_progress(
    db: State<'_, Arc<Database>>,
    audio_id: i64,
    completed: i32,
) -> Result<(), String> {
    db.update_chapter_audio_status(audio_id, "processing", None, Some(completed))
        .map_err(|e: DbError| e.to_string())?;
    Ok(())
}

// 标记音频完成
#[tauri::command]
pub async fn mark_audio_completed(
    db: State<'_, Arc<Database>>,
    audio_id: i64,
    merged_path: Option<String>,
) -> Result<(), String> {
    if let Some(path) = merged_path {
        db.update_merged_audio_path(audio_id, &path)
            .map_err(|e: DbError| e.to_string())?;
    } else {
        db.update_chapter_audio_status(audio_id, "completed", None, None)
            .map_err(|e: DbError| e.to_string())?;
    }
    Ok(())
}

// 删除音频
#[tauri::command]
pub async fn delete_chapter_audio(
    db: State<'_, Arc<Database>>,
    audio_id: i64,
) -> Result<(), String> {
    db.delete_chapter_audio(audio_id)
        .map_err(|e: DbError| e.to_string())?;
    Ok(())
}

// 获取音频存储目录
#[tauri::command]
pub fn get_audio_storage_dir() -> Result<String, String> {
    // 使用统一的路径获取函数
    let dir = get_audio_storage_dir_internal();
    Ok(dir.to_string_lossy().to_string())
}

// 合并章节音频
#[tauri::command]
pub async fn merge_chapter_audio(
    db: State<'_, Arc<Database>>,
    audio_id: i64,
    chapter_id: i64,
) -> Result<String, String> {
    // 获取所有已完成的段落音频
    let paragraphs = db.get_paragraphs_by_chapter_id(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    let completed_paragraphs: Vec<_> = paragraphs.iter()
        .filter(|p| p.status == "completed" && p.audio_path.is_some())
        .collect();
    
    if completed_paragraphs.is_empty() {
        return Err("没有已完成的音频段落可以合并".to_string());
    }
    
    // 获取音频存储目录
    let exe_path = std::env::current_exe()
        .map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent()
        .ok_or_else(|| "无法获取执行目录".to_string())?;
    let app_dir = exe_dir.join("data").join("audio");
    
    std::fs::create_dir_all(&app_dir).map_err(|e: std::io::Error| e.to_string())?;
    
    // 创建合并列表文件
    let list_file_path = app_dir.join(format!("merge_list_{}.txt", audio_id));
    let mut list_content = String::new();
    
    for para in &completed_paragraphs {
        if let Some(audio_path) = &para.audio_path {
            // 检查文件是否存在
            let path = std::path::Path::new(audio_path);
            if path.exists() {
                // 使用绝对路径并处理Windows路径
                // 在Windows上，使用display()来获取更友好的路径格式，避免\\?\前缀
                let path_str = if path.is_absolute() {
                    path.display().to_string()
                } else {
                    let abs_path = std::env::current_dir().map_err(|e| e.to_string())?.join(path);
                    abs_path.display().to_string()
                };
                // 对于ffmpeg，Windows路径需要特殊处理：替换反斜杠为正斜杠
                let escaped_path = path_str.replace("\\", "/");
                list_content.push_str(&format!("file '{}'\n", escaped_path));
            }
        }
    }
    
    if list_content.is_empty() {
        return Err("没有找到有效的音频文件".to_string());
    }
    
    std::fs::write(&list_file_path, list_content).map_err(|e: std::io::Error| e.to_string())?;
    
    // 合并后的音频路径
    let output_file_path = app_dir.join(format!("chapter_{}_audio.mp3", chapter_id));
    let output_path_str = output_file_path.to_string_lossy().to_string();
    
    // 智能查找 FFmpeg 路径（开箱即用支持）
    let ffmpeg_path = find_ffmpeg_path(&*db);
    
    // Windows 上使用不同的方法来避免 FFmpeg concat 协议的路径问题
    // 收集所有有效的音频文件路径
    let mut audio_files = Vec::new();
    for para in &completed_paragraphs {
        if let Some(audio_path) = &para.audio_path {
            let path = std::path::Path::new(audio_path);
            if path.exists() {
                audio_files.push(audio_path.clone());
            }
        }
    }
    
    if audio_files.is_empty() {
        return Err("没有找到有效的音频文件".to_string());
    }
    
    // 构建 FFmpeg 命令
    let mut cmd = std::process::Command::new(&ffmpeg_path);
    
    // 添加所有输入文件
    for path in &audio_files {
        cmd.arg("-i").arg(path);
    }
    
    // 动态构建 filter_complex 来连接音频
    let mut filter_input = String::new();
    for i in 0..audio_files.len() {
        filter_input.push_str(&format!("[{}:a]", i));
    }
    let filter = format!(
        "{}concat=n={}:v=0:a=1[out]",
        filter_input,
        audio_files.len()
    );
    
    // 完成命令构建
    cmd.arg("-filter_complex")
        .arg(filter)
        .arg("-map")
        .arg("[out]")
        .arg("-acodec")
        .arg("libmp3lame")
        .arg("-y")
        .arg(&output_path_str);
    
    let ffmpeg_result = cmd.output();
    
    match ffmpeg_result {
        Ok(output) => {
            if output.status.success() {
                // 删除列表文件
                let _ = std::fs::remove_file(&list_file_path);
                
                // 更新数据库中的合并音频路径
                db.update_merged_audio_path(audio_id, &output_path_str)
                    .map_err(|e: DbError| e.to_string())?;
                
                Ok(output_path_str)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("FFmpeg 合并失败: {}", stderr))
            }
        }
        Err(e) => {
            Err(format!("无法执行 FFmpeg: {}。请确保已安装 FFmpeg 并添加到系统 PATH 中。", e))
        }
    }
}
