using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using ReminderDesk.Models;

namespace ReminderDesk.Services
{
    /// <summary>
    /// 定时调度服务
    /// </summary>
    public class SchedulerService : IDisposable
    {
        private readonly List<ReminderItem> _reminders;
        private readonly int _checkIntervalSeconds;
        private CancellationTokenSource? _cancellationToken;
        private bool _isRunning;

        public event EventHandler<ReminderItem>? ReminderTriggered;

        public SchedulerService(List<ReminderItem> reminders, int checkIntervalSeconds = 10)
        {
            _reminders = reminders ?? new List<ReminderItem>();
            _checkIntervalSeconds = checkIntervalSeconds;
        }

        public void Start()
        {
            if (_isRunning) return;

            _isRunning = true;
            _cancellationToken = new CancellationTokenSource();

            Task.Run(() => CheckLoop(_cancellationToken.Token), _cancellationToken.Token);
        }

        public void Stop()
        {
            _isRunning = false;
            _cancellationToken?.Cancel();
        }

        private async Task CheckLoop(CancellationToken token)
        {
            while (!token.IsCancellationRequested && _isRunning)
            {
                CheckReminders();
                await Task.Delay(_checkIntervalSeconds * 1000, token);
            }
        }

        private void CheckReminders()
        {
            var now = DateTime.Now;

            foreach (var reminder in _reminders)
            {
                if (!reminder.Schedule.IsEnabled) continue;
                if (reminder.IsCompleted) continue;

                // 更新下次提醒时间
                if (!reminder.NextReminderTime.HasValue || reminder.NextReminderTime.Value < now)
                {
                    reminder.UpdateNextReminderTime();
                }

                // 检查是否需要触发
                if (reminder.NextReminderTime.HasValue && reminder.NextReminderTime.Value <= now)
                {
                    TriggerReminder(reminder);
                }
            }
        }

        private void TriggerReminder(ReminderItem reminder)
        {
            ReminderTriggered?.Invoke(this, reminder);

            // 单次任务触发后禁用
            if (reminder.Schedule.Type == ScheduleType.Once)
            {
                reminder.Schedule.IsEnabled = false;
            }

            // 更新下次提醒时间
            reminder.UpdateNextReminderTime();
        }

        /// <summary>立即触发指定任务</summary>
        public void TriggerNow(ReminderItem reminder)
        {
            TriggerReminder(reminder);
        }

        public void Dispose()
        {
            Stop();
            _cancellationToken?.Dispose();
        }
    }
}