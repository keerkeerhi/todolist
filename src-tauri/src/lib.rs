use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use tauri::{Manager, State};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[derive(Debug, Serialize, FromRow)]
struct Todo {
    id: String,
    title: String,
    completed: i64,
    position: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
struct Memo {
    id: String,
    title: String,
    body: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct TodoInput {
    id: Option<String>,
    title: String,
}

#[derive(Debug, Deserialize)]
struct MemoInput {
    id: Option<String>,
    title: String,
    body: String,
}

async fn init_pool(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

fn clean(value: &str) -> String { value.trim().to_string() }

#[tauri::command]
async fn list_todos(state: State<'_, AppState>) -> Result<Vec<Todo>, String> {
    sqlx::query_as::<_, Todo>("SELECT * FROM todos ORDER BY completed ASC, position ASC, updated_at DESC")
        .fetch_all(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_todo(state: State<'_, AppState>, input: TodoInput) -> Result<Todo, String> {
    let title = clean(&input.title);
    if title.is_empty() { return Err("待办事项不能为空".into()); }
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    sqlx::query("INSERT INTO todos (id, title, position) VALUES (?1, ?2, (SELECT COALESCE(MAX(position), -1) + 1 FROM todos)) ON CONFLICT(id) DO UPDATE SET title = excluded.title, updated_at = datetime('now')")
        .bind(&id).bind(&title).execute(&state.pool).await.map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?1").bind(id).fetch_one(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn move_todo(state: State<'_, AppState>, id: String, direction: i64) -> Result<(), String> {
    if direction != -1 && direction != 1 { return Err("无效的移动方向".into()); }
    let mut tx = state.pool.begin().await.map_err(|e| e.to_string())?;
    let current = sqlx::query_as::<_, (i64, i64)>("SELECT position, completed FROM todos WHERE id = ?1")
        .bind(&id).fetch_optional(&mut *tx).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "待办事项不存在".to_string())?;
    let neighbor = if direction < 0 {
        sqlx::query_as::<_, (String, i64)>("SELECT id, position FROM todos WHERE completed = ?1 AND position < ?2 ORDER BY position DESC LIMIT 1")
            .bind(current.1).bind(current.0).fetch_optional(&mut *tx).await.map_err(|e| e.to_string())?
    } else {
        sqlx::query_as::<_, (String, i64)>("SELECT id, position FROM todos WHERE completed = ?1 AND position > ?2 ORDER BY position ASC LIMIT 1")
            .bind(current.1).bind(current.0).fetch_optional(&mut *tx).await.map_err(|e| e.to_string())?
    };
    if let Some((neighbor_id, neighbor_position)) = neighbor {
        sqlx::query("UPDATE todos SET position = ?1 WHERE id = ?2").bind(neighbor_position).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        sqlx::query("UPDATE todos SET position = ?1 WHERE id = ?2").bind(current.0).bind(neighbor_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_todo_completed(state: State<'_, AppState>, id: String, completed: bool) -> Result<(), String> {
    sqlx::query("UPDATE todos SET completed = ?1, updated_at = datetime('now') WHERE id = ?2")
        .bind(completed as i64).bind(id).execute(&state.pool).await.map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
async fn delete_todo(state: State<'_, AppState>, id: String) -> Result<(), String> {
    sqlx::query("DELETE FROM todos WHERE id = ?1").bind(id).execute(&state.pool).await.map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
async fn list_memos(state: State<'_, AppState>) -> Result<Vec<Memo>, String> {
    sqlx::query_as::<_, Memo>("SELECT * FROM memos ORDER BY updated_at DESC")
        .fetch_all(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_memo(state: State<'_, AppState>, input: MemoInput) -> Result<Memo, String> {
    let title = clean(&input.title);
    let body = input.body.trim().to_string();
    if title.is_empty() && body.is_empty() { return Err("备忘录内容不能为空".into()); }
    let title = if title.is_empty() { "未命名备忘录".into() } else { title };
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    sqlx::query("INSERT INTO memos (id, title, body) VALUES (?1, ?2, ?3) ON CONFLICT(id) DO UPDATE SET title = excluded.title, body = excluded.body, updated_at = datetime('now')")
        .bind(&id).bind(&title).bind(&body).execute(&state.pool).await.map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Memo>("SELECT * FROM memos WHERE id = ?1").bind(id).fetch_one(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_memo(state: State<'_, AppState>, id: String) -> Result<(), String> {
    sqlx::query("DELETE FROM memos WHERE id = ?1").bind(id).execute(&state.pool).await.map_err(|e| e.to_string()).map(|_| ())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            fs::create_dir_all(&dir)?;
            let pool = tauri::async_runtime::block_on(init_pool(&dir.join("red-horizon.db")))?;
            app.manage(AppState { pool });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_todos, save_todo, set_todo_completed, delete_todo, move_todo,
            list_memos, save_memo, delete_memo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
