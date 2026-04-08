using System;
using System.Collections.Generic;
using System.Windows;
using System.Windows.Threading;
using ReminderDesk.Models;
using Microsoft.Toolkit.Uwp.Notifications;

namespace ReminderDesk.Services
{
    /// <summary>
    /// 提醒检查服务 - 定时检查并弹出通知
    /// </summary>
    public class ReminderCheckService
    {
        private readonly DatabaseService _db;
        private readonly DispatcherTimer _timer;
        private List<ReminderItem> _reminders = new();

        public event Action<ReminderItem>? OnReminderTriggered;

        public ReminderCheckService(DatabaseService db)
        {
            _db = db;
            
            // 每30秒检查一次
            _timer = new DispatcherTimer
            {
                Interval = TimeSpan.FromSeconds(30)
            };
            _timer.Tick += Timer_Tick;
        }

        public void Start()
        {
            LoadReminders();
            _timer.Start();
            System.Diagnostics.Debug.WriteLine("[ReminderCheckService] 已启动");
        }

        public void Stop()
        {
            _timer.Stop();
        }

        public void LoadReminders()
        {
            _reminders = _db.LoadAllReminders();
        }

        private void Timer_Tick(object? sender, EventArgs e)
        {
            CheckReminders();
        }

        private void CheckReminders()
        {
            var now = DateTime.Now;

            foreach (var item in _reminders)
            {
                if (item.IsCompleted) continue;
                if (item.HasReminded) continue;
                if (item.ReminderTime > now) continue;

                // 触发提醒
                ShowNotification(item);
                item.HasReminded = true;
                _db.SaveReminder(item);

                OnReminderTriggered?.Invoke(item);
            }
        }

        private void ShowNotification(ReminderItem item)
        {
            try
            {
                var title = item.Priority switch
                {
                    3 => "紧急任务提醒 ⏰",
                    2 => "高优先级任务提醒 ⚠️",
                    _ => "任务提醒 📌"
                };

                new ToastContentBuilder()
                    .AddText(title)
                    .AddText(item.Description)
                    .AddText($"完成时间: {item.DueTime:MM/dd HH:mm}")
                    .AddButton(new ToastButton()
                        .SetContent("完成")
                        .AddArgument("action", "complete")
                        .AddArgument("id", item.Id.ToString()))
                    .AddButton(new ToastButton()
                        .SetContent("延期")
                        .AddArgument("action", "delay")
                        .AddArgument("id", item.Id.ToString()))
                    .AddButton(new ToastButton()
                        .SetContent("删除")
                        .AddArgument("action", "delete")
                        .AddArgument("id", item.Id.ToString()))
                    .Show();

                System.Diagnostics.Debug.WriteLine($"[ReminderCheckService] 已触发提醒: {item.Description}");
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[ReminderCheckService] 通知失败: {ex.Message}");
            }
        }

        /// <summary>
        /// 处理通知按钮点击
        /// </summary>
        public void HandleToastAction(string action, Guid id)
        {
            var item = _reminders.Find(r => r.Id == id);
            if (item == null) return;

            switch (action)
            {
                case "complete":
                    item.IsCompleted = true;
                    item.HasReminded = true;
                    _db.SaveReminder(item);
                    break;

                case "delay":
                    // 延期1小时
                    item.ReminderTime = item.ReminderTime.AddHours(1);
                    item.DueTime = item.DueTime.AddHours(1);
                    item.HasReminded = false;
                    _db.SaveReminder(item);
                    break;

                case "delete":
                    _db.DeleteReminder(id);
                    _reminders.Remove(item);
                    break;
            }
        }
    }
}