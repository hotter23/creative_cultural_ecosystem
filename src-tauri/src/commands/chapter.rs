use crate::log;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tauri::State;
use crate::db::{Database, DbError};
use crate::db::models::{Chapter, CreateChapterRequest, UpdateChapterRequest, ChapterParagraph};

// 带音频状态的章节信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterWithAudioStatus {
    pub id: i64,
    pub novel_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub plain_text: Option<String>,
    pub order_num: i32,
    pub word_count: i32,
    pub status: String,
    pub created_at: String,
    pub audio_status: String,
}

#[tauri::command]
pub fn get_chapters(
    db: State<Arc<Database>>,
    #[allow(unused_variables)] novel_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] novelId: Option<i64>,
) -> Result<Vec<ChapterWithAudioStatus>, String> {
    let id = novel_id.or(novelId).unwrap_or(0);
    if id == 0 {
        return Err("缺少 novel_id 或 novelId 参数".to_string());
    }
    
    let conn = db.get_conn();
    // LEFT JOIN chapter_audios 表来获取音频状态
    let mut stmt = conn
        .prepare("
            SELECT 
                c.id, c.novel_id, c.title, c.content, c.plain_text, c.order_num, c.word_count, c.status, c.created_at,
                COALESCE(ca.status, 'not_created') as audio_status
            FROM chapters c 
            LEFT JOIN chapter_audios ca ON c.id = ca.chapter_id 
            WHERE c.novel_id = ?1 
            ORDER BY c.order_num ASC
        ")
        .map_err(DbError::from)?;
    
    let chapters = stmt
        .query_map([id], |row| {
            Ok(ChapterWithAudioStatus {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                plain_text: row.get(4)?,
                order_num: row.get(5)?,
                word_count: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                audio_status: row.get(9)?,
            })
        })
        .map_err(DbError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DbError::from)?;
    
    Ok(chapters)
}

#[tauri::command]
pub fn get_chapter(db: State<Arc<Database>>, id: i64) -> Result<Chapter, String> {
    let conn = db.get_conn();
    let mut stmt = conn
        .prepare("SELECT id, novel_id, title, content, plain_text, order_num, word_count, status, created_at FROM chapters WHERE id = ?1")
        .map_err(DbError::from)?;
    
    let chapter = stmt
        .query_row([id], |row| {
            Ok(Chapter {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                plain_text: row.get(4)?,
                order_num: row.get(5)?,
                word_count: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(DbError::from)?;
    
    Ok(chapter)
}

#[tauri::command]
pub fn create_chapter(
    db: State<Arc<Database>>,
    data: CreateChapterRequest,
) -> Result<Chapter, String> {
    let conn = db.get_conn();
    
    let word_count = data.content.as_ref().map(|c| c.chars().count()).unwrap_or(0) as i32;
    let plain_text = data.content.as_ref().map(|c| html2text::from_read(c.as_bytes(), 1000));
    
    let now = chrono::Local::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO chapters (novel_id, title, content, plain_text, order_num, word_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            data.novel_id,
            &data.title,
            &data.content,
            &plain_text,
            data.order_num,
            word_count,
            &now,
        ),
    )
    .map_err(DbError::from)?;
    
    let id = conn.last_insert_rowid();
    
    // 更新小说的章节数和字数
    update_novel_stats(&conn, data.novel_id)?;

    Ok(Chapter {
        id,
        novel_id: data.novel_id,
        title: data.title.clone(),
        content: data.content.clone(),
        plain_text: Some(plain_text.unwrap_or_default()),
        order_num: data.order_num.unwrap_or(0),
        word_count,
        status: "draft".to_string(),
        created_at: now.clone(),
    })
}

#[tauri::command]
pub fn update_chapter(
    db: State<Arc<Database>>,
    id: i64,
    data: UpdateChapterRequest,
) -> Result<(), String> {
    let conn = db.get_conn();
    
    let mut updates = vec![];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    
    if let Some(title) = &data.title {
        updates.push("title = ?");
        params.push(Box::new(title.clone()));
    }
    if let Some(content) = &data.content {
        updates.push("content = ?");
        params.push(Box::new(content.clone()));
        let plain_text = html2text::from_read(content.as_bytes(), 1000);
        updates.push("plain_text = ?");
        params.push(Box::new(plain_text));
        let word_count = content.chars().count() as i32;
        updates.push("word_count = ?");
        params.push(Box::new(word_count));
    }
    if let Some(order_num) = data.order_num {
        updates.push("order_num = ?");
        params.push(Box::new(order_num));
    }
    if let Some(status) = &data.status {
        updates.push("status = ?");
        params.push(Box::new(status.clone()));
    }
    
    if updates.is_empty() {
        return Ok(());
    }
    
    // 添加 id 参数
    params.push(Box::new(id));
    
    let sql = format!(
        "UPDATE chapters SET {} WHERE id = ?",
        updates.join(", ")
    );
    
    // 将 params 转换为引用切片
    let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    conn.execute(&sql, params_ref.as_slice())
        .map_err(DbError::from)?;
    
    // 更新小说的字数统计
    let novel_id: i64 = conn
        .query_row("SELECT novel_id FROM chapters WHERE id = ?1", [id], |row| row.get(0))
        .map_err(DbError::from)?;
    update_novel_stats(&conn, novel_id)?;
    
    Ok(())
}

#[tauri::command]
pub fn delete_chapter(db: State<Arc<Database>>, id: i64) -> Result<(), String> {
    let conn = db.get_conn();
    
    // 获取小说ID用于更新统计
    let novel_id: i64 = conn
        .query_row("SELECT novel_id FROM chapters WHERE id = ?1", [id], |row| row.get(0))
        .map_err(DbError::from)?;
    
    conn.execute("DELETE FROM chapters WHERE id = ?1", [id])
        .map_err(DbError::from)?;
    
    // 更新小说统计
    update_novel_stats(&conn, novel_id)?;
    
    Ok(())
}

// 保存章节段落标注
#[tauri::command]
#[allow(dead_code)]
pub fn save_chapter_paragraphs(
    db: State<Arc<Database>>,
    #[allow(unused_variables)] chapter_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] chapterId: Option<i64>,
    paragraphs: Vec<crate::db::models::ParagraphMarkRequest>,
) -> Result<(), String> {
    let id = chapter_id.or(chapterId).unwrap_or(0);
    if id == 0 {
        return Err("缺少 chapter_id 或 chapterId 参数".to_string());
    }
    Ok(db.save_chapter_paragraphs(id, paragraphs)
        .map_err(DbError::from)?)
}

// 获取章节的所有段落标注
#[tauri::command]
#[allow(dead_code)]
pub fn get_chapter_paragraphs(
    db: State<Arc<Database>>,
    chapter_id: Option<i64>,
    #[allow(non_snake_case)] chapterId: Option<i64>,
) -> Result<Vec<ChapterParagraph>, String> {
    log!("get_chapter_paragraphs: 收到 chapter_id = {:?}", chapter_id);
    log!("get_chapter_paragraphs: 收到 chapterId = {:?}", chapterId);
    
    let id = chapter_id.or(chapterId).unwrap_or(0);
    if id == 0 {
        log!("get_chapter_paragraphs: 错误 - 缺少 chapter_id 或 chapterId 参数");
        return Err("缺少 chapter_id 或 chapterId 参数".to_string());
    }
    log!("get_chapter_paragraphs: 使用 id = {}", id);
    
    let result = db.get_chapter_paragraphs(id)
        .map_err(DbError::from)?;
    log!("get_chapter_paragraphs: 查询到 {} 条段落", result.len());
    
    Ok(result)
}

// 删除章节的所有段落标注
#[tauri::command]
#[allow(dead_code)]
pub fn delete_chapter_paragraphs(
    db: State<Arc<Database>>,
    #[allow(unused_variables)] chapter_id: Option<i64>,
    #[allow(unused_variables)] #[allow(non_snake_case)] chapterId: Option<i64>,
) -> Result<(), String> {
    let id = chapter_id.or(chapterId).unwrap_or(0);
    if id == 0 {
        return Err("缺少 chapter_id 或 chapterId 参数".to_string());
    }
    Ok(db.delete_chapter_paragraphs(id)
        .map_err(DbError::from)?)
}

// 辅助函数：更新小说的章节数和总字数
pub fn update_novel_stats(conn: &rusqlite::Connection, novel_id: i64) -> Result<(), String> {
    // 更新总章节数
    conn.execute(
        "UPDATE novels SET total_chapters = (SELECT COUNT(*) FROM chapters WHERE novel_id = ?1) WHERE id = ?1",
        [novel_id],
    )
    .map_err(DbError::from)?;
    
    // 更新总字数
    conn.execute(
        "UPDATE novels SET total_words = (SELECT COALESCE(SUM(word_count), 0) FROM chapters WHERE novel_id = ?1) WHERE id = ?1",
        [novel_id],
    )
    .map_err(DbError::from)?;
    
    Ok(())
}
