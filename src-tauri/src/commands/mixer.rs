//! 混音相关命令模块
//! 支持段落级和章节级的音频与环境音混音

use crate::db::{models::*, Database};
use crate::db::DbError;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use std::process::Command;
use std::fs;
use serde_json::json;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::log;

/// 获取混音文件存储目录
fn get_mixer_storage_dir() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or_else(|| "无法获取执行目录".to_string())?
        .to_path_buf();
    
    let mixer_dir = exe_dir.join("data").join("mixer");
    fs::create_dir_all(&mixer_dir).map_err(|e| e.to_string())?;
    Ok(mixer_dir)
}

/// 获取 FFmpeg 路径
fn get_ffmpeg_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    
    let possible_paths = vec![
        exe_dir.clone().map(|p| p.join("resources/bin/ffmpeg.exe")),
        exe_dir.clone().map(|p| p.join("resources/ffmpeg.exe")),
        exe_dir.clone().map(|p| p.join("bin/ffmpeg.exe")),
        exe_dir.clone().map(|p| p.join("ffmpeg.exe")),
        Some(std::path::PathBuf::from("../../bin/ffmpeg.exe")),
        Some(std::path::PathBuf::from("../bin/ffmpeg.exe")),
    ].into_iter().flatten().collect::<Vec<_>>();
    
    for path in possible_paths {
        if path.exists() {
            log!("[混音] 找到 FFmpeg: {}", path.display());
            return path;
        }
    }
    
    PathBuf::from("ffmpeg")
}

/// 获取段落混音音频路径
fn get_paragraph_mixed_path(paragraph_id: i64) -> String {
    let mixer_dir = get_mixer_storage_dir().unwrap_or_else(|_| PathBuf::from("data/mixer"));
    let path = mixer_dir.join(format!("para_{}_mixed.mp3", paragraph_id));
    path.to_string_lossy().to_string()
}

/// 获取章节混音音频路径
fn get_chapter_mixed_path(chapter_id: i64) -> String {
    let mixer_dir = get_mixer_storage_dir().unwrap_or_else(|_| PathBuf::from("data/mixer"));
    let path = mixer_dir.join(format!("chapter_{}_mixed.mp3", chapter_id));
    path.to_string_lossy().to_string()
}

/// 检查音频文件是否存在
fn audio_file_exists(path: &Option<String>) -> bool {
    if let Some(p) = path {
        std::path::Path::new(p).exists()
    } else {
        false
    }
}

/// 获取有效的音频路径（优先混音路径，回退到原始路径）
fn get_valid_audio_path(paragraph: &ChapterParagraph) -> Option<String> {
    if let Some(ref mixed) = paragraph.mixed_audio_path {
        if std::path::Path::new(mixed).exists() {
            return Some(mixed.clone());
        }
    }
    
    if let Some(ref audio) = paragraph.audio_path {
        if std::path::Path::new(audio).exists() {
            return Some(audio.clone());
        }
    }
    
    None
}

// ==================== 段落环境音配置 ====================

/// 获取段落的混音配置
#[tauri::command]
pub async fn get_paragraph_ambient_configs(
    db: State<'_, Arc<Database>>,
    paragraph_id: i64,
) -> Result<Vec<ParagraphAmbientConfig>, String> {
    db.get_paragraph_ambient_configs(paragraph_id)
        .map_err(|e: DbError| e.to_string())
}

/// 保存段落环境音配置
#[tauri::command]
pub async fn save_paragraph_ambient_config(
    db: State<'_, Arc<Database>>,
    config: SaveParagraphAmbientConfigRequest,
) -> Result<ParagraphAmbientConfig, String> {
    db.save_paragraph_ambient_config(&config)
        .map_err(|e: DbError| e.to_string())
}

/// 批量保存段落环境音配置
#[tauri::command]
pub async fn batch_save_paragraph_ambient_configs(
    db: State<'_, Arc<Database>>,
    configs: BatchSaveParagraphAmbientConfigRequest,
) -> Result<Vec<ParagraphAmbientConfig>, String> {
    let mut results = Vec::new();
    
    for config in configs.configs {
        match db.save_paragraph_ambient_config(&config) {
            Ok(result) => results.push(result),
            Err(e) => {
                log!("[批量保存配置] 段落 {} 保存失败: {}", config.paragraph_id, e);
            }
        }
    }
    
    Ok(results)
}

