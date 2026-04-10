use crate::database::DbState;
use crate::reminder::{calculate_reminder_time, Reminder};
use crate::recurring;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);
static mut NOTIFIED_IDS: Option<HashSet<String>> = None;

pub fn start_scheduler(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if SCHEDULER_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    SCHEDULER_RUNNING.store(true, Ordering::SeqCst);

    // 初始化已提醒ID集合
    unsafe {
        NOTIFIED_IDS = Some(HashSet::new());
    }

    // 启动时立即生成一次循环任务
    if let Some(db) = app.try_state::<DbState>() {
        let _ = recurring::generate_upcoming_instances(&db);
    }

    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            check_and_notify(&app_handle);

            // 每分钟检查是否需要生成新的循环任务实例
            if let Some(db) = app_handle.try_state::<DbState>() {
                let _ = recurring::generate_upcoming_instances(&db);
            }
        }
    });

    Ok(())
}

fn check_and_notify(app: &AppHandle) {
    let db = match app.try_state::<DbState>() {
        Some(db) => db,
        None => return,
    };

    let conn = match db.0.lock() {
        Ok(c) => c,
        Err(_) => return,
    };

    let Ok(mut stmt) = conn.prepare(
        "SELECT id, title, description, priority, category_id, due_time, reminder_function, is_completed, is_pinned, created_at, template_id FROM reminders WHERE is_completed = 0"
    ) else {
        return;
    };

    let Ok(reminders) = stmt.query_map([], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            priority: row.get(3)?,
            category_id: row.get(4)?,
            category_name: None,
            category_color: None,
            due_time: row.get(5)?,
            reminder_function: row.get(6)?,
            is_completed: row.get::<_, i32>(7)? != 0,
            is_pinned: row.get::<_, i32>(8)? != 0,
            created_at: row.get(9)?,
            template_id: row.get(10)?,
        })
    }) else {
        return;
    };

    let now = chrono::Local::now();

    for reminder_result in reminders {
        if let Ok(reminder) = reminder_result {
            if let Some(reminder_time) = calculate_reminder_time(&reminder.due_time, &reminder.reminder_function) {
                // 检查提醒时间是否已经到达（在过去60秒内或即将在30秒内）
                let diff = reminder_time - now;
                let diff_secs = diff.num_seconds();

                // 提醒时间在过去60秒到未来30秒之间
                if diff_secs >= -60 && diff_secs <= 30 {
                    // 检查是否已经提醒过
                    let should_notify = unsafe {
                        if let Some(ref mut notified) = NOTIFIED_IDS {
                            if notified.contains(&reminder.id) {
                                false
                            } else {
                                notified.insert(reminder.id.clone());
                                true
                            }
                        } else {
                            true
                        }
                    };

                    if should_notify {
                        send_notification(app, &reminder);
                    }
                }
            }
        }
    }

    // 清理过期的已提醒ID（超过1小时的）
    unsafe {
        if let Some(ref mut notified) = NOTIFIED_IDS {
            // 简单清理：每隔一段时间清空集合，防止内存增长
            if notified.len() > 1000 {
                notified.clear();
            }
        }
    }
}

fn send_notification(app: &AppHandle, reminder: &Reminder) {
    let priority_text = match reminder.priority {
        0 => "低优先级",
        1 => "中优先级",
        2 => "高优先级",
        3 => "紧急",
        _ => "",
    };

    let _ = app
        .notification()
        .builder()
        .title(format!("⏰ {}", reminder.title))
        .body(format!("{} - {}", priority_text, reminder.description))
        .show();
}