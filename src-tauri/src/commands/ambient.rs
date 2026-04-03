//! 环境音相关 Tauri 命令

use crate::log;
use std::{sync::Arc, path::PathBuf, process::Command};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use tauri::State;

use crate::{
    db::{models::*, Database},
    db::DbError,
};

/// 获取打包的 Stable Audio 推理程序路径
pub fn get_stable_audio_exe_path() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()?
        .parent()?
        .to_path_buf();

    let possible_paths = vec![
        exe_dir.join("resources/stable_audio/stable_audio_inference.exe"),
        exe_dir.join("stable_audio/stable_audio_inference.exe"),
        exe_dir.join("../../stable_audio/stable_audio_inference.exe"),
        exe_dir.join("../../../stable_audio/stable_audio_inference.exe"),
    ];

    for path in possible_paths.iter() {
        if path.exists() {
            log!("[StableAudio] 找到推理程序: {}", path.display());
            return Some(path.clone());
        }
    }

    None
}

/// 获取 Stable Audio 模型文件路径
pub fn get_stable_audio_model_path() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or_else(|| "无法获取执行目录".to_string())?
        .to_path_buf();

    let possible_model_paths = vec![
        // 打包后的模型路径
        exe_dir.join("resources/stable_audio/model/f21265c1e2710b3bd2386596943f0007f55f802e_fp16"),
        exe_dir.join("stable_audio/model/f21265c1e2710b3bd2386596943f0007f55f802e_fp16"),
        // 开发模式路径
        exe_dir.join("../../stable_audio/model/f21265c1e2710b3bd2386596943f0007f55f802e_fp16"),
        exe_dir.join("../../../stable_audio/model/f21265c1e2710b3bd2386596943f0007f55f802e_fp16"),
    ];

    for path in possible_model_paths {
        if path.exists() {
            log!("[StableAudio] 找到模型目录: {}", path.display());
            return Ok(path);
        }
    }

    // 如果都不存在，返回默认路径
    let default_path = exe_dir.join("stable_audio/model/f21265c1e2710b3bd2386596943f0007f55f802e_fp16");
    std::fs::create_dir_all(&default_path).map_err(|e| e.to_string())?;
    Ok(default_path)
}

/// 检查模型文件是否存在
pub fn check_model_files() -> Result<(), String> {
    let model_path = get_stable_audio_model_path()?;
    
    // 检查必要的模型文件
    let required_files = vec![
        model_path.join("transformer/diffusion_pytorch_model.safetensors"),
        model_path.join("vae/diffusion_pytorch_model.safetensors"),
        model_path.join("text_encoder/model.safetensors"),
        model_path.join("tokenizer/tokenizer.json"),
        model_path.join("tokenizer/spiece.model"),
    ];

    let mut missing_files = Vec::new();
    for file_path in &required_files {
        if !file_path.exists() {
            missing_files.push(file_path.to_string_lossy().to_string());
        }
    }

    if !missing_files.is_empty() {
        Err(format!("缺少模型文件: {:?}", missing_files))
    } else {
        Ok(())
    }
}


// 获取脚本目录
#[allow(dead_code)]
fn get_scripts_dir() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or_else(|| "无法获取执行目录".to_string())?
        .to_path_buf();
    
    // 可能的脚本位置（相对于 exe 所在目录）
    let possible_paths = vec![
        // 打包后的 scripts 目录（NSIS 安装模式）
        exe_dir.join("resources/scripts/"),
        exe_dir.join("resources/scripts/"),
        // 新路径：scripts 与 src-tauri 同级（在 short_video 根目录）
        exe_dir.join("../../../scripts/"),  // src-tauri/target/debug/ -> short_video/scripts/
        exe_dir.join("../../scripts/"),     // src-tauri/target/ -> short_video/scripts/
        // 旧路径兼容：scripts 在 src-tauri 目录下
        exe_dir.join("../../scripts/"),
        exe_dir.join("../scripts/"),
        exe_dir.join("scripts/"),
        PathBuf::from("scripts/"),
    ];
    
    for path in possible_paths {
        if path.exists() {
            log!("[Scripts] 找到脚本目录: {}", path.display());
            return Ok(path);
        }
    }
    
    // 如果都不存在，返回默认路径
    let default_path = exe_dir.join("../../../scripts/");
    std::fs::create_dir_all(&default_path).map_err(|e| e.to_string())?;
    Ok(default_path)
}

