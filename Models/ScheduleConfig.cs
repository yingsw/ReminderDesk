using System;

namespace ReminderDesk.Models
{
    /// <summary>
    /// 定时类型
    /// </summary>
    public enum ScheduleType
    {
        /// <summary>单次提醒</summary>
        Once,
        /// <summary>每日提醒</summary>
        Daily,
        /// <summary>每周提醒</summary>
        Weekly,
        /// <summary>每月提醒</summary>
        Monthly,
        /// <summary>Cron表达式</summary>
        CronExpression
    }

    /// <summary>
    /// 定时配置
    /// </summary>
    public class ScheduleConfig
    {
        /// <summary>定时类型</summary>
        public ScheduleType Type { get; set; } = ScheduleType.Once;

        /// <summary>是否启用</summary>
        public bool IsEnabled { get; set; } = true;

        /// <summary>定时时间（单次使用）</summary>
        public DateTime? ScheduledTime { get; set; }

        /// <summary>小时（每日/每周/每月使用）</summary>
        public int Hour { get; set; } = 9;

        /// <summary>分钟</summary>
        public int Minute { get; set; } = 0;

        /// <summary>星期几（每周使用，逗号分隔，0=周日）</summary>
        public string WeekDays { get; set; } = "1";

        /// <summary>月日期（每月使用，逗号分隔）</summary>
        public string MonthDays { get; set; } = "1";

        /// <summary>Cron表达式</summary>
        public string CronExpression { get; set; } = "";

        /// <summary>获取定时描述</summary>
        public string GetDescription()
        {
            return Type switch
            {
                ScheduleType.Once => $"单次: {ScheduledTime?.ToString("yyyy-MM-dd HH:mm") ?? "未设置"}",
                ScheduleType.Daily => $"每日 {Hour:00}:{Minute:00}",
                ScheduleType.Weekly => $"每周 {GetWeekDayNames()} {Hour:00}:{Minute:00}",
                ScheduleType.Monthly => $"每月 {MonthDays}日 {Hour:00}:{Minute:00}",
                ScheduleType.CronExpression => $"Cron: {CronExpression}",
                _ => "未知"
            };
        }

        private string GetWeekDayNames()
        {
            var names = new[] { "周日", "周一", "周二", "周三", "周四", "周五", "周六" };
            var days = WeekDays.Split(',');
            var result = new System.Text.StringBuilder();
            foreach (var d in days)
            {
                if (int.TryParse(d.Trim(), out int dayIndex) && dayIndex >= 0 && dayIndex <= 6)
                {
                    if (result.Length > 0) result.Append(",");
                    result.Append(names[dayIndex]);
                }
            }
            return result.ToString();
        }
    }
}