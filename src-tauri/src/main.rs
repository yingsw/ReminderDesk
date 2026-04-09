// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod reminder;
mod recurring;
mod scheduler;
mod settings;
mod tray;

use settings::WindowSettings;
use tauri::{AppHandle, Manager};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化数据库
            database::init_database(&app.handle())?;

            // 加载窗口设置
            let window_settings = settings::load_settings(&app.handle());

            // 应用窗口尺寸
            let width = window_settings.width.max(900);
            let height = window_settings.height.max(650);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                    width,
                    height,
                }));

                if window_settings.x >= 0 && window_settings.y >= 0 {
                    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: window_settings.x,
                        y: window_settings.y,
                    }));
                }
            }

            // 设置系统托盘
            tray::setup_tray(&app.handle())?;

            // 启动定时器
            scheduler::start_scheduler(&app.handle())?;

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    save_window_settings(&window.app_handle());
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            // 分类
            reminder::get_categories,
            reminder::add_category,
            reminder::update_category,
            reminder::delete_category,
            // 任务
            reminder::get_reminders,
            reminder::add_reminder,
            reminder::update_reminder,
            reminder::delete_reminder,
            reminder::complete_reminder,
            // 导入导出
            reminder::export_data,
            reminder::import_data,
            // 循环任务
            recurring::get_recurring_templates,
            recurring::add_recurring_template,
            recurring::update_recurring_template,
            recurring::delete_recurring_template,
            recurring::get_recurring_instances,
            recurring::complete_recurring_instance,
            recurring::preview_next_occurrences,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn save_window_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let (Ok(size), Ok(position)) = (window.inner_size(), window.outer_position()) {
            let width = size.width.max(900);
            let height = size.height.max(650);
            let settings = WindowSettings {
                width,
                height,
                x: position.x,
                y: position.y,
            };
            settings::save_settings(app, &settings);
        }
    }
}