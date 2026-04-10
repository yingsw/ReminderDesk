use crate::database::DbState;
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use chrono::{DateTime, Local, Duration, Datelike, Weekday};

// ==================== 分类 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCategory {
    pub name: String,
    pub color: String,
}

#[command]
pub fn get_categories(db: State<DbState>) -> Result<Vec<Category>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, color FROM categories ORDER BY id ASC")
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(categories)
}

#[command]
pub fn add_category(db: State<DbState>, category: NewCategory) -> Result<Category, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO categories (name, color) VALUES (?1, ?2)",
        rusqlite::params![&category.name, &category.color],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(Category {
        id: id as i32,
        name: category.name,
        color: category.color,
    })
}

#[command]
pub fn update_category(db: State<DbState>, id: i32, name: String, color: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE categories SET name = ?1, color = ?2 WHERE id = ?3",
        rusqlite::params![&name, &color, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn delete_category(db: State<DbState>, id: i32) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 将该分类下的任务设为无分类
    conn.execute("UPDATE reminders SET category_id = NULL WHERE category_id = ?1", [id])
        .map_err(|e| e.to_string())?;

    // 删除分类
    conn.execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== 任务 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub due_time: String,
    pub reminder_function: String,
    pub is_completed: bool,
    pub is_pinned: bool,
    pub created_at: String,
    pub template_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewReminder {
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub category_id: Option<i32>,
    pub due_time: String,
    pub reminder_function: String,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResult {
    pub items: Vec<Reminder>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetRemindersParams {
    pub page: i32,
    pub page_size: i32,
    pub category_id: Option<i32>,
    pub status: Option<String>, // "all", "pending", "completed"
    pub sort_by: Option<String>, // "due_time", "created_at", "priority"
}

#[command]
pub fn get_reminders(db: State<DbState>, params: GetRemindersParams) -> Result<PaginatedResult, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 构建查询条件
    let mut where_clause = String::new();
    let mut conditions: Vec<String> = Vec::new();

    if let Some(cat_id) = params.category_id {
        conditions.push(format!("category_id = {}", cat_id));
    }

    if let Some(status) = &params.status {
        if status == "pending" {
            conditions.push("is_completed = 0".to_string());
        } else if status == "completed" {
            conditions.push("is_completed = 1".to_string());
        }
    }

    if !conditions.is_empty() {
        where_clause = " WHERE ".to_string() + &conditions.join(" AND ");
    }

    // 查询总数
    let total: i32 = conn
        .query_row(
            &format!("SELECT COUNT(*) FROM reminders{}", where_clause),
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // 计算分页
    let page = params.page.max(1);
    let page_size = params.page_size.max(1).min(100);
    let offset = (page - 1) * page_size;
    let total_pages = (total as f64 / page_size as f64).ceil() as i32;

    // 确定排序字段（置顶优先、未完成优先、然后按选择的字段排序）
    let sort_field = match params.sort_by.as_deref() {
        Some("created_at") => "r.created_at",
        Some("priority") => "r.priority DESC",
        _ => "r.due_time ASC", // 默认按到期时间
    };

    // 查询数据（置顶优先，未完成优先于已完成）
    let query = format!(
        "SELECT r.id, r.title, r.description, r.priority, r.category_id, c.name, c.color, r.due_time, r.reminder_function, r.is_completed, r.is_pinned, r.created_at, r.template_id
         FROM reminders r
         LEFT JOIN categories c ON r.category_id = c.id
         {} ORDER BY r.is_pinned DESC, r.is_completed ASC, {} LIMIT {} OFFSET {}",
        where_clause, sort_field, page_size, offset
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([], |row| {
            Ok(Reminder {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                category_id: row.get(4)?,
                category_name: row.get(5)?,
                category_color: row.get(6)?,
                due_time: row.get(7)?,
                reminder_function: row.get(8)?,
                is_completed: row.get::<_, i32>(9)? != 0,
                is_pinned: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                template_id: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(PaginatedResult {
        items,
        total,
        page,
        page_size,
        total_pages,
    })
}

#[command]
pub fn add_reminder(db: State<DbState>, reminder: NewReminder) -> Result<Reminder, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO reminders (id, title, description, priority, category_id, due_time, reminder_function, is_completed, is_pinned, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 0, ?8)",
        rusqlite::params![
            &id,
            &reminder.title,
            &reminder.description,
            &reminder.priority,
            &reminder.category_id,
            &reminder.due_time,
            &reminder.reminder_function,
            &now
        ],
    )
    .map_err(|e| e.to_string())?;

    // 获取分类信息
    let (category_name, category_color): (Option<String>, Option<String>) = match reminder.category_id {
        Some(cat_id) => conn
            .query_row(
                "SELECT name, color FROM categories WHERE id = ?1",
                [cat_id],
                |row| Ok((Some(row.get(0)?), Some(row.get(1)?))),
            )
            .unwrap_or((None, None)),
        None => (None, None),
    };

    Ok(Reminder {
        id,
        title: reminder.title,
        description: reminder.description,
        priority: reminder.priority,
        category_id: reminder.category_id,
        category_name,
        category_color,
        due_time: reminder.due_time,
        reminder_function: reminder.reminder_function,
        is_completed: false,
        is_pinned: false,
        created_at: now,
        template_id: None,
    })
}

#[command]
pub fn complete_reminder(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE reminders SET is_completed = 1, is_pinned = 0 WHERE id = ?1",
        [&id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn pin_reminder(db: State<DbState>, id: String, pinned: bool) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE reminders SET is_pinned = ?1 WHERE id = ?2",
        rusqlite::params![pinned as i32, &id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn delete_reminder(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM reminders WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn update_reminder(db: State<DbState>, id: String, title: String, description: String, priority: i32, category_id: Option<i32>, due_time: String, reminder_function: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE reminders SET title = ?1, description = ?2, priority = ?3, category_id = ?4, due_time = ?5, reminder_function = ?6 WHERE id = ?7",
        rusqlite::params![&title, &description, priority, category_id, &due_time, &reminder_function, &id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== 提醒时间计算 ====================

/// 计算提醒时间
pub fn calculate_reminder_time(due_time: &str, function: &str) -> Option<DateTime<Local>> {
    let due = DateTime::parse_from_rfc3339(due_time)
        .map(|dt| dt.with_timezone(&Local))
        .ok()?;

    // 内置函数
    match function {
        "完成时间提醒" => Some(due),
        "提前5分钟" => Some(due - Duration::minutes(5)),
        "提前10分钟" => Some(due - Duration::minutes(10)),
        "提前15分钟" => Some(due - Duration::minutes(15)),
        "提前20分钟" => Some(due - Duration::minutes(20)),
        "提前30分钟" => Some(due - Duration::minutes(30)),
        "提前45分钟" => Some(due - Duration::minutes(45)),
        "提前1小时" => Some(due - Duration::hours(1)),
        "提前2小时" => Some(due - Duration::hours(2)),
        "提前3小时" => Some(due - Duration::hours(3)),
        "提前6小时" => Some(due - Duration::hours(6)),
        "提前12小时" => Some(due - Duration::hours(12)),
        "提前1天" => Some(due - Duration::days(1)),
        "提前2天" => Some(due - Duration::days(2)),
        "提前3天" => Some(due - Duration::days(3)),
        "提前1周" => Some(due - Duration::weeks(1)),
        "当天早上7点" => due.date_naive().and_hms_opt(7, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "当天早上8点" => due.date_naive().and_hms_opt(8, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "当天早上9点" => due.date_naive().and_hms_opt(9, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "当天中午12点" => due.date_naive().and_hms_opt(12, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "当天傍晚17点" => due.date_naive().and_hms_opt(17, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "当天傍晚18点" => due.date_naive().and_hms_opt(18, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "当天晚上20点" => due.date_naive().and_hms_opt(20, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "第二天早上8点" => (due.date_naive() + Duration::days(1)).and_hms_opt(8, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        "第二天早上9点" => (due.date_naive() + Duration::days(1)).and_hms_opt(9, 0, 0).and_then(|t| t.and_local_timezone(Local).single()),
        _ => parse_custom_expression(due, function),
    }
}

/// 解析自定义表达式
fn parse_custom_expression(due: DateTime<Local>, expr: &str) -> Option<DateTime<Local>> {
    let expr = expr.trim().to_lowercase();

    // DueTime-1h, DueTime+30m, DueTime-1d
    if expr.starts_with("duetime") {
        let offset = &expr[7..];
        return apply_offset(due, offset);
    }

    // Date+9h (当天指定时间)
    if expr.starts_with("date") {
        let offset = &expr[4..];
        let base = due.date_naive().and_hms_opt(0, 0, 0).and_then(|t| t.and_local_timezone(Local).single());
        return base.and_then(|b| apply_offset(b, offset));
    }

    // Tomorrow+9h
    if expr.starts_with("tomorrow") {
        let offset = &expr[8..];
        let base = (due.date_naive() + Duration::days(1)).and_hms_opt(0, 0, 0).and_then(|t| t.and_local_timezone(Local).single());
        return base.and_then(|b| apply_offset(b, offset));
    }

    // NextWorkday+9h
    if expr.starts_with("nextworkday") {
        let offset = &expr[11..];
        let mut next = due.date_naive() + Duration::days(1);
        while next.weekday() == Weekday::Sat || next.weekday() == Weekday::Sun {
            next = next + Duration::days(1);
        }
        let base = next.and_hms_opt(0, 0, 0).and_then(|t| t.and_local_timezone(Local).single());
        return base.and_then(|b| apply_offset(b, offset));
    }

    // 默认返回到期时间
    Some(due)
}

fn apply_offset(base: DateTime<Local>, offset: &str) -> Option<DateTime<Local>> {
    let offset = offset.trim();
    if offset.is_empty() {
        return Some(base);
    }

    // 解析符号
    let sign: i64 = if offset.starts_with('+') {
        1
    } else if offset.starts_with('-') {
        -1
    } else {
        // 没有符号，直接返回原时间
        return Some(base);
    };

    // 获取单位（最后一个字符）
    let unit = offset.chars().last()?;

    // 提取数字部分：跳过符号，收集数字字符
    let num_str: String = offset
        .chars()
        .skip(1)
        .take_while(|c| c.is_ascii_digit())
        .collect();

    if num_str.is_empty() {
        return Some(base);
    }

    let num: i64 = num_str.parse().ok()?;

    match unit {
        'm' => Some(base + Duration::minutes(sign * num)),
        'h' => Some(base + Duration::hours(sign * num)),
        'd' => Some(base + Duration::days(sign * num)),
        _ => Some(base),
    }
}

// ==================== 数据导出导入 ====================

#[derive(Debug, Serialize)]
pub struct ExportData {
    pub categories: Vec<Category>,
    pub reminders: Vec<Reminder>,
    pub export_time: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportData {
    pub categories: Option<Vec<ImportCategory>>,
    pub reminders: Option<Vec<ImportReminder>>,
}

#[derive(Debug, Deserialize)]
pub struct ImportCategory {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportReminder {
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub category_name: Option<String>,
    pub due_time: String,
    pub reminder_function: String,
}

#[command]
pub fn export_data(db: State<DbState>) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 导出分类
    let mut cat_stmt = conn
        .prepare("SELECT id, name, color FROM categories ORDER BY id ASC")
        .map_err(|e| e.to_string())?;

    let categories = cat_stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 导出任务
    let mut rem_stmt = conn
        .prepare("SELECT r.id, r.title, r.description, r.priority, r.category_id, c.name, c.color, r.due_time, r.reminder_function, r.is_completed, r.is_pinned, r.created_at, r.template_id FROM reminders r LEFT JOIN categories c ON r.category_id = c.id ORDER BY r.created_at ASC")
        .map_err(|e| e.to_string())?;

    let reminders = rem_stmt
        .query_map([], |row| {
            Ok(Reminder {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                category_id: row.get(4)?,
                category_name: row.get(5)?,
                category_color: row.get(6)?,
                due_time: row.get(7)?,
                reminder_function: row.get(8)?,
                is_completed: row.get::<_, i32>(9)? != 0,
                is_pinned: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                template_id: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let export_data = ExportData {
        categories,
        reminders,
        export_time: Local::now().to_rfc3339(),
    };

    serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())
}

#[command]
pub fn import_data(db: State<DbState>, json_data: String, merge: bool) -> Result<ImportResult, String> {
    let import_data: ImportData = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut categories_imported = 0;
    let mut reminders_imported = 0;

    // 如果不合并，先清空数据
    if !merge {
        conn.execute("DELETE FROM reminders", []).map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM categories WHERE id > 4", []).map_err(|e| e.to_string())?; // 保留默认分类
    }

    // 导入分类
    if let Some(categories) = &import_data.categories {
        for cat in categories {
            // 检查是否已存在同名分类
            let exists: bool = conn
                .query_row("SELECT COUNT(*) > 0 FROM categories WHERE name = ?1", [&cat.name], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            if !exists {
                conn.execute(
                    "INSERT INTO categories (name, color) VALUES (?1, ?2)",
                    rusqlite::params![&cat.name, &cat.color],
                )
                .map_err(|e| e.to_string())?;
                categories_imported += 1;
            }
        }
    }

    // 导入任务
    if let Some(reminders) = &import_data.reminders {
        for rem in reminders {
            // 查找分类ID
            let category_id: Option<i32> = match &rem.category_name {
                Some(name) => conn
                    .query_row("SELECT id FROM categories WHERE name = ?1", [name], |row| row.get(0))
                    .ok(),
                None => None,
            };

            let id = uuid::Uuid::new_v4().to_string();
            let now = Local::now().to_rfc3339();

            conn.execute(
                "INSERT INTO reminders (id, title, description, priority, category_id, due_time, reminder_function, is_completed, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
                rusqlite::params![&id, &rem.title, &rem.description, rem.priority, category_id, &rem.due_time, &rem.reminder_function, &now],
            )
            .map_err(|e| e.to_string())?;

            reminders_imported += 1;
        }
    }

    Ok(ImportResult {
        categories_imported,
        reminders_imported,
    })
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub categories_imported: i32,
    pub reminders_imported: i32,
}