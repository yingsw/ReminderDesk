use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct DbState(pub Mutex<Connection>);

pub fn init_database(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("reminders.db");
    let conn = Connection::open(&db_path)?;

    // 创建分类表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            color TEXT DEFAULT '#3b82f6',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 创建任务表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reminders (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            priority INTEGER DEFAULT 1,
            category_id INTEGER,
            due_time TEXT NOT NULL,
            reminder_function TEXT,
            is_completed INTEGER DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (category_id) REFERENCES categories(id)
        )",
        [],
    )?;

    // 检查是否需要添加默认分类
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
    if count == 0 {
        // 插入默认分类
        let default_categories = [
            ("工作", "#3b82f6"),
            ("个人", "#22c55e"),
            ("会议", "#f97316"),
            ("其他", "#6b7280"),
        ];
        for (name, color) in default_categories {
            conn.execute(
                "INSERT INTO categories (name, color) VALUES (?1, ?2)",
                rusqlite::params![name, color],
            )?;
        }
    }

    // 检查是否需要添加 category_id 列（兼容旧数据库）
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(reminders)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|c| c == "category_id") {
        conn.execute("ALTER TABLE reminders ADD COLUMN category_id INTEGER", [])?;
    }

    app.manage(DbState(Mutex::new(conn)));

    Ok(())
}