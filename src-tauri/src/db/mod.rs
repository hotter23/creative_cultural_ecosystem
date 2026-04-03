use rusqlite::Connection;
use std::sync::Mutex;
use tauri::AppHandle;

pub mod models;
pub mod migrations;

use models::{ChapterAudio, AudioSentence, ChapterParagraph, ParagraphMarkRequest};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(_app_handle: &AppHandle) -> anyhow::Result<Self> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().expect("Failed to get executable directory");
        let app_dir = exe_dir.join("data");
        
        std::fs::create_dir_all(&app_dir)?;
        
        let db_path = app_dir.join("content_creator.db");
        let conn = Connection::open(db_path)?;
        
        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        // 运行迁移
        migrations::run_migrations(&conn)?;
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    pub fn get_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

// 数据库错误
#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl From<DbError> for String {
    fn from(err: DbError) -> Self {
        err.to_string()
    }
}

// 系统配置相关方法
impl Database {
    // 获取配置值
    #[allow(dead_code)]
    pub fn get_config(&self, key: &str) -> Result<Option<String>, DbError> {
        let conn = self.get_conn();
        let result = conn.query_row(
            "SELECT value FROM system_config WHERE key = ?",
            [key],
            |row| row.get(0),
        );
        
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Rusqlite(e)),
        }
    }
    
    // 获取分类下的所有配置
    pub fn get_config_by_category(&self, category: &str) -> Result<std::collections::HashMap<String, String>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            "SELECT key, value FROM system_config WHERE category = ?"
        )?;
        
        let rows = stmt.query_map([category], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        
        let mut config = std::collections::HashMap::new();
        for row in rows {
            let (key, value) = row?;
            config.insert(key, value);
        }
        
        Ok(config)
    }
    
    // 设置配置值
    #[allow(dead_code)]
    pub fn set_config(&self, key: &str, value: &str, category: Option<&str>) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            "INSERT OR REPLACE INTO system_config (key, value, category, updated_at) 
             VALUES (?, ?, COALESCE(?, (SELECT category FROM system_config WHERE key = ?), 'general'), CURRENT_TIMESTAMP)",
            [key, value, category.unwrap_or("general"), key],
        )?;
        Ok(())
    }
    
    // 批量设置配置
    pub fn set_config_batch(&self, configs: &[(&str, &str, &str)]) -> Result<(), DbError> {
        let mut conn = self.get_conn();
        let tx = conn.transaction()?;
        
        for (key, value, category) in configs {
            tx.execute(
                "INSERT OR REPLACE INTO system_config (key, value, category, updated_at) 
                 VALUES (?, ?, ?, CURRENT_TIMESTAMP)",
                [key, value, category],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    // ==================== 音频相关方法 ====================
    
    // 创建章节音频记录
    #[allow(dead_code)]
    pub fn create_chapter_audio(&self, novel_id: i64, chapter_id: Option<i64>) -> Result<i64, DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"INSERT INTO chapter_audios (novel_id, chapter_id, status, total_sentences, completed_sentences)
               VALUES (?1, ?2, 'pending', 0, 0)"#,
            (novel_id, chapter_id),
        )?;
        Ok(conn.last_insert_rowid())
    }

    // 更新章节音频状态
    #[allow(dead_code)]
    pub fn update_chapter_audio_status(&self, id: i64, status: &str, total_sentences: Option<i32>, completed_sentences: Option<i32>) -> Result<(), DbError> {
        let conn = self.get_conn();
        
        match (total_sentences, completed_sentences) {
            (Some(ts), Some(cs)) => {
                conn.execute(
                    "UPDATE chapter_audios SET status = ?1, total_sentences = ?2, completed_sentences = ?3 WHERE id = ?4",
                    (status, ts, cs, id),
                )?;
            }
            (Some(ts), None) => {
                conn.execute(
                    "UPDATE chapter_audios SET status = ?1, total_sentences = ?2 WHERE id = ?3",
                    (status, ts, id),
                )?;
            }
            (None, Some(cs)) => {
                conn.execute(
                    "UPDATE chapter_audios SET status = ?1, completed_sentences = ?2 WHERE id = ?3",
                    (status, cs, id),
                )?;
            }
            (None, None) => {
                conn.execute(
                    "UPDATE chapter_audios SET status = ?1 WHERE id = ?2",
                    (status, id),
                )?;
            }
        }
        
        Ok(())
    }

    // 更新合并后的音频路径
    #[allow(dead_code)]
    pub fn update_merged_audio_path(&self, id: i64, path: &str) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            "UPDATE chapter_audios SET merged_audio_path = ?1, status = 'completed' WHERE id = ?2",
            (path, id),
        )?;
        Ok(())
    }

    // 根据ID获取章节音频
    #[allow(dead_code)]
    pub fn get_chapter_audio_by_id(&self, id: i64) -> Result<Option<ChapterAudio>, DbError> {
        let conn = self.get_conn();
        let result = conn.query_row(
            r#"SELECT id, novel_id, chapter_id, status, total_sentences, completed_sentences, merged_audio_path, mixed_audio_path, created_at
               FROM chapter_audios WHERE id = ?"#,
            [id],
            |row| {
                Ok(ChapterAudio {
                    id: row.get(0)?,
                    novel_id: row.get(1)?,
                    chapter_id: row.get(2)?,
                    status: row.get(3)?,
                    total_sentences: row.get(4)?,
                    completed_sentences: row.get(5)?,
                    merged_audio_path: row.get(6)?,
                    mixed_audio_path: row.get(7)?,
                    created_at: row.get(8)?,
                })
            },
        );
        
        match result {
            Ok(audio) => Ok(Some(audio)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Rusqlite(e)),
        }
    }

    // 根据章节ID获取章节音频
    #[allow(dead_code)]
    pub fn get_chapter_audio_by_chapter(&self, chapter_id: i64) -> Result<Option<ChapterAudio>, DbError> {
        let conn = self.get_conn();
        let result = conn.query_row(
            r#"SELECT id, novel_id, chapter_id, status, total_sentences, completed_sentences, merged_audio_path, mixed_audio_path, created_at
               FROM chapter_audios WHERE chapter_id = ?"#,
            [chapter_id],
            |row| {
                Ok(ChapterAudio {
                    id: row.get(0)?,
                    novel_id: row.get(1)?,
                    chapter_id: row.get(2)?,
                    status: row.get(3)?,
                    total_sentences: row.get(4)?,
                    completed_sentences: row.get(5)?,
                    merged_audio_path: row.get(6)?,
                    mixed_audio_path: row.get(7)?,
                    created_at: row.get(8)?,
                })
            },
        );
        
        match result {
            Ok(audio) => Ok(Some(audio)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Rusqlite(e)),
        }
    }

    // 获取小说的所有音频
    #[allow(dead_code)]
    pub fn get_chapter_audios_by_novel(&self, novel_id: i64) -> Result<Vec<ChapterAudio>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, novel_id, chapter_id, status, total_sentences, completed_sentences, merged_audio_path, mixed_audio_path, created_at
               FROM chapter_audios WHERE novel_id = ? ORDER BY created_at DESC"#,
        )?;
        
        let rows = stmt.query_map([novel_id], |row| {
            Ok(ChapterAudio {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                chapter_id: row.get(2)?,
                status: row.get(3)?,
                total_sentences: row.get(4)?,
                completed_sentences: row.get(5)?,
                merged_audio_path: row.get(6)?,
                mixed_audio_path: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        
        let mut audios = Vec::new();
        for row in rows {
            audios.push(row?);
        }
        
        Ok(audios)
    }

    // 更新章节音频混音路径
    pub fn update_chapter_audio_mixed_path(&self, audio_id: i64, mixed_path: &str) -> Result<(), DbError> {
        let conn = self.get_conn();
        
        if mixed_path.is_empty() {
            conn.execute(
                r#"UPDATE chapter_audios SET mixed_audio_path = NULL WHERE id = ?"#,
                [audio_id],
            )?;
        } else {
            conn.execute(
                r#"UPDATE chapter_audios SET mixed_audio_path = ? WHERE id = ?"#,
                (mixed_path, audio_id),
            )?;
        }
        
        Ok(())
    }

    // 创建音频句子
    #[allow(dead_code)]
    pub fn create_audio_sentence(&self, sentence: &AudioSentence) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"INSERT INTO audio_sentences 
               (id, audio_id, sentence_index, text, voice_id, speed, pitch, volume, emotion, audio_path, duration, character_id, is_dialogue, status)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'pending')"#,
            (
                &sentence.id,
                sentence.audio_id,
                sentence.sentence_index,
                &sentence.text,
                &sentence.voice_id,
                sentence.speed,
                sentence.pitch,
                sentence.volume,
                &sentence.emotion,
                &sentence.audio_path,
                sentence.duration,
                sentence.character_id,
                sentence.is_dialogue,
            ),
        )?;
        Ok(())
    }

    // 批量创建音频句子
    #[allow(dead_code)]
    pub fn create_audio_sentences_batch(&self, sentences: &[AudioSentence]) -> Result<(), DbError> {
        let mut conn = self.get_conn();
        let tx = conn.transaction()?;
        
        for sentence in sentences {
            tx.execute(
                r#"INSERT INTO audio_sentences 
                   (id, audio_id, sentence_index, text, voice_id, speed, pitch, volume, emotion, audio_path, duration, character_id, is_dialogue, status, task_id, task_token)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'pending', ?14, ?15)"#,
                (
                    &sentence.id,
                    sentence.audio_id,
                    sentence.sentence_index,
                    &sentence.text,
                    &sentence.voice_id,
                    sentence.speed,
                    sentence.pitch,
                    sentence.volume,
                    &sentence.emotion,
                    &sentence.audio_path,
                    sentence.duration,
                    sentence.character_id,
                    sentence.is_dialogue,
                    &sentence.task_id,
                    &sentence.task_token,
                ),
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    // 根据句子ID获取单个音频句子
    #[allow(dead_code)]
    pub fn get_audio_sentence(&self, id: &str) -> Result<AudioSentence, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, audio_id, sentence_index, text, voice_id, speed, pitch, volume, emotion, audio_path, duration, character_id, is_dialogue, status, error_msg, task_id, task_token, created_at
               FROM audio_sentences WHERE id = ?"#,
        )?;
        
        let row = stmt.query_row([id], |row| {
            Ok(AudioSentence {
                id: row.get(0)?,
                audio_id: row.get(1)?,
                sentence_index: row.get(2)?,
                text: row.get(3)?,
                voice_id: row.get(4)?,
                speed: row.get(5)?,
                pitch: row.get(6)?,
                volume: row.get(7)?,
                emotion: row.get(8)?,
                audio_path: row.get(9)?,
                duration: row.get(10)?,
                character_id: row.get(11)?,
                is_dialogue: row.get(12)?,
                status: row.get(13)?,
                error_msg: row.get(14)?,
                task_id: row.get(15)?,
                task_token: row.get(16)?,
                created_at: row.get(17)?,
            })
        })?;
        
        Ok(row)
    }

    // 根据 task_id 获取音频句子
    #[allow(dead_code)]
    pub fn get_audio_sentences_by_task_id(&self, task_id: &str) -> Result<Vec<AudioSentence>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, audio_id, sentence_index, text, voice_id, speed, pitch, volume, emotion, audio_path, duration, character_id, is_dialogue, status, error_msg, task_id, task_token, created_at
               FROM audio_sentences WHERE task_id = ?"#,
        )?;
        
        let rows = stmt.query_map([task_id], |row| {
            Ok(AudioSentence {
                id: row.get(0)?,
                audio_id: row.get(1)?,
                sentence_index: row.get(2)?,
                text: row.get(3)?,
                voice_id: row.get(4)?,
                speed: row.get(5)?,
                pitch: row.get(6)?,
                volume: row.get(7)?,
                emotion: row.get(8)?,
                audio_path: row.get(9)?,
                duration: row.get(10)?,
                character_id: row.get(11)?,
                is_dialogue: row.get(12)?,
                status: row.get(13)?,
                error_msg: row.get(14)?,
                task_id: row.get(15)?,
                task_token: row.get(16)?,
                created_at: row.get(17)?,
            })
        })?;
        
        let mut sentences = Vec::new();
        for row in rows {
            sentences.push(row?);
        }
        
        Ok(sentences)
    }

    // 获取音频的所有句子
    #[allow(dead_code)]
    pub fn get_audio_sentences_by_audio_id(&self, audio_id: i64) -> Result<Vec<AudioSentence>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, audio_id, sentence_index, text, voice_id, speed, pitch, volume, emotion, audio_path, duration, character_id, is_dialogue, status, error_msg, task_id, task_token, created_at
               FROM audio_sentences WHERE audio_id = ? ORDER BY sentence_index ASC"#,
        )?;
        
        let rows = stmt.query_map([audio_id], |row| {
            Ok(AudioSentence {
                id: row.get(0)?,
                audio_id: row.get(1)?,
                sentence_index: row.get(2)?,
                text: row.get(3)?,
                voice_id: row.get(4)?,
                speed: row.get(5)?,
                pitch: row.get(6)?,
                volume: row.get(7)?,
                emotion: row.get(8)?,
                audio_path: row.get(9)?,
                duration: row.get(10)?,
                character_id: row.get(11)?,
                is_dialogue: row.get(12)?,
                status: row.get(13)?,
                error_msg: row.get(14)?,
                task_id: row.get(15)?,
                task_token: row.get(16)?,
                created_at: row.get(17)?,
            })
        })?;
        
        let mut sentences = Vec::new();
        for row in rows {
            sentences.push(row?);
        }
        
        Ok(sentences)
    }

    // 更新音频句子状态和路径
    #[allow(dead_code)]
    pub fn update_audio_sentence_result(&self, id: &str, status: &str, audio_path: Option<&str>, duration: Option<i32>, error_msg: Option<&str>, task_id: Option<&str>, task_token: Option<&str>) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"UPDATE audio_sentences 
               SET status = ?1, audio_path = ?2, duration = ?3, error_msg = ?4, task_id = ?5, task_token = ?6
               WHERE id = ?7"#,
            (status, audio_path, duration, error_msg, task_id, task_token, id),
        )?;
        
        // 更新音频进度
        // 获取该句子所属的音频ID
        let audio_id: i64 = conn.query_row(
            "SELECT audio_id FROM audio_sentences WHERE id = ?",
            [id],
            |row| row.get(0),
        )?;
        
        // 计算完成的句子数和总句子数
        let completed_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audio_sentences WHERE audio_id = ? AND status = 'completed'",
            [audio_id],
            |row| row.get(0),
        )?;
        
        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audio_sentences WHERE audio_id = ?",
            [audio_id],
            |row| row.get(0),
        )?;
        
        // 更新 chapter_audios 表
        conn.execute(
            "UPDATE chapter_audios SET completed_sentences = ?, total_sentences = ? WHERE id = ?",
            (completed_count, total_count, audio_id),
        )?;
        
        Ok(())
    }

    // 更新音频句子参数（音色、语速）
    #[allow(dead_code)]
    pub fn update_audio_sentence_params(&self, id: &str, voice_id: Option<String>, speed: Option<f32>) -> Result<(), DbError> {
        let conn = self.get_conn();
        
        match (voice_id, speed) {
            (Some(vid), Some(spd)) => {
                conn.execute(
                    r#"UPDATE audio_sentences 
                       SET voice_id = ?1, speed = ?2, status = 'pending', error_msg = NULL
                       WHERE id = ?3"#,
                    (vid, spd, id),
                )?;
            }
            (Some(vid), None) => {
                conn.execute(
                    r#"UPDATE audio_sentences 
                       SET voice_id = ?1, status = 'pending', error_msg = NULL
                       WHERE id = ?2"#,
                    (vid, id),
                )?;
            }
            (None, Some(spd)) => {
                conn.execute(
                    r#"UPDATE audio_sentences 
                       SET speed = ?1, status = 'pending', error_msg = NULL
                       WHERE id = ?2"#,
                    (spd, id),
                )?;
            }
            (None, None) => {}
        }
        
        Ok(())
    }

    // 删除章节音频及其句子
    #[allow(dead_code)]
    pub fn delete_chapter_audio(&self, id: i64) -> Result<(), DbError> {
        let conn = self.get_conn();
        // 由于外键设置了 ON DELETE CASCADE，删除主记录会自动删除句子
        conn.execute("DELETE FROM chapter_audios WHERE id = ?", [id])?;
        Ok(())
    }
}