// 获取环境音存储目录
fn get_ambient_storage_dir() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or_else(|| "无法获取执行目录".to_string())?
        .to_path_buf();
    
    let app_dir = exe_dir.join("data").join("ambient");

    std::fs::create_dir_all(&app_dir).map_err(|e: std::io::Error| e.to_string())?;

    Ok(app_dir)
}

// 获取环境音列表
#[tauri::command]
pub async fn get_ambient_sounds(
    db: State<'_, Arc<Database>>,
    category: Option<String>,
) -> Result<Vec<AmbientSound>, String> {
    let sounds = if let Some(ref cat) = category {
        if cat.is_empty() {
            db.get_all_ambient_sounds()
        } else {
            db.get_ambient_sounds_by_category(cat)
        }
    } else {
        db.get_all_ambient_sounds()
    }.map_err(|e: DbError| e.to_string())?;
    
    Ok(sounds)
}

// 获取章节环境音配置
#[tauri::command]
pub async fn get_chapter_ambient_config(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
) -> Result<Vec<ChapterAmbientConfig>, String> {
    let configs = db
        .get_chapter_ambient_configs(chapter_id)
        .map_err(|e: DbError| e.to_string())?;
    
    Ok(configs)
}

// 保存章节环境音配置
#[tauri::command]
pub async fn save_chapter_ambient_config(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
    ambient_sound_id: Option<i64>,
    volume: Option<f32>,
    fade_in: Option<f32>,
    fade_out: Option<f32>,
) -> Result<i64, String> {
    let config_id = db
        .save_chapter_ambient_config(chapter_id, ambient_sound_id, volume, fade_in, fade_out)
        .map_err(|e: DbError| e.to_string())?;
    
    Ok(config_id)
}

// 删除环境音配置
#[tauri::command]
pub async fn delete_chapter_ambient_config(
    db: State<'_, Arc<Database>>,
    config_id: i64,
) -> Result<(), String> {
    db
        .delete_chapter_ambient_config(config_id)
        .map_err(|e: DbError| e.to_string())?;
    
    Ok(())
}

