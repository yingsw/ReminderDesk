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
        return Ok(())
    }

    SCHEDULER_RUNNING.store(true, Ordering::SeqCst);

    // 初始化已提醒ID集合
    unsafe {
        NOTIFIED_IDS = Some(HashSet::new());
    }

    // 启动时发送测试通知，确认通知功能正常
    println!("[Scheduler] 发送启动通知");
    let _ = app
        .notification()
        .builder()
        .title("任务提醒助手已启动")
        .body("通知功能已就绪，将按时提醒您的任务")
        .show();

    // 启动时立即生成一次循环任务
    if let Some(db) = app.try_state::<DbState>() {
        let _ = recurring::generate_upcoming_instances(&db);
    }

    let app_handle = app.clone();

    // 启动时立即检查一次
    println!("[Scheduler] 启动时立即检查任务");
    check_and_notify(&app_handle);

    // 使用标准线程而不是异步运行时
    std::thread::spawn(move || {
        let mut count = 0;
        loop {
            // 每10秒检查一次
            std::thread::sleep(Duration::from_secs(10));
            count += 1;
            println!("[Scheduler] 第{}次检查任务", count);
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
        None => {
            println!("[Scheduler] 无法获取数据库状态");
            return;
        }
    };

    let conn = match db.0.lock() {
        Ok(c) => c,
        Err(_) => {
            println!("[Scheduler] 无法锁定数据库连接");
            return;
        }
    };

    let Ok(mut stmt) = conn.prepare(
        "SELECT id, title, description, priority, category_id, due_time, reminder_function, is_completed, is_pinned, created_at, template_id FROM reminders WHERE is_completed = 0"
    ) else {
        println!("[Scheduler] 无法准备查询语句");
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
        println!("[Scheduler] 无法查询任务");
        return;
    };

    let now = chrono::Local::now();
    println!("[Scheduler] 当前时间: {}", now.format("%Y-%m-%d %H:%M:%S"));

    let reminders_vec: Vec<Reminder> = reminders.collect::<Result<Vec<_>, _>>().unwrap_or_default();
    println!("[Scheduler] 找到 {} 个未完成任务", reminders_vec.len());

    for reminder in reminders_vec {
        println!("[Scheduler] 检查任务: {} (提醒函数: {})", reminder.title, reminder.reminder_function);

        if let Some(reminder_time) = calculate_reminder_time(&reminder.due_time, &reminder.reminder_function) {
            let diff = reminder_time - now;
            let diff_secs = diff.num_seconds();

            println!("[Scheduler] 提醒时间: {}, 与当前时间差: {}秒",
                reminder_time.format("%Y-%m-%d %H:%M:%S"), diff_secs);

            // 扩大窗口：提醒时间在未来120秒到过去120秒之间
            // 10秒检测间隔 + 120秒窗口 = 绝对不会错过
            if diff_secs >= -120 && diff_secs <= 120 {
                println!("[Scheduler] 任务 {} 进入提醒窗口!", reminder.title);

                // 检查是否已经提醒过
                let should_notify = unsafe {
                    if let Some(ref mut notified) = NOTIFIED_IDS {
                        if notified.contains(&reminder.id) {
                            println!("[Scheduler] 任务 {} 已经提醒过，跳过", reminder.title);
                            false
                        } else {
                            notified.insert(reminder.id.clone());
                            println!("[Scheduler] 任务 {} 标记为需要提醒", reminder.title);
                            true
                        }
                    } else {
                        true
                    }
                };

                if should_notify {
                    println!("[Scheduler] 发送通知: {}", reminder.title);
                    send_notification(app, &reminder);
                }
            }
        } else {
            println!("[Scheduler] 无法计算提醒时间: {}", reminder.title);
        }
    }

    // 清理过期的已提醒ID
    unsafe {
        if let Some(ref mut notified) = NOTIFIED_IDS {
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

    let title = format!("⏰ {}", reminder.title);
    let body = format!("{} - {}", priority_text, reminder.description);

    println!("[Notification] 发送通知标题: {}", title);
    println!("[Notification] 发送通知内容: {}", body);

    let result = app
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show();

    match result {
        Ok(_) => println!("[Notification] 通知发送成功"),
        Err(e) => println!("[Notification] 通知发送失败: {}", e),
    }
}