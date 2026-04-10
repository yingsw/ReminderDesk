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
            is_pinned INTEGER DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            template_id TEXT,
            FOREIGN KEY (category_id) REFERENCES categories(id)
        )",
        [],
    )?;

    // 创建循环模板表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recurring_templates (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            priority INTEGER DEFAULT 1,
            category_id INTEGER,
            base_time TEXT NOT NULL,
            recurrence_type TEXT NOT NULL,
            recurrence_interval INTEGER DEFAULT 1,
            recurrence_days TEXT,
            end_type TEXT DEFAULT 'never',
            end_count INTEGER,
            end_date TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            is_active INTEGER DEFAULT 1,
            FOREIGN KEY (category_id) REFERENCES categories(id)
        )",
        [],
    )?;

    // 创建循环实例表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recurring_instances (
            id TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            reminder_id TEXT,
            due_time TEXT NOT NULL,
            is_completed INTEGER DEFAULT 0,
            completed_at TEXT,
            instance_number INTEGER DEFAULT 1,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (template_id) REFERENCES recurring_templates(id),
            FOREIGN KEY (reminder_id) REFERENCES reminders(id)
        )",
        [],
    )?;

    // 检查是否需要添加默认分类
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
    if count == 0 {
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

    // 兼容性检查：添加缺失的列
    add_column_if_missing(&conn, "reminders", "template_id", "TEXT")?;
    add_column_if_missing(&conn, "reminders", "category_id", "INTEGER")?;
    add_column_if_missing(&conn, "reminders", "is_pinned", "INTEGER DEFAULT 0")?;

    app.manage(DbState(Mutex::new(conn)));

    Ok(())
}

fn add_column_if_missing(conn: &Connection, table: &str, column: &str, col_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let columns: Vec<String> = conn
        .prepare(&format!("PRAGMA table_info({})", table))?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|c| c == column) {
        conn.execute(&format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type), [])?;
    }

    Ok(())
}