// 生成环境音
#[tauri::command]
pub async fn generate_ambient_sound(
    db: State<'_, Arc<Database>>,
    prompt: String,
    duration: Option<u32>,
    name: Option<String>,
) -> Result<GenerateAmbientResponse, String> {
    let duration = duration.unwrap_or(10);
    let ambient_name = name.unwrap_or_else(|| format!("环境音_{}", chrono::Local::now().format("%Y%m%d_%H%M%S")));

    let output_dir = get_ambient_storage_dir()?;
    let file_name = format!("ambient_{}.wav", uuid::Uuid::new_v4());
    let output_path = output_dir.join(&file_name);

    let exe_path = get_stable_audio_exe_path()
        .ok_or_else(|| "未找到 Stable Audio 推理程序，请确保 stable_audio_inference.exe 存在".to_string())?;

    // 检查模型文件是否存在
    check_model_files()?;

    let mut command = Command::new(&exe_path);
    #[cfg(windows)]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    
    let output = command
        .args([
            "--prompt", &prompt,
            "--duration", &duration.to_string(),
            "--output", output_path.to_str().unwrap(),
        ])
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .map_err(|e| format!("推理程序执行失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    log!("[StableAudio] 输出: {}", stdout);

    if output.status.success() {
        let actual_duration = duration as f32;
        let ambient_id = db.create_ambient_sound(
            &ambient_name,
            Some(&prompt),
            "ai_generated",
            Some(&prompt),
            output_path.to_str().unwrap(),
            actual_duration,
            0.5,
            true,
        ).map_err(|e: DbError| e.to_string())?;

        Ok(GenerateAmbientResponse {
            success: true,
            file_path: Some(output_path.to_string_lossy().to_string()),
            error: None,
            duration: Some(actual_duration),
            ambient_id: Some(ambient_id),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("生成失败: {}", stderr))
    }
}


// 删除环境音（只删除记录，保留文件）
#[tauri::command]
pub async fn delete_ambient_sound(
    db: State<'_, Arc<Database>>,
    ambient_id: i64,
) -> Result<(), String> {
    // 检查环境音是否存在
    let _sound = db
        .get_ambient_sound_by_id(ambient_id)
        .map_err(|e: DbError| e.to_string())?
        .ok_or_else(|| "环境音不存在".to_string())?;
    
    // 只删除数据库记录，不删除文件
    db
        .delete_ambient_sound(ambient_id)
        .map_err(|e: DbError| e.to_string())?;
    
    Ok(())
}

// 混合人声与环境音
#[tauri::command]
pub async fn mix_voice_with_ambient(
    db: State<'_, Arc<Database>>,
    chapter_id: i64,
    ambient_sound_id: Option<i64>,
    ambient_file_path: Option<String>,
    volume: Option<f32>,
    fade_in: Option<f32>,
    fade_out: Option<f32>,
) -> Result<String, String> {
    // 获取章节目频
    let chapter_audio = db
        .get_chapter_audio_by_chapter(chapter_id)
        .map_err(|e: DbError| e.to_string())?
        .ok_or_else(|| "章节音频不存在，请先生成语音".to_string())?;
    
    let voice_path = chapter_audio.merged_audio_path
        .ok_or_else(|| "章节目频尚未合并".to_string())?;
    
    // 获取环境音路径
    let ambient_path = if let Some(ambient_id) = ambient_sound_id {
        db.get_ambient_sound_by_id(ambient_id)
            .map_err(|e: DbError| e.to_string())?
            .ok_or_else(|| "环境音不存在".to_string())?
            .file_path
    } else if let Some(path) = ambient_file_path {
        path
    } else {
        return Err("需要指定 ambient_sound_id 或 ambient_file_path".to_string());
    };
    
    let ambient_volume = volume.unwrap_or(0.3);
    let fade_in_sec = fade_in.unwrap_or(2.0);
    let fade_out_sec = fade_out.unwrap_or(2.0);
    
    // 输出路径
    let output_dir = get_ambient_storage_dir()?;
    let output_file = output_dir.join(format!("chapter_{}_with_ambient.mp3", chapter_id));
    
    // FFmpeg 混音命令
    let ffmpeg_path = super::audio::get_bundled_ffmpeg_path()
        .unwrap_or_else(|| PathBuf::from("ffmpeg"));
    
    // 复杂滤镜：循环环境音 + 淡入淡出 + 混音
    let filter_complex = format!(
        "[1:a]aloop=loop=-1:size=1e7[a_ambient];\
         [a_ambient]volume={}[a_vol];\
         [a_vol]afade=t=in:st=0:d={},afade=t=out:st=0:d={}[a_faded];\
         [0:a][a_faded]amix=inputs=2:duration=shortest:weights=1 {}[out]",
        ambient_volume, fade_in_sec, fade_out_sec, ambient_volume
    );
    
    let mut cmd = Command::new(&ffmpeg_path);
    cmd.args([
        "-i", &voice_path,
        "-i", &ambient_path,
        "-filter_complex", &filter_complex,
        "-map", "[out]",
        "-y",
        output_file.to_str().unwrap()
    ]);
    
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    
    let output = cmd.output().map_err(|e| format!("FFmpeg 执行失败: {}", e))?;
    
    if output.status.success() {
        Ok(output_file.to_string_lossy().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("混音失败: {}", stderr))
    }
}

// 使用 Stable Audio Open 生成环境音（AI 驱动的高质量生成）
#[tauri::command]
pub async fn generate_ambient_sound_stable_audio(
    db: State<'_, Arc<Database>>,
    prompt: String,
    duration: Option<u32>,
    name: Option<String>,
) -> Result<GenerateAmbientResponse, String> {
    let duration = duration.unwrap_or(10);
    let ambient_name = name.unwrap_or_else(|| format!("环境音_{}", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    
    let output_dir = get_ambient_storage_dir()?;
    let file_name = format!("ambient_{}.wav", uuid::Uuid::new_v4());
    let output_path = output_dir.join(&file_name);
    
    let exe_path = get_stable_audio_exe_path()
        .ok_or_else(|| "未找到 Stable Audio 推理程序，请确保 stable_audio_inference.exe 存在".to_string())?;
    
    // 检查模型文件是否存在
    check_model_files()?;
    
    let mut command = Command::new(&exe_path);
    #[cfg(windows)]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    
    let output = command
        .args([
            "--prompt", &prompt,
            "--duration", &duration.to_string(),
            "--output", output_path.to_str().unwrap(),
        ])
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .map_err(|e| format!("推理程序执行失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    log!("[Stable Audio] 输出: {}", stdout);
    
    if output.status.success() {
        let ambient_id = db.create_ambient_sound(
            &ambient_name,
            Some(&prompt),
            "ai_generated",
            Some(&prompt),
            output_path.to_str().unwrap(),
            duration as f32,
            0.5,
            true,
        ).map_err(|e: DbError| e.to_string())?;
        
        return Ok(GenerateAmbientResponse {
            success: true,
            file_path: Some(output_path.to_string_lossy().to_string()),
            error: None,
            duration: Some(duration as f32),
            ambient_id: Some(ambient_id),
        });
    }
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("生成失败: {}", stderr))
}
