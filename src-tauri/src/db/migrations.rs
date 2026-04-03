use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    // 初始迁移
    r#"
-- 项目主表
CREATE TABLE IF NOT EXISTS novels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    cover_path TEXT,
    status TEXT DEFAULT 'draft',
    current_stage TEXT DEFAULT 'novel',
    total_chapters INTEGER DEFAULT 0,
    total_words INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 章节表
CREATE TABLE IF NOT EXISTS chapters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    novel_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    plain_text TEXT,
    order_num INTEGER DEFAULT 0,
    word_count INTEGER DEFAULT 0,
    status TEXT DEFAULT 'draft',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 音频项目表
CREATE TABLE IF NOT EXISTS chapter_audios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    novel_id INTEGER NOT NULL,
    chapter_id INTEGER,
    status TEXT DEFAULT 'pending',
    total_sentences INTEGER DEFAULT 0,
    completed_sentences INTEGER DEFAULT 0,
    merged_audio_path TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 句子级音频表
CREATE TABLE IF NOT EXISTS audio_sentences (
    id TEXT PRIMARY KEY,
    audio_id INTEGER NOT NULL,
    sentence_index INTEGER,
    text TEXT NOT NULL,
    voice_id TEXT,
    speed REAL DEFAULT 1.0,
    pitch INTEGER DEFAULT 0,
    volume INTEGER DEFAULT 100,
    emotion TEXT DEFAULT 'neutral',
    audio_path TEXT,
    duration INTEGER DEFAULT 0,
    character_id INTEGER,
    is_dialogue INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending',
    error_msg TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (audio_id) REFERENCES chapter_audios(id) ON DELETE CASCADE
);

-- 角色表
CREATE TABLE IF NOT EXISTS characters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    novel_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    aliases TEXT,
    gender TEXT,
    role TEXT,
    description TEXT,
    appearance TEXT,
    personality TEXT,
    voice_id TEXT,
    tags TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 角色形象表
CREATE TABLE IF NOT EXISTS character_images (
    id TEXT PRIMARY KEY,
    character_id INTEGER NOT NULL,
    image_type TEXT,
    pose TEXT,
    expression TEXT,
    image_path TEXT NOT NULL,
    prompt TEXT,
    seed INTEGER,
    is_default INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- 视频项目表
CREATE TABLE IF NOT EXISTS videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    novel_id INTEGER NOT NULL,
    chapter_id INTEGER,
    title TEXT NOT NULL,
    audio_path TEXT NOT NULL,
    video_path TEXT,
    duration INTEGER DEFAULT 0,
    resolution TEXT DEFAULT '1080p',
    status TEXT DEFAULT 'pending',
    progress INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
);

-- 视频分镜表
CREATE TABLE IF NOT EXISTS video_scenes (
    id TEXT PRIMARY KEY,
    video_id INTEGER NOT NULL,
    scene_order INTEGER,
    start_time REAL,
    duration REAL,
    scene_type TEXT,
    text_content TEXT,
    bg_type TEXT,
    bg_path TEXT,
    bg_prompt TEXT,
    image_path TEXT,
    status TEXT DEFAULT 'pending',
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_chapters_novel_id ON chapters(novel_id);
CREATE INDEX IF NOT EXISTS idx_audio_sentences_audio_id ON audio_sentences(audio_id);
CREATE INDEX IF NOT EXISTS idx_character_images_character_id ON character_images(character_id);
CREATE INDEX IF NOT EXISTS idx_video_scenes_video_id ON video_scenes(video_id);
CREATE INDEX IF NOT EXISTS idx_novels_status ON novels(status);
"#,
    // 迁移2: 添加系统配置表
    r#"
-- 系统配置表（存储 API Key、设置等）
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    category TEXT DEFAULT 'general',
    is_encrypted INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 插入默认配置
INSERT OR IGNORE INTO system_config (key, value, category) VALUES 
('minimax_enabled', 'false', 'minimax'),
('minimax_api_key', '', 'minimax'),
('minimax_base_url', 'https://api.minimaxi.com', 'minimax'),
('minimax_default_model', 'minimaxi-2.7', 'minimax'),
('minimax_group_id', '', 'minimax'),
('tts_default_voice', 'voice_female_01', 'tts'),
('tts_speed', '1.0', 'tts');
"#,
    // 迁移3: 为 audio_sentences 表添加 task_id 和 task_token 字段
    r#"
-- 为音频句子表添加 task_id 和 task_token 字段
ALTER TABLE audio_sentences ADD COLUMN task_id TEXT;
ALTER TABLE audio_sentences ADD COLUMN task_token TEXT;
"#,
    // 迁移4: 添加章节段落标注表
    r#"
-- 章节段落标注表
CREATE TABLE IF NOT EXISTS chapter_paragraphs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER NOT NULL,
    paragraph_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'narration',
    character_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE SET NULL,
    UNIQUE(chapter_id, paragraph_index)
);

