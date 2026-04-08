using System;
using System.Collections.Generic;
using Cronos;

namespace ReminderDesk.Models
{
    /// <summary>
    /// 定时配置帮助类
    /// </summary>
    public static class ScheduleHelper
    {
        /// <summary>Cron表达式示例</summary>
        public static readonly Dictionary<string, string> CronExamples = new()
        {
            { "工作日9:30", "0 30 9 * * 1-5" },
            { "每天18:00", "0 0 18 * * *" },
            { "每周一9:00", "0 0 9 * * 1" },
            { "每月1日10:00", "0 0 10 1 * *" },
            { "每小时", "0 0 * * * *" },
            { "每30分钟", "0 */30 * * * *" }
        };

        /// <summary>获取Cron示例列表</summary>
        public static List<string> GetCronExamples()
        {
            var list = new List<string>();
            foreach (var kvp in CronExamples)
            {
                list.Add($"{kvp.Key}: {kvp.Value}");
            }
            return list;
        }

        /// <summary>创建单次定时</summary>
        public static ScheduleConfig CreateOnce(DateTime time)
        {
            return new ScheduleConfig
            {
                Type = ScheduleType.Once,
                ScheduledTime = time,
                IsEnabled = true
            };
        }

        /// <summary>创建每日定时</summary>
        public static ScheduleConfig CreateDaily(int hour, int minute)
        {
            return new ScheduleConfig
            {
                Type = ScheduleType.Daily,
                Hour = hour,
                Minute = minute,
                IsEnabled = true
            };
        }

        /// <summary>创建每周定时</summary>
        public static ScheduleConfig CreateWeekly(string weekDays, int hour, int minute)
        {
            return new ScheduleConfig
            {
                Type = ScheduleType.Weekly,
                WeekDays = weekDays,
                Hour = hour,
                Minute = minute,
                IsEnabled = true
            };
        }

        /// <summary>创建每月定时</summary>
        public static ScheduleConfig CreateMonthly(string monthDays, int hour, int minute)
        {
            return new ScheduleConfig
            {
                Type = ScheduleType.Monthly,
                MonthDays = monthDays,
                Hour = hour,
                Minute = minute,
                IsEnabled = true
            };
        }

        /// <summary>创建Cron定时</summary>
        public static ScheduleConfig CreateCron(string cronExpression)
        {
            return new ScheduleConfig
            {
                Type = ScheduleType.CronExpression,
                CronExpression = cronExpression,
                IsEnabled = true
            };
        }

        /// <summary>验证Cron表达式</summary>
        public static bool ValidateCronExpression(string expression)
        {
            try
            {
                CronExpression.Parse(expression);
                return true;
            }
            catch
            {
                return false;
            }
        }

        /// <summary>计算下次提醒时间</summary>
        public static DateTime? CalculateNextReminderTime(ScheduleConfig schedule, DateTime? lastReminderTime = null)
        {
            if (!schedule.IsEnabled) return null;

            var now = DateTime.Now;
            var referenceTime = lastReminderTime ?? now;

            try
            {
                switch (schedule.Type)
                {
                    case ScheduleType.Once:
                        if (schedule.ScheduledTime.HasValue && schedule.ScheduledTime.Value > now)
                            return schedule.ScheduledTime.Value;
                        return null;

                    case ScheduleType.Daily:
                        return GetNextDailyTime(schedule.Hour, schedule.Minute, referenceTime);

                    case ScheduleType.Weekly:
                        return GetNextWeeklyTime(schedule.WeekDays, schedule.Hour, schedule.Minute, referenceTime);

                    case ScheduleType.Monthly:
                        return GetNextMonthlyTime(schedule.MonthDays, schedule.Hour, schedule.Minute, referenceTime);

                    case ScheduleType.CronExpression:
                        return GetNextCronTime(schedule.CronExpression, referenceTime);

                    default:
                        return null;
                }
            }
            catch
            {
                return null;
            }
        }

        private static DateTime GetNextDailyTime(int hour, int minute, DateTime reference)
        {
            var todayTarget = reference.Date.AddHours(hour).AddMinutes(minute);
            if (todayTarget > reference)
                return todayTarget;
            return todayTarget.AddDays(1);
        }

        private static DateTime GetNextWeeklyTime(string weekDays, int hour, int minute, DateTime reference)
        {
            var days = weekDays.Split(',');
            var validDays = new HashSet<int>();
            foreach (var d in days)
            {
                if (int.TryParse(d.Trim(), out int dayIndex))
                    validDays.Add(dayIndex);
            }

            if (validDays.Count == 0) return reference.AddDays(1);

            var current = reference;
            for (int i = 0; i <= 7; i++)
            {
                var checkDate = current.Date.AddDays(i).AddHours(hour).AddMinutes(minute);
                if (validDays.Contains((int)checkDate.DayOfWeek) && checkDate > reference)
                    return checkDate;
            }

            return reference.AddDays(7);
        }

        private static DateTime GetNextMonthlyTime(string monthDays, int hour, int minute, DateTime reference)
        {
            var days = monthDays.Split(',');
            var validDays = new HashSet<int>();
            foreach (var d in days)
            {
                if (int.TryParse(d.Trim(), out int dayIndex) && dayIndex >= 1 && dayIndex <= 31)
                    validDays.Add(dayIndex);
            }

            if (validDays.Count == 0) return reference.AddDays(1);

            // 检查本月剩余日期
            for (int day = reference.Day + 1; day <= 31; day++)
            {
                if (validDays.Contains(day))
                {
                    try
                    {
                        var checkDate = new DateTime(reference.Year, reference.Month, day, hour, minute, 0);
                        if (checkDate > reference) return checkDate;
                    }
                    catch { } // 日期无效（如2月30日）
                }
            }

            // 检查下月
            var nextMonth = reference.AddMonths(1);
            for (int day = 1; day <= 31; day++)
            {
                if (validDays.Contains(day))
                {
                    try
                    {
                        return new DateTime(nextMonth.Year, nextMonth.Month, day, hour, minute, 0);
                    }
                    catch { }
                }
            }

            return reference.AddDays(30);
        }

        private static DateTime? GetNextCronTime(string cronExpression, DateTime reference)
        {
            try
            {
                var cron = CronExpression.Parse(cronExpression);
                var next = cron.GetNextOccurrence(reference, TimeZoneInfo.Local);
                return next;
            }
            catch
            {
                return null;
            }
        }
    }
}