/// 删除段落环境音配置
#[tauri::command]
pub async fn delete_paragraph_ambient_config(
    db: State<'_, Arc<Database>>,
    config_id: i64,
) -> Result<(), String> {
    db.delete_paragraph_ambient_config(config_id)
        .map_err(|e: DbError| e.to_string())
}

/// 获取章节的所有段落混音配置
#[tauri::command]
pub async fn get_chapter_paragraph_ambient_configs(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
) -> Result<Vec<(i64, Vec<ParagraphAmbientConfig>)>, String> {
    db.get_chapter_paragraph_ambient_configs(chapter_id)
        .map_err(|e: DbError| e.to_string())
}

// ==================== 混音预设 ====================

/// 获取混音预设列表
#[tauri::command]
pub async fn get_mix_presets(
    db: State<'_, Arc<Database>>,
    category: Option<String>,
) -> Result<Vec<AmbientMixPreset>, String> {
    db.get_mix_presets(category)
        .map_err(|e: DbError| e.to_string())
}

/// 保存混音预设
#[tauri::command]
pub async fn save_mix_preset(
    db: State<'_, Arc<Database>>,
    preset: SaveMixPresetRequest,
) -> Result<AmbientMixPreset, String> {
    db.save_mix_preset(&preset)
        .map_err(|e: DbError| e.to_string())
}

/// 删除混音预设
#[tauri::command]
pub async fn delete_mix_preset(
    db: State<'_, Arc<Database>>,
    preset_id: i64,
) -> Result<(), String> {
    db.delete_mix_preset(preset_id)
        .map_err(|e: DbError| e.to_string())
}

// ==================== 段落混音 ====================

/// 混音单个段落音频
#[tauri::command]
pub async fn mix_paragraph_audio(
    db: State<'_, Arc<Database>>,
    paragraph_id: i64,
    ambient_sound_id: Option<i64>,
    volume: Option<f32>,
    fade_in: Option<f32>,
    fade_out: Option<f32>,
) -> Result<ParagraphMixResult, String> {
    log!("[混音段落] 开始混音段落 {}", paragraph_id);
    
    // 获取段落信息
    let paragraph = db.get_paragraph_by_id(paragraph_id)
        .map_err(|e: DbError| e.to_string())?
        .ok_or_else(|| format!("段落 {} 不存在", paragraph_id))?;
    
    // 检查音频文件是否存在
    let voice_path = get_valid_audio_path(&paragraph)
        .ok_or_else(|| format!("段落 {} 没有有效的音频文件", paragraph_id))?;
    
    // 获取环境音路径
    let ambient_path = if let Some(ambient_id) = ambient_sound_id {
        let ambient = db.get_ambient_sound_by_id(ambient_id)
            .map_err(|e: DbError| e.to_string())?
            .ok_or_else(|| format!("环境音 {} 不存在", ambient_id))?;
        Some(ambient.file_path)
    } else {
        None
    };
    
    // 如果没有指定环境音，但段落有关联的环境音
    let ambient_path = if ambient_path.is_none() {
        paragraph.ambient_sound_id.and_then(|id| {
            db.get_ambient_sound_by_id(id).ok().flatten().map(|a| a.file_path)
        })
    } else {
        ambient_path
    };
    
    let vol = volume.unwrap_or(paragraph.ambient_volume);
    let fade_in_sec = fade_in.unwrap_or(paragraph.ambient_fade_in);
    let fade_out_sec = fade_out.unwrap_or(paragraph.ambient_fade_out);
    
    // 生成输出路径
    let output_path = get_paragraph_mixed_path(paragraph_id);
    
    // 先删除已存在的文件
    if std::path::Path::new(&output_path).exists() {
        log!("[混音段落] 删除已存在的文件: {}", output_path);
        let _ = fs::remove_file(&output_path);
    }
    
    let result = if let Some(ref ambient_file) = ambient_path {
        // 执行混音
        mix_audio_with_ambient_internal(&voice_path, ambient_file, &output_path, vol, fade_in_sec, fade_out_sec)?
    } else {
        // 没有环境音，直接复制音频文件
        fs::copy(&voice_path, &output_path)
            .map_err(|e| format!("复制音频文件失败: {}", e))?;
        output_path.clone()
    };
    
    // 更新数据库中的混音路径
    db.update_paragraph_mixed_path(paragraph_id, &result)
        .map_err(|e: DbError| e.to_string())?;
    
    // 同时更新段落的环境音关联（如果使用了环境音）
    let ambient_id_to_save = ambient_sound_id.or(paragraph.ambient_sound_id);
    if let Some(ambient_id) = ambient_id_to_save {
        log!("[混音段落] 更新段落 {} 的环境音关联: {}", paragraph_id, ambient_id);
        db.update_paragraph_ambient_sound(paragraph_id, Some(ambient_id))
            .map_err(|e: DbError| e.to_string())?;
    }
    
    log!("[混音段落] 段落 {} 混音完成: {}", paragraph_id, result);
    
    Ok(ParagraphMixResult {
        paragraph_id,
        mixed_audio_path: Some(result),
        success: true,
        error: None,
    })
}