-- 为章节段落标注表创建索引
CREATE INDEX IF NOT EXISTS idx_chapter_paragraphs_chapter_id ON chapter_paragraphs(chapter_id);
CREATE INDEX IF NOT EXISTS idx_chapter_paragraphs_character_id ON chapter_paragraphs(character_id);
"#,
    // 迁移5: 为 chapter_paragraphs 添加音频相关字段
    r#"
-- 为章节段落标注表添加音频相关字段
ALTER TABLE chapter_paragraphs ADD COLUMN audio_id INTEGER;
ALTER TABLE chapter_paragraphs ADD COLUMN voice_id TEXT;
ALTER TABLE chapter_paragraphs ADD COLUMN speed REAL DEFAULT 1.0;
ALTER TABLE chapter_paragraphs ADD COLUMN pitch INTEGER DEFAULT 0;
ALTER TABLE chapter_paragraphs ADD COLUMN volume INTEGER DEFAULT 100;
ALTER TABLE chapter_paragraphs ADD COLUMN emotion TEXT DEFAULT 'neutral';
ALTER TABLE chapter_paragraphs ADD COLUMN audio_path TEXT;
ALTER TABLE chapter_paragraphs ADD COLUMN duration INTEGER;
ALTER TABLE chapter_paragraphs ADD COLUMN status TEXT DEFAULT 'pending';
ALTER TABLE chapter_paragraphs ADD COLUMN error_msg TEXT;
ALTER TABLE chapter_paragraphs ADD COLUMN task_id TEXT;
ALTER TABLE chapter_paragraphs ADD COLUMN task_token TEXT;

-- 创建音频相关索引
CREATE INDEX IF NOT EXISTS idx_chapter_paragraphs_audio_id ON chapter_paragraphs(audio_id);
"#,
    // 迁移6: 环境音相关表
    r#"
-- 环境音素材表
CREATE TABLE IF NOT EXISTS ambient_sounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL DEFAULT 'custom',
    prompt TEXT,
    file_path TEXT NOT NULL,
    duration REAL DEFAULT 0,
    volume REAL DEFAULT 0.3,
    is_loopable INTEGER DEFAULT 1,
    is_system INTEGER DEFAULT 0,
    tags TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 章节环境音配置表
CREATE TABLE IF NOT EXISTS chapter_ambient_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER NOT NULL,
    ambient_sound_id INTEGER,
    volume REAL DEFAULT 0.3,
    fade_in REAL DEFAULT 2.0,
    fade_out REAL DEFAULT 2.0,
    start_paragraph INTEGER DEFAULT 0,
    end_paragraph INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
    FOREIGN KEY (ambient_sound_id) REFERENCES ambient_sounds(id) ON DELETE SET NULL,
    UNIQUE(chapter_id, ambient_sound_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_ambient_sounds_category ON ambient_sounds(category);
CREATE INDEX IF NOT EXISTS idx_chapter_ambient_config_chapter_id ON chapter_ambient_config(chapter_id);

-- 插入预置场景模板
INSERT OR IGNORE INTO ambient_sounds (name, description, category, prompt, volume, is_loopable, is_system) VALUES 
('雨天咖啡馆', '安静的咖啡馆背景音，窗外淅淅沥沥的小雨声', '城市', '安静的咖啡馆背景音，窗外淅淅沥沥的小雨声，轻微的杯碟碰撞声', 0.25, 1, 1),
('森林早晨', '茂密森林的清晨，鸟鸣声，树叶沙沙声', '自然', '茂密森林的清晨，各种鸟鸣，树叶沙沙声，远处溪流', 0.3, 1, 1),
('海滩放松', '轻柔的海浪拍打沙滩，远处海鸥鸣叫', '自然', '轻柔的海浪拍打沙滩，远处海鸥鸣叫，温暖的海风', 0.35, 1, 1),
('深夜书房', '安静的书房，轻微的翻书声，时钟滴答', '室内', '安静的书房，轻微的翻书声，时钟滴答声', 0.15, 1, 1),
('壁炉温暖', '壁炉木柴燃烧声，噼啪作响', '室内', '壁炉木柴燃烧声，噼啪作响，温暖舒适', 0.3, 1, 1),
('城市雨夜', '城市街道的雨夜，雨伞滴水，远处汽车驶过', '城市', '城市街道的雨夜，雨伞滴水，远处汽车驶过积水', 0.25, 1, 1);
"#,
    // 迁移7: 为 chapter_paragraphs 添加环境音字段
    r#"
-- 为章节段落标注表添加环境音关联字段
ALTER TABLE chapter_paragraphs ADD COLUMN ambient_sound_id INTEGER;
ALTER TABLE chapter_paragraphs ADD COLUMN ambient_volume REAL DEFAULT 0.3;
ALTER TABLE chapter_paragraphs ADD COLUMN ambient_fade_in REAL DEFAULT 0.0;
ALTER TABLE chapter_paragraphs ADD COLUMN ambient_fade_out REAL DEFAULT 0.0;

-- 创建环境音关联索引
CREATE INDEX IF NOT EXISTS idx_chapter_paragraphs_ambient_sound_id ON chapter_paragraphs(ambient_sound_id);
"#,
    // 迁移8: 删除段落中的 voice_id 字段（音色从角色获取）
    // 此迁移使用 SQLite 3.35.0+ 的 DROP COLUMN 功能
    // 如果列不存在会报错，我们捕获并忽略
    r#"
-- 删除 chapter_paragraphs 表中的 voice_id 字段
-- 音色现在通过 character_id 关联到角色，从角色获取音色ID
"#,
    // 迁移9: 为 chapter_paragraphs 添加混音路径字段
    r#"
-- 为章节段落标注表添加混音路径字段
-- 段落混音后的音频文件路径
ALTER TABLE chapter_paragraphs ADD COLUMN mixed_audio_path TEXT;

-- 创建混音路径索引
CREATE INDEX IF NOT EXISTS idx_chapter_paragraphs_mixed_audio_path ON chapter_paragraphs(mixed_audio_path);
"#,
    // 迁移10: 为 chapter_audios 添加混音路径字段
    r#"
-- 为音频项目表添加章节混音路径字段
-- 章节混音后的音频文件路径
ALTER TABLE chapter_audios ADD COLUMN mixed_audio_path TEXT;
"#,
    // 迁移11: 创建段落环境音配置表
    r#"
-- 段落环境音配置表
-- 支持为每个段落单独配置环境音
CREATE TABLE IF NOT EXISTS paragraph_ambient_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    paragraph_id INTEGER NOT NULL,
    ambient_sound_id INTEGER,
    position_offset REAL DEFAULT 0.0,
    volume REAL DEFAULT 0.3,
    fade_in REAL DEFAULT 0.0,
    fade_out REAL DEFAULT 0.0,
    fade_mode TEXT DEFAULT 'linear',
    is_muted INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (paragraph_id) REFERENCES chapter_paragraphs(id) ON DELETE CASCADE,
    FOREIGN KEY (ambient_sound_id) REFERENCES ambient_sounds(id) ON DELETE SET NULL,
    UNIQUE(paragraph_id, ambient_sound_id)
);

