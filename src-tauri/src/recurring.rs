use crate::database::DbState;
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use chrono::{DateTime, Local, NaiveTime, Datelike, Duration, Timelike};
use rusqlite::OptionalExtension;
use uuid::Uuid;

// ==================== 数据结构 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTemplate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub base_time: String,           // HH:mm 格式
    pub recurrence_type: String,     // daily, weekly, monthly, custom
    pub recurrence_interval: i32,    // 间隔
    pub recurrence_days: Option<String>, // JSON 数组
    pub end_type: String,            // never, count, date
    pub end_count: Option<i32>,
    pub end_date: Option<String>,
    pub created_at: String,
    pub is_active: bool,
    pub next_due_time: Option<String>,
    pub completed_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct NewRecurringTemplate {
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub category_id: Option<i32>,
    pub base_time: String,
    pub recurrence_type: String,
    pub recurrence_interval: i32,
    pub recurrence_days: Option<Vec<i32>>,
    pub end_type: String,
    pub end_count: Option<i32>,
    pub end_date: Option<String>,
    pub generate_first: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringInstance {
    pub id: String,
    pub template_id: String,
    pub reminder_id: Option<String>,
    pub due_time: String,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub instance_number: i32,
    pub created_at: String,
    // 关联的任务信息
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
}

// ==================== API 命令 ====================

#[command]
pub fn get_recurring_templates(db: State<DbState>) -> Result<Vec<RecurringTemplate>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.description, t.priority, t.category_id, c.name, c.color,
                t.base_time, t.recurrence_type, t.recurrence_interval, t.recurrence_days,
                t.end_type, t.end_count, t.end_date, t.created_at, t.is_active
         FROM recurring_templates t
         LEFT JOIN categories c ON t.category_id = c.id
         WHERE t.is_active = 1
         ORDER BY t.created_at DESC"
    ).map_err(|e| e.to_string())?;

    let templates = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let completed_count = get_completed_count(&conn, &id).unwrap_or(0);
        let next_due = calculate_next_due(&conn, &id).ok().flatten();

        Ok(RecurringTemplate {
            id,
            title: row.get(1)?,
            description: row.get(2)?,
            priority: row.get(3)?,
            category_id: row.get(4)?,
            category_name: row.get(5)?,
            category_color: row.get(6)?,
            base_time: row.get(7)?,
            recurrence_type: row.get(8)?,
            recurrence_interval: row.get(9)?,
            recurrence_days: row.get(10)?,
            end_type: row.get(11)?,
            end_count: row.get(12)?,
            end_date: row.get(13)?,
            created_at: row.get(14)?,
            is_active: row.get::<_, i32>(15)? != 0,
            next_due_time: next_due,
            completed_count,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(templates)
}

#[command]
pub fn add_recurring_template(
    db: State<DbState>,
    template: NewRecurringTemplate
) -> Result<RecurringTemplate, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    let now = Local::now().to_rfc3339();
    let recurrence_days_json = template.recurrence_days.clone()
        .map(|d| serde_json::to_string(&d).unwrap_or_default());

    conn.execute(
        "INSERT INTO recurring_templates
         (id, title, description, priority, category_id, base_time,
          recurrence_type, recurrence_interval, recurrence_days,
          end_type, end_count, end_date, created_at, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1)",
        rusqlite::params![
            &id,
            &template.title,
            &template.description,
            template.priority,
            template.category_id,
            &template.base_time,
            &template.recurrence_type,
            template.recurrence_interval,
            recurrence_days_json,
            &template.end_type,
            template.end_count,
            &template.end_date,
            &now
        ],
    ).map_err(|e| e.to_string())?;

    // 获取分类信息
    let (category_name, category_color) = match template.category_id {
        Some(cat_id) => conn.query_row(
            "SELECT name, color FROM categories WHERE id = ?1",
            [cat_id],
            |row| Ok((Some(row.get(0)?), Some(row.get(1)?)))
        ).unwrap_or((None, None)),
        None => (None, None),
    };

    // 如果需要立即生成第一个任务
    let next_due_time = if template.generate_first.unwrap_or(false) {
        let time: NaiveTime = template.base_time.parse().unwrap_or_else(|_| {
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        });
        let days: Vec<i32> = template.recurrence_days.clone()
            .unwrap_or_default();

        if let Some(first_due) = calculate_next_occurrence_for_template(
            &template.recurrence_type,
            template.recurrence_interval,
            &days,
            &Local::now(),
            time
        ) {
            // 创建实例
            let instance_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO recurring_instances
                 (id, template_id, due_time, instance_number, created_at, is_completed)
                 VALUES (?1, ?2, ?3, 1, ?4, 0)",
                rusqlite::params![&instance_id, &id, &first_due.to_rfc3339(), &now],
            ).ok();

            // 创建提醒任务
            let reminder_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO reminders
                 (id, title, description, priority, category_id, due_time,
                  reminder_function, is_completed, created_at, template_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, '完成时间提醒', 0, ?7, ?8)",
                rusqlite::params![
                    &reminder_id,
                    &template.title,
                    &template.description,
                    template.priority,
                    template.category_id,
                    &first_due.to_rfc3339(),
                    &now,
                    &id
                ],
            ).ok();

            // 更新实例关联提醒ID
            conn.execute(
                "UPDATE recurring_instances SET reminder_id = ?1 WHERE id = ?2",
                [&reminder_id, &instance_id]
            ).ok();

            Some(first_due.to_rfc3339())
        } else {
            None
        }
    } else {
        None
    };

    Ok(RecurringTemplate {
        id,
        title: template.title,
        description: template.description,
        priority: template.priority,
        category_id: template.category_id,
        category_name,
        category_color,
        base_time: template.base_time,
        recurrence_type: template.recurrence_type,
        recurrence_interval: template.recurrence_interval,
        recurrence_days: recurrence_days_json,
        end_type: template.end_type,
        end_count: template.end_count,
        end_date: template.end_date,
        created_at: now,
        is_active: true,
        next_due_time,
        completed_count: 0,
    })
}