/// 内部混音函数
fn mix_audio_with_ambient_internal(
    voice_path: &str,
    ambient_path: &str,
    output_path: &str,
    volume: f32,
    fade_in: f32,
    fade_out: f32,
) -> Result<String, String> {
    let ffmpeg_path = get_ffmpeg_path();
    
    log!("[混音] FFmpeg 路径: {}", ffmpeg_path.display());
    log!("[混音] 人声音频: {}", voice_path);
    log!("[混音] 环境音: {}", ambient_path);
    log!("[混音] 输出: {}", output_path);
    log!("[混音] 参数: volume={}, fade_in={}, fade_out={}", volume, fade_in, fade_out);
    
    // 构建 FFmpeg 命令
    // 环境音音量 + 淡入淡出 + 与人声混合
    let filter_complex = format!(
        "[1:a]volume={},afade=t=in:st=0:d={},afade=t=out:st=0:d={}[a_faded];\
         [0:a][a_faded]amix=inputs=2:duration=first:normalize=0[out]",
        volume, fade_in, fade_out
    );
    
    let mut cmd = Command::new(&ffmpeg_path);
    cmd.args([
        "-i", voice_path,
        "-i", ambient_path,
        "-filter_complex", &filter_complex,
        "-map", "[out]",
        "-y",
        output_path,
    ]);
    
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    
    // 直接使用路径，Command 会自动处理
    cmd.arg(output_path);
    
    let output = cmd.output()
        .map_err(|e| format!("FFmpeg 执行失败: {}", e))?;
    
    if output.status.success() {
        Ok(output_path.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("混音失败: {}", stderr))
    }
}

/// 批量混音段落
#[tauri::command]
pub async fn batch_mix_paragraphs(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
    ambient_sound_id: Option<i64>,
    volume: Option<f32>,
    fade_in: Option<f32>,
    fade_out: Option<f32>,
) -> Result<Vec<ParagraphMixResult>, String> {
    log!("[批量混音] 开始批量混音章节 {}", chapter_id);
    
    let paragraphs = db.get_chapter_paragraphs(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    let mut results = Vec::new();
    
    for para in paragraphs {
        if para.status == "completed" {
            match mix_paragraph_audio(
                db.clone(),
                para.id,
                ambient_sound_id,
                volume,
                fade_in,
                fade_out,
            ).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    log!("[批量混音] 段落 {} 混音失败: {}", para.id, e);
                    results.push(ParagraphMixResult {
                        paragraph_id: para.id,
                        mixed_audio_path: None,
                        success: false,
                        error: Some(e),
                    });
                }
            }
        }
    }
    
    log!("[批量混音] 完成，成功 {} 个，失败 {} 个", 
        results.iter().filter(|r| r.success).count(),
        results.iter().filter(|r| !r.success).count()
    );
    
    Ok(results)
}

// ==================== 章节混音 ====================