-- 创建段落环境音配置索引
CREATE INDEX IF NOT EXISTS idx_paragraph_ambient_configs_paragraph_id ON paragraph_ambient_configs(paragraph_id);
CREATE INDEX IF NOT EXISTS idx_paragraph_ambient_configs_ambient_sound_id ON paragraph_ambient_configs(ambient_sound_id);
"#,
    // 迁移12: 创建混音预设模板表
    r#"
-- 混音预设模板表
-- 存储用户创建的混音配置模板
CREATE TABLE IF NOT EXISTS ambient_mix_presets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    preset_data TEXT NOT NULL,
    default_volume REAL DEFAULT 0.3,
    default_fade_in REAL DEFAULT 2.0,
    default_fade_out REAL DEFAULT 2.0,
    default_fade_mode TEXT DEFAULT 'linear',
    is_system INTEGER DEFAULT 0,
    category TEXT,
    tags TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 创建混音预设索引
CREATE INDEX IF NOT EXISTS idx_ambient_mix_presets_category ON ambient_mix_presets(category);
CREATE INDEX IF NOT EXISTS idx_ambient_mix_presets_is_system ON ambient_mix_presets(is_system);

-- 插入系统预设
INSERT OR IGNORE INTO ambient_mix_presets (name, description, preset_data, default_volume, default_fade_in, default_fade_out, is_system, category, tags) VALUES
('森林探险', '森林环境，适合冒险场景', '{"tracks":[],"master_volume":0.8}', 0.3, 2.0, 2.0, 1, '自然', '森林,冒险,自然'),
('雨天咖啡馆', '雨天咖啡馆，适合温馨场景', '{"tracks":[],"master_volume":0.8}', 0.25, 1.5, 1.5, 1, '室内', '咖啡馆,雨天,温馨'),
('海边日落', '海边环境，适合浪漫场景', '{"tracks":[],"master_volume":0.8}', 0.35, 3.0, 3.0, 1, '自然', '海边,浪漫,日落'),
('城市喧嚣', '城市背景音，适合都市场景', '{"tracks":[],"master_volume":0.8}', 0.2, 1.0, 1.0, 1, '城市', '城市,喧嚣,都市');
"#,
];

pub fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    // 创建迁移版本表
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;

    // 获取当前版本
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 应用迁移
    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i32;
        if version > current_version {
            conn.execute_batch(migration)?;
            conn.execute(
                "INSERT INTO migrations (version) VALUES (?)",
                [version],
            )?;
        }
    }

    Ok(())
}