#[command]
pub fn update_recurring_template(
    db: State<DbState>,
    id: String,
    template: NewRecurringTemplate
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let recurrence_days_json = template.recurrence_days
        .map(|d| serde_json::to_string(&d).unwrap_or_default());

    conn.execute(
        "UPDATE recurring_templates SET
         title = ?1, description = ?2, priority = ?3, category_id = ?4,
         base_time = ?5, recurrence_type = ?6, recurrence_interval = ?7,
         recurrence_days = ?8, end_type = ?9, end_count = ?10, end_date = ?11
         WHERE id = ?12",
        rusqlite::params![
            &template.title,
            &template.description,
            template.priority,
            template.category_id,
            &template.base_time,
            &template.recurrence_type,
            template.recurrence_interval,
            recurrence_days_json,
            &template.end_type,
            template.end_count,
            &template.end_date,
            &id
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn delete_recurring_template(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 软删除
    conn.execute(
        "UPDATE recurring_templates SET is_active = 0 WHERE id = ?1",
        [&id]
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn get_recurring_instances(
    db: State<DbState>,
    template_id: String,
    limit: i32
) -> Result<Vec<RecurringInstance>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT i.id, i.template_id, i.reminder_id, i.due_time, i.is_completed,
                i.completed_at, i.instance_number, i.created_at,
                t.title, t.description, t.priority, c.name, c.color
         FROM recurring_instances i
         JOIN recurring_templates t ON i.template_id = t.id
         LEFT JOIN categories c ON t.category_id = c.id
         WHERE i.template_id = ?1
         ORDER BY i.due_time DESC
         LIMIT ?2"
    ).map_err(|e| e.to_string())?;

    let instances = stmt.query_map(rusqlite::params![&template_id, limit], |row| {
        Ok(RecurringInstance {
            id: row.get(0)?,
            template_id: row.get(1)?,
            reminder_id: row.get(2)?,
            due_time: row.get(3)?,
            is_completed: row.get::<_, i32>(4)? != 0,
            completed_at: row.get(5)?,
            instance_number: row.get(6)?,
            created_at: row.get(7)?,
            title: row.get(8)?,
            description: row.get(9)?,
            priority: row.get(10)?,
            category_name: row.get(11)?,
            category_color: row.get(12)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(instances)
}

#[command]
pub fn complete_recurring_instance(
    db: State<DbState>,
    instance_id: String
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = Local::now().to_rfc3339();

    conn.execute(
        "UPDATE recurring_instances SET is_completed = 1, completed_at = ?1 WHERE id = ?2",
        [&now, &instance_id]
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn preview_next_occurrences(
    template: NewRecurringTemplate,
    count: i32
) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let now = Local::now();

    let base_time: NaiveTime = template.base_time
        .parse()
        .map_err(|_| "Invalid time format")?;

    let days: Vec<i32> = template.recurrence_days.unwrap_or_default();

    for i in 0..count {
        if let Some(next) = calculate_nth_occurrence(
            &template.recurrence_type,
            template.recurrence_interval,
            &days,
            &now,
            i,
            base_time,
        ) {
            result.push(next.to_rfc3339());
        }
    }

    Ok(result)
}

// ==================== 内部函数 ====================

fn get_completed_count(conn: &rusqlite::Connection, template_id: &str) -> Result<i32, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM recurring_instances WHERE template_id = ?1 AND is_completed = 1",
        [template_id],
        |row| row.get(0)
    )
}

fn calculate_next_due(conn: &rusqlite::Connection, template_id: &str) -> Result<Option<String>, rusqlite::Error> {
    let now = Local::now().to_rfc3339();
    conn.query_row(
        "SELECT due_time FROM recurring_instances
         WHERE template_id = ?1 AND is_completed = 0 AND due_time > ?2
         ORDER BY due_time ASC LIMIT 1",
        rusqlite::params![template_id, now],
        |row| row.get(0)
    ).optional()
}

fn calculate_nth_occurrence(
    recurrence_type: &str,
    interval: i32,
    days: &[i32],
    base_date: &DateTime<Local>,
    n: i32,
    time: NaiveTime,
) -> Option<DateTime<Local>> {
    match recurrence_type {
        "daily" => {
            let today = base_date.date_naive();
            if let Some(today_dt) = today.and_hms_opt(time.hour(), time.minute(), 0)
                .and_then(|dt| dt.and_local_timezone(Local).single())
            {
                if today_dt > *base_date {
                    // 今天的时间还没过，从今天开始算
                    let date = today + Duration::days((n * interval) as i64);
                    return date.and_hms_opt(time.hour(), time.minute(), 0)
                        .and_then(|dt| dt.and_local_timezone(Local).single());
                }
            }
            // 今天的时间已过，从明天开始算
            let date = today + Duration::days(((n + 1) * interval) as i64);
            date.and_hms_opt(time.hour(), time.minute(), 0)
                .and_then(|dt| dt.and_local_timezone(Local).single())
        }
        "weekly" => {
            // days 存储星期几 (0=周一, 6=周日)
            if days.is_empty() {
                return None;
            }

            let mut current = base_date.date_naive();
            let mut found = 0;

            // 最多搜索365天
            for _ in 0..365 {
                let weekday = current.weekday().num_days_from_monday() as i32;
                if days.contains(&weekday) {
                    if let Some(dt) = current.and_hms_opt(time.hour(), time.minute(), 0)
                        .and_then(|d| d.and_local_timezone(Local).single())
                    {
                        if dt > *base_date {
                            if found >= n {
                                return Some(dt);
                            }
                            found += 1;
                        }
                    }
                }
                current = current + Duration::days(1);
            }
            None
        }
        "monthly" => {
            // days 存储每月几号
            if days.is_empty() {
                return None;
            }

            let mut current_month = base_date.month();
            let mut current_year = base_date.year();
            let mut found = 0;

            loop {
                for &day in days.iter() {
                    if let Some(date) = chrono::NaiveDate::from_ymd_opt(current_year, current_month, day as u32) {
                        let dt = date.and_hms_opt(time.hour(), time.minute(), 0)
                            .and_then(|d| d.and_local_timezone(Local).single());

                        if let Some(dt) = dt {
                            if dt > *base_date {
                                if found >= n {
                                    return Some(dt);
                                }
                                found += 1;
                            }
                        }
                    }
                }

                current_month += interval as u32;
                while current_month > 12 {
                    current_month -= 12;
                    current_year += 1;
                }

                if current_year > base_date.year() + 10 {
                    return None;
                }
            }
        }
        "custom" => {
            let today = base_date.date_naive();
            if let Some(today_dt) = today.and_hms_opt(time.hour(), time.minute(), 0)
                .and_then(|dt| dt.and_local_timezone(Local).single())
            {
                if today_dt > *base_date {
                    // 今天的时间还没过，从今天开始算
                    let date = today + Duration::days((n * interval) as i64);
                    return date.and_hms_opt(time.hour(), time.minute(), 0)
                        .and_then(|dt| dt.and_local_timezone(Local).single());
                }
            }
            // 今天的时间已过，从明天开始算
            let date = today + Duration::days(((n + 1) * interval) as i64);
            date.and_hms_opt(time.hour(), time.minute(), 0)
                .and_then(|dt| dt.and_local_timezone(Local).single())
        }
        _ => None
    }
}

// ==================== 定时任务：生成实例 ====================

pub fn generate_upcoming_instances(db: &DbState) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 获取所有活跃的循环模板
    let mut stmt = conn.prepare(
        "SELECT id, title, description, priority, category_id, base_time,
                recurrence_type, recurrence_interval, recurrence_days,
                end_type, end_count, end_date
         FROM recurring_templates WHERE is_active = 1"
    ).map_err(|e| e.to_string())?;

    let templates: Vec<(String, String, String, i32, Option<i32>, String,
                        String, i32, Option<String>, String, Option<i32>, Option<String>)> =
        stmt.query_map([], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?,
                row.get(10)?, row.get(11)?
            ))
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let now = Local::now();
    let future_limit = now + Duration::days(30); // 生成未来30天的实例

    for (id, title, description, priority, category_id, base_time,
         recurrence_type, interval, recurrence_days, end_type, end_count, end_date) in templates {

        // 检查是否已达到结束条件
        if should_end(&conn, &id, &end_type, end_count, &end_date, &now)? {
            continue;
        }

        // 解析时间
        let time: NaiveTime = base_time.parse().unwrap_or_else(|_| {
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        });

        // 解析循环日期
        let days: Vec<i32> = recurrence_days
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or_default();

        // 查找下一个应该生成的日期
        let mut current = now;
        while current < future_limit {
            if let Some(next) = calculate_next_occurrence_for_template(
                &recurrence_type, interval, &days, &current, time
            ) {
                // 检查是否已存在该实例
                let exists: bool = conn.query_row(
                    "SELECT COUNT(*) > 0 FROM recurring_instances
                     WHERE template_id = ?1 AND date(due_time) = date(?2)",
                    rusqlite::params![&id, next.to_rfc3339()],
                    |row| row.get(0)
                ).unwrap_or(true);

                if !exists {
                    // 创建实例
                    let instance_id = Uuid::new_v4().to_string();
                    let instance_number = get_next_instance_number(&conn, &id)?;

                    conn.execute(
                        "INSERT INTO recurring_instances
                         (id, template_id, due_time, instance_number, created_at, is_completed)
                         VALUES (?1, ?2, ?3, ?4, ?5, 0)",
                        rusqlite::params![
                            &instance_id,
                            &id,
                            &next.to_rfc3339(),
                            instance_number,
                            &now.to_rfc3339()
                        ]
                    ).map_err(|e| e.to_string())?;

                    // 同时创建提醒任务
                    let reminder_id = Uuid::new_v4().to_string();
                    conn.execute(
                        "INSERT INTO reminders
                         (id, title, description, priority, category_id, due_time,
                          reminder_function, is_completed, created_at, template_id)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, '完成时间提醒', 0, ?7, ?8)",
                        rusqlite::params![
                            &reminder_id,
                            &title,
                            &description,
                            priority,
                            category_id,
                            &next.to_rfc3339(),
                            &now.to_rfc3339(),
                            &id
                        ]
                    ).map_err(|e| e.to_string())?;

                    // 更新实例关联提醒ID
                    conn.execute(
                        "UPDATE recurring_instances SET reminder_id = ?1 WHERE id = ?2",
                        [&reminder_id, &instance_id]
                    ).map_err(|e| e.to_string())?;
                }

                current = next + Duration::seconds(1);
            } else {
                break;
            }
        }
    }

    Ok(())
}

fn calculate_next_occurrence_for_template(
    recurrence_type: &str,
    interval: i32,
    days: &[i32],
    after: &DateTime<Local>,
    time: NaiveTime,
) -> Option<DateTime<Local>> {
    calculate_nth_occurrence(recurrence_type, interval, days, after, 0, time)
}

fn should_end(
    conn: &rusqlite::Connection,
    template_id: &str,
    end_type: &str,
    end_count: Option<i32>,
    end_date: &Option<String>,
    now: &DateTime<Local>
) -> Result<bool, String> {
    match end_type {
        "count" => {
            let completed = get_completed_count(conn, template_id).map_err(|e| e.to_string())?;
            Ok(end_count.map(|c| completed >= c).unwrap_or(false))
        }
        "date" => {
            if let Some(date_str) = end_date {
                let end = DateTime::parse_from_rfc3339(date_str)
                    .map(|d| d.with_timezone(&Local))
                    .map_err(|e| e.to_string())?;
                Ok(now >= &end)
            } else {
                Ok(false)
            }
        }
        _ => Ok(false)
    }
}

fn get_next_instance_number(conn: &rusqlite::Connection, template_id: &str) -> Result<i32, String> {
    let max: i32 = conn.query_row(
        "SELECT COALESCE(MAX(instance_number), 0) FROM recurring_instances WHERE template_id = ?1",
        [template_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    Ok(max + 1)
}