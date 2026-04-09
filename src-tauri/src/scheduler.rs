use crate::database::DbState;
use crate::reminder::{calculate_reminder_time, Reminder};
use crate::recurring;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn start_scheduler(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if SCHEDULER_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    SCHEDULER_RUNNING.store(true, Ordering::SeqCst);

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
        "SELECT id, title, description, priority, category_id, due_time, reminder_function, is_completed, created_at, template_id FROM reminders WHERE is_completed = 0"
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
            created_at: row.get(8)?,
            template_id: row.get(9)?,
        })
    }) else {
        return;
    };

    let now = chrono::Local::now();

    for reminder_result in reminders {
        if let Ok(reminder) = reminder_result {
            if let Some(reminder_time) = calculate_reminder_time(&reminder.due_time, &reminder.reminder_function) {
                let diff: chrono::Duration = reminder_time - now;
                if diff.num_seconds().abs() <= 30 {
                    send_notification(app, &reminder);
                }
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