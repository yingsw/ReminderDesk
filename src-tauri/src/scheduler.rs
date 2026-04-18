use crate::database::DbState;
use crate::reminder::{calculate_reminder_time, Reminder};
use crate::recurring;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;

static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);
static mut NOTIFIED_KEYS: Option<HashSet<String>> = None;

const CLOSE_SCHEME: &str = "tauri-ipc";
const CLOSE_HOST: &str = "close-reminder";

pub fn start_scheduler(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if SCHEDULER_RUNNING.load(Ordering::SeqCst) {
        return Ok(())
    }

    SCHEDULER_RUNNING.store(true, Ordering::SeqCst);

    unsafe {
        NOTIFIED_KEYS = Some(HashSet::new());
    }

    if let Some(db) = app.try_state::<DbState>() {
        let _ = recurring::generate_upcoming_instances(&db);
    }

    let app_handle = app.clone();

    check_and_notify(&app_handle);

    std::thread::spawn(move || {
        let mut count = 0;
        loop {
            std::thread::sleep(Duration::from_secs(10));
            count += 1;
            check_and_notify(&app_handle);

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
        Err(_) => {
            eprintln!("[scheduler] Failed to acquire DB lock, skipping check");
            return;
        }
    };

    let Ok(mut stmt) = conn.prepare(
        "SELECT id, title, description, priority, category_id, due_time, reminder_function, is_completed, is_pinned, created_at, template_id FROM reminders WHERE is_completed = 0"
    ) else {
        eprintln!("[scheduler] Failed to prepare statement");
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
        eprintln!("[scheduler] Failed to query reminders");
        return;
    };

    let now = chrono::Local::now();
    let reminders_vec: Vec<Reminder> = reminders.collect::<Result<Vec<_>, _>>().unwrap_or_default();

    for reminder in reminders_vec {
        if let Some(reminder_time) = calculate_reminder_time(&reminder.due_time, &reminder.reminder_function) {
            let diff = reminder_time - now;
            let diff_secs = diff.num_seconds();

            if diff_secs >= -10 && diff_secs <= 10 {
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
                    eprintln!("[scheduler] Triggering reminder for: {} (id={})", reminder.title, reminder.id);
                    send_notification(app, &reminder);
                }
            }
        }
    }

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

    let title = format!(" 任务提醒: {}", reminder.title);
    let body = format!("{} - {}", priority_text, reminder.description);

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

    show_reminder_window(app, reminder);
}

#[tauri::command]
pub fn close_reminder_window(window: tauri::Window) {
    let _ = window.close();
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
    let window_title = format!(" 任务提醒: {}", reminder.title);

    let app_clone = app.clone();
    let label_clone = label.clone();
    let window_title_clone = window_title.clone();
    let reminder_id = reminder.id.clone();

    // The IPC URL to intercept: when clicked, we navigate to this and block+close
    let close_url = Url::parse(&format!("{}://{}", CLOSE_SCHEME, CLOSE_HOST)).unwrap();
    let close_url_for_nav = close_url.clone();
    let app_for_nav = app_clone.clone();
    let label_for_nav = label_clone.clone();

    // Escape for JS string literal
    let title_escaped = reminder.title.replace('\\', "\\\\").replace('\'', "\\'").replace('"', "\\\"");
    let desc_escaped = reminder.description.replace('\\', "\\\\").replace('\'', "\\'").replace('"', "\\\"");

    let _ = app.run_on_main_thread(move || {
        if let Some(existing) = app_clone.get_webview_window(&label_clone) {
            let _ = existing.close();
        }

        // JS to inject BEFORE page load: waits for DOM, updates content, adds close handler
        let init_js = format!(
            r#"document.addEventListener('DOMContentLoaded',function(){{
                var t=document.getElementById('reminderTitle');
                var p=document.getElementById('reminderPriority');
                var d=document.getElementById('reminderDescription');
                var b=document.getElementById('closeBtn');
                if(t)t.textContent='{}';
                if(p)p.textContent='{}';
                if(d)d.textContent='{}';
                if(b)b.onclick=function(){{location.href='{}://{}';}};
            }});"#,
            title_escaped, priority_text, desc_escaped, CLOSE_SCHEME, CLOSE_HOST
        );

        let result = WebviewWindowBuilder::new(
            &app_clone,
            &label_clone,
            WebviewUrl::App("reminder.html".into())
        )
        .title(&window_title_clone)
        .inner_size(400.0, 250.0)
        .resizable(false)
        .decorations(true)
        .always_on_top(true)
        .skip_taskbar(false)
        .center()
        .initialization_script(&init_js)
        .on_navigation(move |url| {
            if *url == close_url_for_nav {
                eprintln!("[scheduler] Intercepted close navigation");
                if let Some(win) = app_for_nav.get_webview_window(&label_for_nav) {
                    let _ = win.close();
                }
                false
            } else {
                true
            }
        })
        .build();

        match result {
            Ok(_win) => {
                eprintln!("[scheduler] Reminder window created for reminder {}", reminder_id);
            }
            Err(e) => {
                eprintln!("[scheduler] Failed to create reminder window for reminder {}: {}", reminder_id, e);
            }
        }
    });
}
