use crate::database::DbState;
use crate::reminder::{calculate_reminder_time, Reminder};
use crate::recurring;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;

static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);
static mut NOTIFIED_KEYS: Option<HashSet<String>> = None;

pub fn start_scheduler(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if SCHEDULER_RUNNING.load(Ordering::SeqCst) {
        return Ok(())
    }

    SCHEDULER_RUNNING.store(true, Ordering::SeqCst);

    // 初始化已提醒ID集合
    unsafe {
        NOTIFIED_KEYS = Some(HashSet::new());
    }

    // 启动时立即生成一次循环任务
    if let Some(db) = app.try_state::<DbState>() {
        let _ = recurring::generate_upcoming_instances(&db);
    }

    let app_handle = app.clone();

    // 启动时立即检查一次
    check_and_notify(&app_handle);

    // 使用标准线程运行调度器
    std::thread::spawn(move || {
        let mut count = 0;
        loop {
            std::thread::sleep(Duration::from_secs(10));
            count += 1;
            check_and_notify(&app_handle);

            // 每分钟检查是否需要生成新的循环任务实例
            if count % 6 == 0 {
                if let Some(db) = app_handle.try_state::<DbState>() {
                    let _ = recurring::generate_upcoming_instances(&db);
                }
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
    let reminders_vec: Vec<Reminder> = reminders.collect::<Result<Vec<_>, _>>().unwrap_or_default();

    for reminder in reminders_vec {
        if let Some(reminder_time) = calculate_reminder_time(&reminder.due_time, &reminder.reminder_function) {
            let diff = reminder_time - now;
            let diff_secs = diff.num_seconds();

            // 精确提醒窗口：提醒时间前后10秒内触发
            if diff_secs >= -10 && diff_secs <= 10 {
                // 使用 ID + 提醒时间 作为唯一键
                let notification_key = format!("{}_{}", reminder.id, reminder_time.format("%Y%m%d%H%M"));

                let should_notify = unsafe {
                    if let Some(ref mut notified) = NOTIFIED_KEYS {
                        if notified.contains(&notification_key) {
                            false
                        } else {
                            notified.insert(notification_key.clone());
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

    // 清理过期的已提醒键
    unsafe {
        if let Some(ref mut notified) = NOTIFIED_KEYS {
            if notified.len() > 500 {
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

    let title = format!("⏰ {}", reminder.title);
    let body = format!("{} - {}", priority_text, reminder.description);

    // 发送系统通知
    let app_clone = app.clone();
    let title_clone = title.clone();
    let body_clone = body.clone();

    let _ = app.run_on_main_thread(move || {
        let _ = app_clone
            .notification()
            .builder()
            .title(&title_clone)
            .body(&body_clone)
            .show();
    });

    // 同时显示持久化的提醒窗口
    show_reminder_window(app, reminder);
}

fn show_reminder_window(app: &AppHandle, reminder: &Reminder) {
    let priority_text = match reminder.priority {
        0 => "低优先级",
        1 => "中优先级",
        2 => "高优先级",
        3 => "紧急",
        _ => "",
    };

    let label = format!("reminder_{}", reminder.id);
    let title = format!("⏰ 任务提醒: {}", reminder.title);
    let html_content = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: 'Microsoft YaHei', sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            margin: 0;
            padding: 20px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
        }}
        .container {{
            text-align: center;
            max-width: 400px;
        }}
        .title {{
            font-size: 24px;
            font-weight: bold;
            margin-bottom: 15px;
        }}
        .priority {{
            font-size: 14px;
            color: #ffeb3b;
            margin-bottom: 10px;
        }}
        .description {{
            font-size: 16px;
            margin-bottom: 20px;
            padding: 10px;
            background: rgba(255,255,255,0.1);
            border-radius: 8px;
        }}
        .btn {{
            padding: 12px 30px;
            font-size: 16px;
            background: #4caf50;
            color: white;
            border: none;
            border-radius: 25px;
            cursor: pointer;
            transition: all 0.3s;
        }}
        .btn:hover {{
            background: #45a049;
            transform: scale(1.05);
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="title">⏰ {}</div>
        <div class="priority">{}</div>
        <div class="description">{}</div>
        <button class="btn" onclick="window.close()">关闭提醒</button>
    </div>
</body>
</html>"#,
        reminder.title, priority_text, reminder.description
    );

    let app_clone = app.clone();
    let label_clone = label.clone();
    let title_clone = title.clone();
    let data_url = format!("data:text/html;charset=utf-8,{}", urlencoding::encode(&html_content));

    let _ = app.run_on_main_thread(move || {
        let _ = WebviewWindowBuilder::new(
            &app_clone,
            &label_clone,
            WebviewUrl::External(data_url.parse().unwrap())
        )
        .title(&title_clone)
        .inner_size(400.0, 250.0)
        .resizable(false)
        .decorations(true)
        .always_on_top(true)
        .center()
        .build();
    });
}