/// 混音章节音频（合并所有段落混音音频）
#[tauri::command]
pub async fn mix_chapter_audio(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
) -> Result<ChapterMixResult, String> {
    log!("[混音章节] 开始混音章节 {}", chapter_id);
    
    let paragraphs = db.get_chapter_paragraphs(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    let mut paragraph_results = Vec::new();
    let mut audio_files = Vec::new();
    
    // 收集有效的音频文件
    for para in paragraphs.iter().filter(|p| p.status == "completed") {
        if let Some(path) = get_valid_audio_path(para) {
            audio_files.push((para.id, para.paragraph_index, path));
        }
    }
    
    // 按段落顺序排序
    audio_files.sort_by_key(|k| k.1);
    
    // 生成 concat 列表文件
    let mixer_dir = get_mixer_storage_dir()?;
    let list_path = mixer_dir.join(format!("chapter_{}_list.txt", chapter_id));
    
    let list_content = audio_files.iter()
        .map(|(_, _, path)| format!("file '{}'", path))
        .collect::<Vec<_>>()
        .join("\n");
    
    fs::write(&list_path, list_content)
        .map_err(|e| format!("写入列表文件失败: {}", e))?;
    
    // 生成输出路径
    let output_path = get_chapter_mixed_path(chapter_id);
    
    // 执行 concat
    let ffmpeg_path = get_ffmpeg_path();
    
    let mut cmd = Command::new(&ffmpeg_path);
    cmd.args([
        "-f", "concat",
        "-safe", "0",
        "-i", list_path.to_str().unwrap(),
        "-c", "copy",
        "-y",
        &output_path,
    ]);
    
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    
    let output = cmd.output()
        .map_err(|e| format!("FFmpeg concat 执行失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("合并音频失败: {}", stderr));
    }
    
    // 清理列表文件
    let _ = fs::remove_file(&list_path);
    
    // 更新章节音频的混音路径
    let chapter_audio = db.get_chapter_audio_by_chapter(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    if let Some(audio) = chapter_audio {
        db.update_chapter_audio_mixed_path(audio.id, &output_path)
            .map_err(|e: DbError| e.to_string())?;
    }
    
    for (para_id, _, _) in &audio_files {
        paragraph_results.push(ParagraphMixResult {
            paragraph_id: *para_id,
            mixed_audio_path: Some(output_path.clone()),
            success: true,
            error: None,
        });
    }
    
    log!("[混音章节] 章节 {} 混音完成: {}", chapter_id, output_path);
    
    Ok(ChapterMixResult {
        chapter_id,
        mixed_audio_path: Some(output_path),
        paragraph_results,
        success: true,
        error: None,
    })
}

/// 全章统一环境音混音
#[tauri::command]
pub async fn mix_chapter_with_same_ambient(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
    ambient_sound_id: i64,
    volume: Option<f32>,
    fade_in: Option<f32>,
    fade_out: Option<f32>,
) -> Result<ChapterMixResult, String> {
    log!("[全章混音] 开始全章混音章节 {}，环境音 {}", chapter_id, ambient_sound_id);
    
    // 1. 先批量混音所有段落
    let paragraph_results = batch_mix_paragraphs(
        db.clone(),
        chapter_id,
        Some(ambient_sound_id),
        volume,
        fade_in,
        fade_out,
    ).await?;
    
    // 2. 再合并所有段落混音音频
    let chapter_result = mix_chapter_audio(db, chapter_id).await?;
    
    Ok(ChapterMixResult {
        chapter_id,
        mixed_audio_path: chapter_result.mixed_audio_path,
        paragraph_results,
        success: true,
        error: None,
    })
}

// ==================== 辅助命令 ====================

/// 获取章节混音状态
#[tauri::command]
pub async fn get_chapter_mix_status(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
) -> Result<serde_json::Value, String> {
    let paragraphs = db.get_chapter_paragraphs(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    let total = paragraphs.len();
    let completed = paragraphs.iter().filter(|p| p.status == "completed").count();
    let mixed = paragraphs.iter().filter(|p| p.mixed_audio_path.is_some() && audio_file_exists(&p.mixed_audio_path)).count();
    
    let chapter_audio = db.get_chapter_audio_by_chapter(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    let chapter_mixed = chapter_audio
        .as_ref()
        .and_then(|a| a.mixed_audio_path.as_ref())
        .map(|p| audio_file_exists(&Some(p.clone())))
        .unwrap_or(false);
    
    Ok(json!({
        "chapter_id": chapter_id,
        "total_paragraphs": total,
        "completed_paragraphs": completed,
        "mixed_paragraphs": mixed,
        "chapter_audio_mixed": chapter_mixed,
        "chapter_audio": chapter_audio,
    }))
}

/// 获取章节段落列表（用于混音界面）
#[tauri::command]
pub async fn get_chapter_paragraphs_for_mix(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
) -> Result<Vec<ParagraphListItem>, String> {
    log!("[混音界面] 获取章节 {} 的段落列表", chapter_id);
    
    let paragraphs = db.get_chapter_paragraphs(chapter_id)
        .map_err(|e: DbError| {
            log!("[混音界面] 获取段落失败: {}", e);
            e.to_string()
        })?;
    
    log!("[混音界面] 获取到 {} 个段落", paragraphs.len());
    
    let mut items = Vec::new();
    
    for para in paragraphs {
        log!("[混音界面] 处理段落 {}, index={}", para.id, para.paragraph_index);
        
        let character_name = if let Some(char_id) = para.character_id {
            log!("[混音界面] 段落 {} 有角色ID: {}", para.id, char_id);
            db.get_character_by_id(char_id)
                .map_err(|e: DbError| {
                    log!("[混音界面] 获取角色失败: {}", e);
                    e.to_string()
                })
                .ok()
                .flatten()
                .map(|c| {
                    log!("[混音界面] 找到角色: {}", c.name);
                    c.name
                })
        } else {
            log!("[混音界面] 段落 {} 没有角色ID", para.id);
            None
        };
        
        log!("[混音界面] 段落 {} 获取段落环境音配置", para.id);
        let ambient_config = db.get_paragraph_ambient_configs(para.id)
            .map_err(|e: DbError| {
                log!("[混音界面] 获取段落环境音配置失败: {}", e);
                e.to_string()
            })
            .ok()
            .and_then(|configs| {
                log!("[混音界面] 段落 {} 有 {} 个环境音配置", para.id, configs.len());
                configs.into_iter().next()
            });
        
        let ambient_sound = if let Some(ref config) = ambient_config {
            log!("[混音界面] 段落 {} 有环境音配置, ambient_id={:?}", para.id, config.ambient_sound_id);
            if let Some(ambient_id) = config.ambient_sound_id {
                db.get_ambient_sound_by_id(ambient_id)
                    .map_err(|e: DbError| {
                        log!("[混音界面] 获取环境音失败: {}", e);
                        e.to_string()
                    })
                    .ok()
                    .flatten()
            } else {
                log!("[混音界面] 段落 {} 环境音配置的 ambient_sound_id 为空", para.id);
                None
            }
        } else if para.ambient_sound_id.is_some() {
            log!("[混音界面] 段落 {} 使用段落自身的 ambient_sound_id", para.id);
            db.get_ambient_sound_by_id(para.ambient_sound_id.unwrap())
                .map_err(|e: DbError| {
                    log!("[混音界面] 获取段落环境音失败: {}", e);
                    e.to_string()
                })
                .ok()
                .flatten()
        } else {
            log!("[混音界面] 段落 {} 没有环境音", para.id);
            None
        };

        // 内容预览（前50字）- 使用 char_indices 确保不在中文字符中间切断
        let content_preview = if para.content.len() > 50 {
            let chars: Vec<char> = para.content.chars().collect();
            if chars.len() > 50 {
                chars[..50].iter().collect::<String>() + "..."
            } else {
                para.content.clone()
            }
        } else {
            para.content.clone()
        };
        
        log!("[混音界面] 段落 {} audio_path={:?}, mixed_audio_path={:?}, status={}", 
             para.id, para.audio_path, para.mixed_audio_path, para.status);

        items.push(ParagraphListItem {
            id: para.id,
            paragraph_index: para.paragraph_index,
            content: para.content.clone(),
            content_preview,
            r#type: para.r#type,
            character_id: para.character_id,
            character_name,
            audio_status: para.status.clone(),
            audio_path: para.audio_path,
            mixed_audio_path: para.mixed_audio_path,
            ambient_config,
            ambient_sound,
        });
    }
    
    log!("[混音界面] 共返回 {} 个段落", items.len());
    Ok(items)
}

/// 清除章节所有混音
#[tauri::command]
pub async fn clear_chapter_mix(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
) -> Result<(), String> {
    log!("[清除混音] 清除章节 {} 的所有混音", chapter_id);
    
    let paragraphs = db.get_chapter_paragraphs(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    for para in paragraphs {
        if let Some(ref mixed_path) = para.mixed_audio_path {
            if std::path::Path::new(mixed_path).exists() {
                let _ = fs::remove_file(mixed_path);
            }
        }
        db.update_paragraph_mixed_path(para.id, "")
            .map_err(|e: DbError| e.to_string())?;
    }
    
    let chapter_audio = db.get_chapter_audio_by_chapter(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    if let Some(audio) = chapter_audio {
        if let Some(ref mixed_path) = audio.mixed_audio_path {
            if std::path::Path::new(mixed_path).exists() {
                let _ = fs::remove_file(mixed_path);
            }
        }
        db.update_chapter_audio_mixed_path(audio.id, "")
            .map_err(|e: DbError| e.to_string())?;
    }
    
    log!("[清除混音] 完成");
    Ok(())
}
