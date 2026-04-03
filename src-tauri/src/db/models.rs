use serde::{Deserialize, Serialize};

// ==================== 基础数据模型 ====================

// 小说
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Novel {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub status: String,
    pub current_stage: String,
    pub total_chapters: i32,
    pub total_words: i32,
    pub created_at: String,
    pub updated_at: String,
}

// 章节
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub id: i64,
    pub novel_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub plain_text: Option<String>,
    pub order_num: i32,
    pub word_count: i32,
    pub status: String,
    pub created_at: String,
}

// 音频句子
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioSentence {
    pub id: i64,
    pub audio_id: Option<i64>,
    pub sentence_index: i32,
    pub text: Option<String>,
    pub voice_id: Option<String>,
    pub speed: f32,
    pub pitch: i32,
    pub volume: i32,
    pub emotion: Option<String>,
    pub audio_path: Option<String>,
    pub duration: Option<i32>,
    pub character_id: Option<i64>,
    pub is_dialogue: bool,
    pub status: String,
    pub error_msg: Option<String>,
    pub task_id: Option<String>,
    pub task_token: Option<String>,
    pub created_at: String,
}

// 音频项目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterAudio {
    pub id: i64,
    pub novel_id: i64,
    pub chapter_id: Option<i64>,
    pub status: String,
    pub total_sentences: i32,
    pub completed_sentences: i32,
    pub merged_audio_path: Option<String>,
    pub mixed_audio_path: Option<String>,  // 新增：章节混音后的音频路径
    pub created_at: String,
}

// 章节段落标注
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterParagraph {
    pub id: i64,
    pub chapter_id: i64,
    pub paragraph_index: i32,
    pub content: String,
    pub r#type: String,
    pub character_id: Option<i64>,
    pub audio_id: Option<i64>,
    pub voice_id: Option<String>,
    pub speed: f32,
    pub pitch: i32,
    pub volume: i32,
    pub emotion: String,
    pub audio_path: Option<String>,
    pub duration: Option<i32>,
    pub status: String,
    pub error_msg: Option<String>,
    pub task_id: Option<String>,
    pub task_token: Option<String>,
    pub ambient_sound_id: Option<i64>,
    pub ambient_volume: f32,
    pub ambient_fade_in: f32,
    pub ambient_fade_out: f32,
    pub mixed_audio_path: Option<String>,  // 新增：段落混音后的音频路径
    pub created_at: String,
    pub updated_at: String,
}

// 角色
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub id: i64,
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
    pub created_at: String,
}

// 角色形象图片
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterImage {
    pub id: i64,
    pub character_id: i64,
    pub image_type: Option<String>,
    pub pose: Option<String>,
    pub expression: Option<String>,
    pub image_path: String,
    pub prompt: Option<String>,
    pub seed: Option<i64>,
    pub is_default: bool,
    pub created_at: String,
}

// ==================== 请求/响应模型 ====================

// 创建小说请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNovelRequest {
    pub title: String,
    pub description: Option<String>,
}

// 更新小说请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNovelRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub cover_path: Option<String>,
    pub current_stage: Option<String>,
}

// 创建章节请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChapterRequest {
    #[serde(rename = "novelId")]
    pub novel_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub order_num: Option<i32>,
}

// 更新章节请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChapterRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub plain_text: Option<String>,
    pub order_num: Option<i32>,
}

// 批量保存段落请求（预留）
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchSaveParagraphsRequest {
    #[serde(rename = "chapterId")]
    pub chapter_id: i64,
    pub paragraphs: Vec<ParagraphMarkRequest>,
}

// 段落标注请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ParagraphMarkRequest {
    pub paragraph_index: i32,
    pub content: String,
    pub r#type: String,
    pub character_id: Option<i64>,
}

// 视频分镜（预留）
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoScene {
    pub id: String,
    pub video_id: i64,
    pub scene_order: i32,
    pub start_time: f32,
    pub duration: f32,
    pub scene_type: String,
    pub text_content: Option<String>,
    pub bg_type: Option<String>,
    pub bg_path: Option<String>,
    pub bg_prompt: Option<String>,
    pub image_path: Option<String>,
    pub status: String,
}

