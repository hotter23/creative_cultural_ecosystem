use std::sync::Arc;
use tauri::State;
use crate::db::{Database, DbError};
use crate::db::models::{Novel, CreateNovelRequest, UpdateNovelRequest};

#[tauri::command]
pub fn get_novels(db: State<Arc<Database>>) -> Result<Vec<Novel>, String> {
    let conn = db.get_conn();
    let mut stmt = conn
        .prepare("SELECT id, title, description, cover_path, status, current_stage, total_chapters, total_words, created_at, updated_at FROM novels ORDER BY updated_at DESC")
        .map_err(DbError::from)?;
    
    let novels = stmt
        .query_map([], |row| {
            Ok(Novel {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cover_path: row.get(3)?,
                status: row.get(4)?,
                current_stage: row.get(5)?,
                total_chapters: row.get(6)?,
                total_words: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(DbError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DbError::from)?;
    
    Ok(novels)
}

#[tauri::command]
pub fn get_novel(db: State<Arc<Database>>, id: i64) -> Result<Novel, String> {
    let conn = db.get_conn();
    let mut stmt = conn
        .prepare("SELECT id, title, description, cover_path, status, current_stage, total_chapters, total_words, created_at, updated_at FROM novels WHERE id = ?1")
        .map_err(DbError::from)?;
    
    let novel = stmt
        .query_row([id], |row| {
            Ok(Novel {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cover_path: row.get(3)?,
                status: row.get(4)?,
                current_stage: row.get(5)?,
                total_chapters: row.get(6)?,
                total_words: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(DbError::from)?;
    
    Ok(novel)
}

#[tauri::command]
pub fn create_novel(
    db: State<Arc<Database>>,
    data: CreateNovelRequest,
) -> Result<Novel, String> {
    let conn = db.get_conn();
    
    let now = chrono::Local::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO novels (title, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        (&data.title, &data.description, &now, &now),
    )
    .map_err(DbError::from)?;
    
    let id = conn.last_insert_rowid();
    
    Ok(Novel {
        id,
        title: data.title,
        description: data.description,
        cover_path: None,
        status: "draft".to_string(),
        current_stage: "novel".to_string(),
        total_chapters: 0,
        total_words: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_novel(
    db: State<Arc<Database>>,
    id: i64,
    data: UpdateNovelRequest,
) -> Result<(), String> {
    let conn = db.get_conn();
    
    let mut updates = vec![];
    let mut params: Vec<&dyn rusqlite::ToSql> = vec![];
    
    if let Some(title) = &data.title {
        updates.push("title = ?");
        params.push(title);
    }
    if let Some(description) = &data.description {
        updates.push("description = ?");
        params.push(description);
    }
    if let Some(cover_path) = &data.cover_path {
        updates.push("cover_path = ?");
        params.push(cover_path);
    }
    if let Some(status) = &data.status {
        updates.push("status = ?");
        params.push(status);
    }
    if let Some(stage) = &data.current_stage {
        updates.push("current_stage = ?");
        params.push(stage);
    }
    
    if updates.is_empty() {
        return Ok(());
    }
    
    let sql = format!(
        "UPDATE novels SET {}, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        updates.join(", ")
    );
    
    params.push(&id);
    
    conn.execute(&sql, params.as_slice())
        .map_err(DbError::from)?;
    
    Ok(())
}

#[tauri::command]
pub fn delete_novel(db: State<Arc<Database>>, id: i64) -> Result<(), String> {
    let conn = db.get_conn();
    conn.execute("DELETE FROM novels WHERE id = ?1", [id])
        .map_err(DbError::from)?;
    Ok(())
}