// 角色相关数据库操作
use crate::db::models::{Character, CharacterImage};

impl Database {
    // 创建角色
    #[allow(dead_code)]
    pub fn create_character(&self, character: &Character) -> Result<i64, DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"INSERT INTO characters 
               (novel_id, name, aliases, gender, role, description, appearance, personality, voice_id, tags) 
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            (
                character.novel_id,
                &character.name,
                &character.aliases,
                &character.gender,
                &character.role,
                &character.description,
                &character.appearance,
                &character.personality,
                &character.voice_id,
                &character.tags,
            ),
        )?;
        Ok(conn.last_insert_rowid())
    }

    // 更新角色
    #[allow(dead_code)]
    pub fn update_character(&self, id: i64, character: &Character) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"UPDATE characters 
               SET name = ?1, aliases = ?2, gender = ?3, role = ?4, description = ?5, 
                   appearance = ?6, personality = ?7, voice_id = ?8, tags = ?9
               WHERE id = ?10"#,
            (
                &character.name,
                &character.aliases,
                &character.gender,
                &character.role,
                &character.description,
                &character.appearance,
                &character.personality,
                &character.voice_id,
                &character.tags,
                id,
            ),
        )?;
        Ok(())
    }

    // 删除角色
    #[allow(dead_code)]
    pub fn delete_character(&self, id: i64) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute("DELETE FROM characters WHERE id = ?", [id])?;
        Ok(())
    }

    // 根据ID获取角色
    #[allow(dead_code)]
    pub fn get_character_by_id(&self, id: i64) -> Result<Option<Character>, DbError> {
        let conn = self.get_conn();
        let result = conn.query_row(
            r#"SELECT id, novel_id, name, aliases, gender, role, description, 
                      appearance, personality, voice_id, tags, created_at 
               FROM characters WHERE id = ?"#,
            [id],
            |row| {
                Ok(Character {
                    id: row.get(0)?,
                    novel_id: row.get(1)?,
                    name: row.get(2)?,
                    aliases: row.get(3)?,
                    gender: row.get(4)?,
                    role: row.get(5)?,
                    description: row.get(6)?,
                    appearance: row.get(7)?,
                    personality: row.get(8)?,
                    voice_id: row.get(9)?,
                    tags: row.get(10)?,
                    created_at: row.get(11)?,
                })
            },
        );
        
        match result {
            Ok(character) => Ok(Some(character)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Rusqlite(e)),
        }
    }

    // 获取小说的所有角色
    #[allow(dead_code)]
    pub fn get_characters_by_novel_id(&self, novel_id: i64) -> Result<Vec<Character>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, novel_id, name, aliases, gender, role, description, 
                      appearance, personality, voice_id, tags, created_at 
               FROM characters WHERE novel_id = ? ORDER BY created_at DESC"#,
        )?;
        
        let rows = stmt.query_map([novel_id], |row| {
            Ok(Character {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                name: row.get(2)?,
                aliases: row.get(3)?,
                gender: row.get(4)?,
                role: row.get(5)?,
                description: row.get(6)?,
                appearance: row.get(7)?,
                personality: row.get(8)?,
                voice_id: row.get(9)?,
                tags: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        
        let mut characters = Vec::new();
        for row in rows {
            characters.push(row?);
        }
        
        Ok(characters)
    }

    // 获取角色的所有形象图片
    #[allow(dead_code)]
    pub fn get_character_images(&self, character_id: i64) -> Result<Vec<CharacterImage>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, character_id, image_type, pose, expression, image_path, 
                      prompt, seed, is_default, created_at 
               FROM character_images WHERE character_id = ? ORDER BY created_at DESC"#,
        )?;
        
        let rows = stmt.query_map([character_id], |row| {
            Ok(CharacterImage {
                id: row.get(0)?,
                character_id: row.get(1)?,
                image_type: row.get(2)?,
                pose: row.get(3)?,
                expression: row.get(4)?,
                image_path: row.get(5)?,
                prompt: row.get(6)?,
                seed: row.get(7)?,
                is_default: {
                    let val: i32 = row.get(8)?;
                    val != 0
                },
                created_at: row.get(9)?,
            })
        })?;
        
        let mut images = Vec::new();
        for row in rows {
            images.push(row?);
        }
        
        Ok(images)
    }

    // 创建角色形象
    #[allow(dead_code)]
    pub fn create_character_image(&self, image: &CharacterImage) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"INSERT INTO character_images 
               (id, character_id, image_type, pose, expression, image_path, prompt, seed, is_default) 
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            (
                &image.id,
                image.character_id,
                &image.image_type,
                &image.pose,
                &image.expression,
                &image.image_path,
                &image.prompt,
                &image.seed,
                if image.is_default { 1 } else { 0 },
            ),
        )?;
        Ok(())
    }

    // 删除角色形象
    #[allow(dead_code)]
    pub fn delete_character_image(&self, id: &str) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute("DELETE FROM character_images WHERE id = ?", [id])?;
        Ok(())
    }

    // 设置角色默认形象
    #[allow(dead_code)]
    pub fn set_default_character_image(&self, character_id: i64, image_id: &str) -> Result<(), DbError> {
        let mut conn = self.get_conn();
        let tx = conn.transaction()?;
        
        // 取消之前的默认设置
        tx.execute(
            "UPDATE character_images SET is_default = 0 WHERE character_id = ?",
            [character_id],
        )?;
        
        // 设置新的默认
        tx.execute(
            "UPDATE character_images SET is_default = 1 WHERE character_id = ? AND id = ?",
            (character_id, image_id),
        )?;
        
        tx.commit()?;
        Ok(())
    }

    // 绑定角色音色
    #[allow(dead_code)]
    pub fn bind_character_voice(&self, character_id: i64, voice_id: &str) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            "UPDATE characters SET voice_id = ? WHERE id = ?",
            (voice_id, character_id),
        )?;
        Ok(())
    }

    // 列出角色图片（重命名以符合MCP工具调用）
    #[allow(dead_code)]
    pub fn list_character_images(&self, character_id: i64) -> Result<Vec<CharacterImage>, DbError> {
        self.get_character_images(character_id)
    }

    // 批量保存章节段落标注
    #[allow(dead_code)]
    pub fn save_chapter_paragraphs(&self, chapter_id: i64, paragraphs: Vec<ParagraphMarkRequest>) -> Result<(), DbError> {
        let mut conn = self.get_conn();
        let tx = conn.transaction()?;
        
        // 先删除该章节的所有现有标注
        tx.execute(
            "DELETE FROM chapter_paragraphs WHERE chapter_id = ?",
            [chapter_id],
        )?;
        
        // 保存新标注
        for para in paragraphs {
            tx.execute(
                "INSERT INTO chapter_paragraphs (chapter_id, paragraph_index, content, type, character_id) 
                 VALUES (?, ?, ?, ?, ?)",
                (
                    chapter_id,
                    para.paragraph_index,
                    para.content,
                    para.r#type,
                    para.character_id,
                ),
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    // 获取章节的所有段落标注
    #[allow(dead_code)]
    pub fn get_chapter_paragraphs(&self, chapter_id: i64) -> Result<Vec<ChapterParagraph>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, chapter_id, paragraph_index, content, type, character_id, 
                    audio_id, voice_id, speed, pitch, volume, emotion, audio_path, duration, 
                    status, error_msg, task_id, task_token, ambient_sound_id, 
                    ambient_volume, ambient_fade_in, ambient_fade_out, mixed_audio_path,
                    created_at, updated_at
             FROM chapter_paragraphs
             WHERE chapter_id = ?
             ORDER BY paragraph_index ASC"
        )?;

        let rows = stmt.query_map([chapter_id], |row| {
            Ok(ChapterParagraph {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                paragraph_index: row.get(2)?,
                content: row.get(3)?,
                r#type: row.get(4)?,
                character_id: row.get(5)?,
                audio_id: row.get(6)?,
                voice_id: row.get(7)?,
                speed: row.get(8)?,
                pitch: row.get(9)?,
                volume: row.get(10)?,
                emotion: row.get(11)?,
                audio_path: row.get(12)?,
                duration: row.get(13)?,
                status: row.get(14)?,
                error_msg: row.get(15)?,
                task_id: row.get(16)?,
                task_token: row.get(17)?,
                ambient_sound_id: row.get(18)?,
                ambient_volume: row.get(19)?,
                ambient_fade_in: row.get(20)?,
                ambient_fade_out: row.get(21)?,
                mixed_audio_path: row.get(22)?,
                created_at: row.get(23)?,
                updated_at: row.get(24)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    // 根据音频ID获取段落
    #[allow(dead_code)]
    pub fn get_paragraphs_by_audio_id(&self, audio_id: i64) -> Result<Vec<ChapterParagraph>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, chapter_id, paragraph_index, content, type, character_id,
                    audio_id, voice_id, speed, pitch, volume, emotion, audio_path, duration,
                    status, error_msg, task_id, task_token, ambient_sound_id,
                    ambient_volume, ambient_fade_in, ambient_fade_out, mixed_audio_path,
                    created_at, updated_at
             FROM chapter_paragraphs
             WHERE audio_id = ?
             ORDER BY paragraph_index ASC"
        )?;

        let rows = stmt.query_map([audio_id], |row| {
            Ok(ChapterParagraph {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                paragraph_index: row.get(2)?,
                content: row.get(3)?,
                r#type: row.get(4)?,
                character_id: row.get(5)?,
                audio_id: row.get(6)?,
                voice_id: row.get(7)?,
                speed: row.get(8)?,
                pitch: row.get(9)?,
                volume: row.get(10)?,
                emotion: row.get(11)?,
                audio_path: row.get(12)?,
                duration: row.get(13)?,
                status: row.get(14)?,
                error_msg: row.get(15)?,
                task_id: row.get(16)?,
                task_token: row.get(17)?,
                ambient_sound_id: row.get(18)?,
                ambient_volume: row.get(19)?,
                ambient_fade_in: row.get(20)?,
                ambient_fade_out: row.get(21)?,
                mixed_audio_path: row.get(22)?,
                created_at: row.get(23)?,
                updated_at: row.get(24)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    // 通过 chapter_id 获取段落列表（用于音频制作）
    pub fn get_paragraphs_by_chapter_id(&self, chapter_id: i64) -> Result<Vec<ChapterParagraph>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, chapter_id, paragraph_index, content, type, character_id,
                    audio_id, voice_id, speed, pitch, volume, emotion, audio_path, duration,
                    status, error_msg, task_id, task_token, ambient_sound_id,
                    ambient_volume, ambient_fade_in, ambient_fade_out, mixed_audio_path,
                    created_at, updated_at
             FROM chapter_paragraphs
             WHERE chapter_id = ?
             ORDER BY paragraph_index ASC"
        )?;

        let rows = stmt.query_map([chapter_id], |row| {
            Ok(ChapterParagraph {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                paragraph_index: row.get(2)?,
                content: row.get(3)?,
                r#type: row.get(4)?,
                character_id: row.get(5)?,
                audio_id: row.get(6)?,
                voice_id: row.get(7)?,
                speed: row.get(8)?,
                pitch: row.get(9)?,
                volume: row.get(10)?,
                emotion: row.get(11)?,
                audio_path: row.get(12)?,
                duration: row.get(13)?,
                status: row.get(14)?,
                error_msg: row.get(15)?,
                task_id: row.get(16)?,
                task_token: row.get(17)?,
                ambient_sound_id: row.get(18)?,
                ambient_volume: row.get(19)?,
                ambient_fade_in: row.get(20)?,
                ambient_fade_out: row.get(21)?,
                mixed_audio_path: row.get(22)?,
                created_at: row.get(23)?,
                updated_at: row.get(24)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    // 通过 paragraph_id 获取单个段落
    #[allow(dead_code)]
    // 初始化段落的音频数据
    #[allow(dead_code)]
    pub fn init_paragraphs_audio(&self, audio_id: i64, chapter_id: i64, default_voice: &str, default_speed: f32) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            "UPDATE chapter_paragraphs 
             SET audio_id = ?, voice_id = ?, speed = ?, status = 'pending'
             WHERE chapter_id = ?",
            (audio_id, default_voice, default_speed, chapter_id),
        )?;
        Ok(())
    }

    // 更新段落音频结果
    #[allow(dead_code)]
    pub fn update_paragraph_audio_result(
        &self,
        paragraph_id: i64,
        status: &str,
        audio_path: Option<&str>,
        duration: Option<i32>,
        error_msg: Option<&str>,
        task_id: Option<&str>,
        task_token: Option<&str>,
    ) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"UPDATE chapter_paragraphs 
               SET status = ?, audio_path = ?, duration = ?, error_msg = ?, task_id = ?, task_token = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            (status, audio_path, duration, error_msg, task_id, task_token, paragraph_id),
        )?;
        
        // 更新音频进度
        let audio_id: Option<i64> = conn.query_row(
            "SELECT audio_id FROM chapter_paragraphs WHERE id = ?",
            [paragraph_id],
            |row| row.get(0),
        )?;
        
        if let Some(aid) = audio_id {
            let completed_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM chapter_paragraphs WHERE audio_id = ? AND status = 'completed'",
                [aid],
                |row| row.get(0),
            )?;
            
            let total_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM chapter_paragraphs WHERE audio_id = ?",
                [aid],
                |row| row.get(0),
            )?;
            
            conn.execute(
                "UPDATE chapter_audios SET completed_sentences = ?, total_sentences = ? WHERE id = ?",
                (completed_count, total_count, aid),
            )?;
        }
        
        Ok(())
    }

    // 更新段落音频参数（仅速度）
    // 注意：voice_id 已移除，音色通过 character_id 从角色获取
    #[allow(dead_code)]
    pub fn update_paragraph_audio_params(&self, paragraph_id: i64, speed: Option<f32>) -> Result<(), DbError> {
        let conn = self.get_conn();
        
        if let Some(spd) = speed {
            conn.execute(
                r#"UPDATE chapter_paragraphs 
                   SET speed = ?, status = 'pending', error_msg = NULL, updated_at = CURRENT_TIMESTAMP
                   WHERE id = ?"#,
                (spd, paragraph_id),
            )?;
        }
        
        Ok(())
    }

    // 更新段落环境音
    #[allow(dead_code)]
    pub fn update_paragraph_ambient_sound(&self, paragraph_id: i64, ambient_sound_id: Option<i64>) -> Result<(), DbError> {
        let conn = self.get_conn();
        
        conn.execute(
            r#"UPDATE chapter_paragraphs
               SET ambient_sound_id = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            (ambient_sound_id, paragraph_id),
        )?;
        
        Ok(())
    }

    // 更新段落音频路径
    #[allow(dead_code)]
    pub fn update_paragraph_audio_path(&self, paragraph_id: i64, audio_path: &str) -> Result<(), DbError> {
        let conn = self.get_conn();
        
        conn.execute(
            r#"UPDATE chapter_paragraphs
               SET audio_path = ?, status = 'completed', updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            (audio_path, paragraph_id),
        )?;
        
        Ok(())
    }

    // 更新段落混音路径
    pub fn update_paragraph_mixed_path(&self, paragraph_id: i64, mixed_path: &str) -> Result<(), DbError> {
        let conn = self.get_conn();
        
        if mixed_path.is_empty() {
            conn.execute(
                r#"UPDATE chapter_paragraphs
                   SET mixed_audio_path = NULL, updated_at = CURRENT_TIMESTAMP
                   WHERE id = ?"#,
                [paragraph_id],
            )?;
        } else {
            conn.execute(
                r#"UPDATE chapter_paragraphs
                   SET mixed_audio_path = ?, updated_at = CURRENT_TIMESTAMP
                   WHERE id = ?"#,
                (mixed_path, paragraph_id),
            )?;
        }
        
        Ok(())
    }

    // 获取段落
    pub fn get_paragraph_by_id(&self, paragraph_id: i64) -> Result<Option<crate::db::models::ChapterParagraph>, DbError> {
        let conn = self.get_conn();
        
        let mut stmt = conn.prepare(
            r#"SELECT id, chapter_id, paragraph_index, content, type,
                       character_id, audio_id, voice_id, speed, pitch, volume,
                       emotion, audio_path, duration, status, error_msg,
                       task_id, task_token, ambient_sound_id, ambient_volume,
                       ambient_fade_in, ambient_fade_out, mixed_audio_path,
                       created_at, updated_at
                FROM chapter_paragraphs WHERE id = ?"#,
        )?;
        
        let result = stmt.query_row([paragraph_id], |row| {
            Ok(crate::db::models::ChapterParagraph {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                paragraph_index: row.get(2)?,
                content: row.get(3)?,
                r#type: row.get(4)?,
                character_id: row.get(5)?,
                audio_id: row.get(6)?,
                voice_id: row.get(7)?,
                speed: row.get(8)?,
                pitch: row.get(9)?,
                volume: row.get(10)?,
                emotion: row.get(11)?,
                audio_path: row.get(12)?,
                duration: row.get(13)?,
                status: row.get(14)?,
                error_msg: row.get(15)?,
                task_id: row.get(16)?,
                task_token: row.get(17)?,
                ambient_sound_id: row.get(18)?,
                ambient_volume: row.get(19)?,
                ambient_fade_in: row.get(20)?,
                ambient_fade_out: row.get(21)?,
                mixed_audio_path: row.get(22)?,
                created_at: row.get(23)?,
                updated_at: row.get(24)?,
            })
        });
        
        match result {
            Ok(para) => Ok(Some(para)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // 删除章节的所有段落标注
    #[allow(dead_code)]
    pub fn delete_chapter_paragraphs(&self, chapter_id: i64) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute(
            "DELETE FROM chapter_paragraphs WHERE chapter_id = ?",
            [chapter_id],
        )?;
        Ok(())
    }
}

// ==================== 环境音相关数据库操作
use crate::db::models::{AmbientSound, ChapterAmbientConfig};

impl Database {
    // 获取所有环境音
    #[allow(dead_code)]
    pub fn get_all_ambient_sounds(&self) -> Result<Vec<AmbientSound>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, name, description, category, prompt, file_path, 
                  duration, volume, is_loopable, is_system, tags, created_at
           FROM ambient_sounds ORDER BY is_system DESC, category, name"#,
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(AmbientSound {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                prompt: row.get(4)?,
                file_path: row.get(5)?,
                duration: row.get(6)?,
                volume: row.get(7)?,
                is_loopable: row.get(8)?,
                is_system: row.get(9)?,
                tags: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        
        let mut sounds = Vec::new();
        for row in rows {
            sounds.push(row?);
        }
        
        Ok(sounds)
    }

    // 按分类获取环境音
    #[allow(dead_code)]
    pub fn get_ambient_sounds_by_category(&self, category: &str) -> Result<Vec<AmbientSound>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, name, description, category, prompt, file_path, 
                  duration, volume, is_loopable, is_system, tags, created_at
           FROM ambient_sounds WHERE category = ? ORDER BY is_system DESC, name"#,
        )?;
        
        let rows = stmt.query_map([category], |row| {
            Ok(AmbientSound {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                prompt: row.get(4)?,
                file_path: row.get(5)?,
                duration: row.get(6)?,
                volume: row.get(7)?,
                is_loopable: row.get(8)?,
                is_system: row.get(9)?,
                tags: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        
        let mut sounds = Vec::new();
        for row in rows {
            sounds.push(row?);
        }
        
        Ok(sounds)
    }

    // 根据ID获取环境音
    #[allow(dead_code)]
    pub fn get_ambient_sound_by_id(&self, id: i64) -> Result<Option<AmbientSound>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, name, description, category, prompt, file_path, 
                  duration, volume, is_loopable, is_system, tags, created_at
           FROM ambient_sounds WHERE id = ?"#,
        )?;
        
        let result = stmt.query_row([id], |row| {
            Ok(AmbientSound {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                prompt: row.get(4)?,
                file_path: row.get(5)?,
                duration: row.get(6)?,
                volume: row.get(7)?,
                is_loopable: row.get(8)?,
                is_system: row.get(9)?,
                tags: row.get(10)?,
                created_at: row.get(11)?,
            })
        });
        
        match result {
            Ok(sound) => Ok(Some(sound)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // 创建环境音
    #[allow(dead_code)]
    pub fn create_ambient_sound(&self, name: &str, description: Option<&str>, 
                                 category: &str, prompt: Option<&str>, file_path: &str,
                                 duration: f32, volume: f32, is_loopable: bool) -> Result<i64, DbError> {
        let conn = self.get_conn();
        conn.execute(
            r#"INSERT INTO ambient_sounds 
               (name, description, category, prompt, file_path, duration, volume, is_loopable)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            (name, description, category, prompt, file_path, duration, volume, is_loopable),
        )?;
        
        Ok(conn.last_insert_rowid())
    }

    // 删除环境音
    #[allow(dead_code)]
    pub fn delete_ambient_sound(&self, id: i64) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute("DELETE FROM ambient_sounds WHERE id = ?", [id])?;
        Ok(())
    }

    // 获取章节环境音配置
    #[allow(dead_code)]
    pub fn get_chapter_ambient_configs(&self, chapter_id: i64) -> Result<Vec<ChapterAmbientConfig>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, chapter_id, ambient_sound_id, volume, fade_in, fade_out, start_paragraph, end_paragraph, created_at
               FROM chapter_ambient_config WHERE chapter_id = ?"#,
        )?;
        
        let rows = stmt.query_map([chapter_id], |row| {
            Ok(ChapterAmbientConfig {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                ambient_sound_id: row.get(2)?,
                volume: row.get(3)?,
                fade_in: row.get(4)?,
                fade_out: row.get(5)?,
                start_paragraph: row.get(6)?,
                end_paragraph: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        
        let mut configs = Vec::new();
        for row in rows {
            configs.push(row?);
        }
        
        Ok(configs)
    }

    // 保存章节环境音配置
    #[allow(dead_code)]
    pub fn save_chapter_ambient_config(&self, chapter_id: i64, ambient_sound_id: Option<i64>, 
                                         volume: Option<f32>, fade_in: Option<f32>, fade_out: Option<f32>) -> Result<i64, DbError> {
        let conn = self.get_conn();
        
        // 检查是否已存在配置
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM chapter_ambient_config WHERE chapter_id = ?",
            [chapter_id],
            |row| row.get(0),
        )?;
        
        if exists > 0 {
            // 更新现有配置
            conn.execute(
                r#"UPDATE chapter_ambient_config 
                   SET ambient_sound_id = COALESCE(?, ambient_sound_id),
                       volume = COALESCE(?, volume),
                       fade_in = COALESCE(?, fade_in),
                       fade_out = COALESCE(?, fade_out)
                   WHERE chapter_id = ?"#,
                (ambient_sound_id, volume, fade_in, fade_out, chapter_id),
            )?;
            
            // 返回配置ID
            let config_id: i64 = conn.query_row(
                "SELECT id FROM chapter_ambient_config WHERE chapter_id = ?",
                [chapter_id],
                |row| row.get(0),
            )?;
            
            Ok(config_id)
        } else {
            // 创建新配置
            conn.execute(
                r#"INSERT INTO chapter_ambient_config 
                   (chapter_id, ambient_sound_id, volume, fade_in, fade_out)
                   VALUES (?, ?, ?, ?, ?)"#,
                (chapter_id, ambient_sound_id, volume.unwrap_or(0.3), fade_in.unwrap_or(2.0), fade_out.unwrap_or(2.0)),
            )?;
            
            Ok(conn.last_insert_rowid())
        }
    }

    // 删除章节环境音配置
    #[allow(dead_code)]
    pub fn delete_chapter_ambient_config(&self, config_id: i64) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute("DELETE FROM chapter_ambient_config WHERE id = ?", [config_id])?;
        Ok(())
    }
}

// ==================== 混音相关数据库操作
use crate::db::models::{ParagraphAmbientConfig, AmbientMixPreset, SaveParagraphAmbientConfigRequest, SaveMixPresetRequest};

impl Database {
    // 获取段落环境音配置
    pub fn get_paragraph_ambient_configs(&self, paragraph_id: i64) -> Result<Vec<ParagraphAmbientConfig>, DbError> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            r#"SELECT id, paragraph_id, ambient_sound_id, position_offset, volume, fade_in, fade_out, fade_mode, is_muted, created_at, updated_at
               FROM paragraph_ambient_configs WHERE paragraph_id = ?"#,
        )?;

        let rows = stmt.query_map([paragraph_id], |row| {
            Ok(ParagraphAmbientConfig {
                id: row.get(0)?,
                paragraph_id: row.get(1)?,
                ambient_sound_id: row.get(2)?,
                position_offset: row.get(3)?,
                volume: row.get(4)?,
                fade_in: row.get(5)?,
                fade_out: row.get(6)?,
                fade_mode: row.get(7)?,
                is_muted: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(row?);
        }

        Ok(configs)
    }

    // 保存段落环境音配置
    pub fn save_paragraph_ambient_config(&self, config: &SaveParagraphAmbientConfigRequest) -> Result<ParagraphAmbientConfig, DbError> {
        let conn = self.get_conn();

        // 检查是否已存在配置
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM paragraph_ambient_configs WHERE paragraph_id = ? AND (ambient_sound_id = ? OR (ambient_sound_id IS NULL AND ? IS NULL))",
            rusqlite::params![config.paragraph_id, config.ambient_sound_id, config.ambient_sound_id],
            |row| row.get(0),
        ).ok();

        if let Some(config_id) = existing {
            // 更新现有配置
            conn.execute(
                r#"UPDATE paragraph_ambient_configs
                   SET position_offset = COALESCE(?, position_offset),
                       volume = COALESCE(?, volume),
                       fade_in = COALESCE(?, fade_in),
                       fade_out = COALESCE(?, fade_out),
                       fade_mode = COALESCE(?, fade_mode),
                       is_muted = COALESCE(?, is_muted),
                       updated_at = CURRENT_TIMESTAMP
                   WHERE id = ?"#,
                (config.position_offset, config.volume, config.fade_in, config.fade_out, config.fade_mode.clone(), config.is_muted.map(|v| v as i32), config_id),
            )?;
        } else {
            // 创建新配置
            conn.execute(
                r#"INSERT INTO paragraph_ambient_configs
                   (paragraph_id, ambient_sound_id, position_offset, volume, fade_in, fade_out, fade_mode, is_muted)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
                (config.paragraph_id, config.ambient_sound_id,
                 config.position_offset.unwrap_or(0.0),
                 config.volume.unwrap_or(0.3),
                 config.fade_in.unwrap_or(0.0),
                 config.fade_out.unwrap_or(0.0),
                 config.fade_mode.clone().unwrap_or_else(|| "linear".to_string()),
                 config.is_muted.unwrap_or(false) as i32),
            )?;
        }

        // 返回更新后的配置
        self.get_paragraph_ambient_configs(config.paragraph_id)
            .map(|configs| configs.into_iter().last())
            .and_then(|opt| opt.ok_or(DbError::Rusqlite(rusqlite::Error::QueryReturnedNoRows)))
    }

    // 删除段落环境音配置
    pub fn delete_paragraph_ambient_config(&self, config_id: i64) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute("DELETE FROM paragraph_ambient_configs WHERE id = ?", [config_id])?;
        Ok(())
    }

    // 获取章节所有段落的段落环境音配置
    pub fn get_chapter_paragraph_ambient_configs(&self, chapter_id: i64) -> Result<Vec<(i64, Vec<ParagraphAmbientConfig>)>, DbError> {
        let paragraphs = self.get_chapter_paragraphs(chapter_id)?;
        let mut results = Vec::new();

        for para in paragraphs {
            let configs = self.get_paragraph_ambient_configs(para.id)?;
            results.push((para.id, configs));
        }

        Ok(results)
    }

    // 获取混音预设列表
    pub fn get_mix_presets(&self, category: Option<String>) -> Result<Vec<AmbientMixPreset>, DbError> {
        let conn = self.get_conn();

        let mut presets = Vec::new();

        if let Some(cat) = category {
            let mut stmt = conn.prepare(
                r#"SELECT id, name, description, preset_data, default_volume, default_fade_in, default_fade_out, default_fade_mode, is_system, category, tags, created_at, updated_at
                   FROM ambient_mix_presets WHERE category = ? ORDER BY is_system DESC, name"#
            )?;
            let rows = stmt.query_map([cat], |row| {
                Ok(AmbientMixPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    preset_data: row.get(3)?,
                    default_volume: row.get(4)?,
                    default_fade_in: row.get(5)?,
                    default_fade_out: row.get(6)?,
                    default_fade_mode: row.get(7)?,
                    is_system: row.get::<_, i32>(8)? != 0,
                    category: row.get(9)?,
                    tags: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })?;
            for row in rows {
                presets.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                r#"SELECT id, name, description, preset_data, default_volume, default_fade_in, default_fade_out, default_fade_mode, is_system, category, tags, created_at, updated_at
                   FROM ambient_mix_presets ORDER BY is_system DESC, name"#
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(AmbientMixPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    preset_data: row.get(3)?,
                    default_volume: row.get(4)?,
                    default_fade_in: row.get(5)?,
                    default_fade_out: row.get(6)?,
                    default_fade_mode: row.get(7)?,
                    is_system: row.get::<_, i32>(8)? != 0,
                    category: row.get(9)?,
                    tags: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })?;
            for row in rows {
                presets.push(row?);
            }
        }

        Ok(presets)
    }

    // 保存混音预设
    pub fn save_mix_preset(&self, preset: &SaveMixPresetRequest) -> Result<AmbientMixPreset, DbError> {
        let conn = self.get_conn();

        conn.execute(
            r#"INSERT INTO ambient_mix_presets
               (name, description, preset_data, default_volume, default_fade_in, default_fade_out, default_fade_mode, category, tags)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            (preset.name.clone(), preset.description.clone(), preset.preset_data.clone(),
             preset.default_volume.unwrap_or(0.3),
             preset.default_fade_in.unwrap_or(2.0),
             preset.default_fade_out.unwrap_or(2.0),
             preset.default_fade_mode.clone().unwrap_or_else(|| "linear".to_string()),
             preset.category.clone(),
             preset.tags.clone()),
        )?;

        let id = conn.last_insert_rowid();

        conn.query_row(
            "SELECT id, name, description, preset_data, default_volume, default_fade_in, default_fade_out, default_fade_mode, is_system, category, tags, created_at, updated_at FROM ambient_mix_presets WHERE id = ?",
            [id],
            |row| {
                Ok(AmbientMixPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    preset_data: row.get(3)?,
                    default_volume: row.get(4)?,
                    default_fade_in: row.get(5)?,
                    default_fade_out: row.get(6)?,
                    default_fade_mode: row.get(7)?,
                    is_system: row.get::<_, i32>(8)? != 0,
                    category: row.get(9)?,
                    tags: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        ).map_err(|e| e.into())
    }

    // 删除混音预设
    pub fn delete_mix_preset(&self, preset_id: i64) -> Result<(), DbError> {
        let conn = self.get_conn();
        conn.execute("DELETE FROM ambient_mix_presets WHERE id = ? AND is_system = 0", [preset_id])?;
        Ok(())
    }
}