// 环境音素材
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AmbientSound {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub prompt: Option<String>,
    pub file_path: String,
    pub duration: f32,
    pub volume: f32,
    pub is_loopable: bool,
    pub is_system: bool,
    pub tags: Option<String>,
    pub created_at: String,
}

// 章节环境音配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterAmbientConfig {
    pub id: i64,
    pub chapter_id: i64,
    pub ambient_sound_id: Option<i64>,
    pub volume: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    pub start_paragraph: i32,
    pub end_paragraph: Option<i32>,
    pub created_at: String,
}

// 环境音生成请求（预留）
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateAmbientRequest {
    pub prompt: String,
    pub duration: Option<u32>,
    pub name: Option<String>,
}

// 环境音生成响应
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateAmbientResponse {
    pub success: bool,
    pub file_path: Option<String>,
    pub error: Option<String>,
    pub duration: Option<f32>,
    pub ambient_id: Option<i64>,
}

// 混音配置（预留）
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MixWithAmbientRequest {
    pub chapter_id: i64,
    pub ambient_sound_id: Option<i64>,
    pub ambient_file_path: Option<String>,
    pub volume: Option<f32>,
    pub fade_in: Option<f32>,
    pub fade_out: Option<f32>,
}

// ==================== 混音相关模型（新增） ====================

// 段落环境音配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParagraphAmbientConfig {
    pub id: i64,
    pub paragraph_id: i64,
    pub ambient_sound_id: Option<i64>,
    pub position_offset: f32,
    pub volume: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    pub fade_mode: String,
    pub is_muted: bool,
    pub created_at: String,
    pub updated_at: String,
}

// 混音预设模板
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AmbientMixPreset {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub preset_data: String,
    pub default_volume: f32,
    pub default_fade_in: f32,
    pub default_fade_out: f32,
    pub default_fade_mode: String,
    pub is_system: bool,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// 保存段落环境音配置请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveParagraphAmbientConfigRequest {
    pub paragraph_id: i64,
    pub ambient_sound_id: Option<i64>,
    pub position_offset: Option<f32>,
    pub volume: Option<f32>,
    pub fade_in: Option<f32>,
    pub fade_out: Option<f32>,
    pub fade_mode: Option<String>,
    pub is_muted: Option<bool>,
}

// 批量保存段落环境音配置请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchSaveParagraphAmbientConfigRequest {
    pub configs: Vec<SaveParagraphAmbientConfigRequest>,
}

// 段落混音请求
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MixParagraphRequest {
    pub paragraph_id: i64,
    pub ambient_sound_id: Option<i64>,
    pub volume: Option<f32>,
    pub fade_in: Option<f32>,
    pub fade_out: Option<f32>,
}

// 章节混音请求
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MixChapterRequest {
    pub chapter_id: i64,
    pub ambient_sound_id: Option<i64>,
    pub volume: Option<f32>,
    pub fade_in: Option<f32>,
    pub fade_out: Option<f32>,
    pub use_same_ambient: Option<bool>,  // 是否全章使用相同环境音
}

// 段落混音结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ParagraphMixResult {
    pub paragraph_id: i64,
    pub mixed_audio_path: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

// 章节混音结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterMixResult {
    pub chapter_id: i64,
    pub mixed_audio_path: Option<String>,
    pub paragraph_results: Vec<ParagraphMixResult>,
    pub success: bool,
    pub error: Option<String>,
}

// 保存混音预设请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveMixPresetRequest {
    pub name: String,
    pub description: Option<String>,
    pub preset_data: String,
    pub default_volume: Option<f32>,
    pub default_fade_in: Option<f32>,
    pub default_fade_out: Option<f32>,
    pub default_fade_mode: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
}

// 段落列表项（用于混音界面展示）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParagraphListItem {
    pub id: i64,
    pub paragraph_index: i32,
    pub content: String,
    pub content_preview: String,  // 内容预览（前50字）
    pub r#type: String,
    pub character_id: Option<i64>,
    pub character_name: Option<String>,
    pub audio_status: String,    // pending / completed / failed
    pub audio_path: Option<String>,
    pub mixed_audio_path: Option<String>,
    pub ambient_config: Option<ParagraphAmbientConfig>,
    pub ambient_sound: Option<AmbientSound>,
}
