using System;
using System.Collections.Generic;

namespace ReminderDesk.Models
{
    /// <summary>
    /// 提醒任务
    /// </summary>
    public class ReminderItem
    {
        public Guid Id { get; set; } = Guid.NewGuid();

        /// <summary>任务标题</summary>
        public string Title { get; set; } = string.Empty;

        public string Description { get; set; } = string.Empty;
        public int Priority { get; set; } = 1;
        public DateTime DueTime { get; set; } = DateTime.Today.AddHours(18);
        public DateTime ReminderTime { get; set; }
        public bool IsCompleted { get; set; }
        public bool HasReminded { get; set; }
        public DateTime CreatedTime { get; set; } = DateTime.Now;

        /// <summary>提醒类型: 0=内置函数, 1=自定义函数</summary>
        public int ReminderType { get; set; } = 0;

        /// <summary>内置函数名或自定义函数表达式</summary>
        public string ReminderFunction { get; set; } = string.Empty;

        /// <summary>完成时间默认小时</summary>
        public static int DefaultDueHour { get; set; } = 18;
        public static int DefaultDueMinute { get; set; } = 0;

        /// <summary>定时配置（MVVM版本使用）</summary>
        public ScheduleConfig Schedule { get; set; } = new ScheduleConfig();

        /// <summary>下次提醒时间</summary>
        public DateTime? NextReminderTime { get; set; }

        /// <summary>
        /// 计算提醒时间
        /// </summary>
        public void CalculateReminderTime()
        {
            ReminderTime = ReminderType == 1 
                ? CalculateCustomFunction(ReminderFunction)
                : CalculateBuiltInFunction(ReminderFunction);
        }

        /// <summary>
        /// 内置提醒函数计算
        /// </summary>
        private DateTime CalculateBuiltInFunction(string functionName)
        {
            return functionName switch
            {
                // 提前提醒
                "完成时间提醒" => DueTime,
                "提前5分钟" => DueTime.AddMinutes(-5),
                "提前10分钟" => DueTime.AddMinutes(-10),
                "提前15分钟" => DueTime.AddMinutes(-15),
                "提前20分钟" => DueTime.AddMinutes(-20),
                "提前30分钟" => DueTime.AddMinutes(-30),
                "提前45分钟" => DueTime.AddMinutes(-45),
                "提前1小时" => DueTime.AddHours(-1),
                "提前2小时" => DueTime.AddHours(-2),
                "提前3小时" => DueTime.AddHours(-3),
                "提前6小时" => DueTime.AddHours(-6),
                "提前12小时" => DueTime.AddHours(-12),
                "提前1天" => DueTime.AddDays(-1),
                "提前2天" => DueTime.AddDays(-2),
                "提前3天" => DueTime.AddDays(-3),
                "提前1周" => DueTime.AddDays(-7),
                
                // 当天特定时间
                "当天早上6点" => DueTime.Date.AddHours(6),
                "当天早上7点" => DueTime.Date.AddHours(7),
                "当天早上8点" => DueTime.Date.AddHours(8),
                "当天早上9点" => DueTime.Date.AddHours(9),
                "当天早上10点" => DueTime.Date.AddHours(10),
                "当天中午12点" => DueTime.Date.AddHours(12),
                "当天中午13点" => DueTime.Date.AddHours(13),
                "当天傍晚17点" => DueTime.Date.AddHours(17),
                "当天傍晚18点" => DueTime.Date.AddHours(18),
                "当天傍晚19点" => DueTime.Date.AddHours(19),
                "当天晚上20点" => DueTime.Date.AddHours(20),
                "当天晚上21点" => DueTime.Date.AddHours(21),
                
                // 隔天提醒
                "第二天早上9点" => DueTime.Date.AddDays(1).AddHours(9),
                "第二天早上8点" => DueTime.Date.AddDays(1).AddHours(8),
                
                // 每周同一天
                "每周同一天早上9点" => GetWeeklyReminder(9),
                
                _ => DueTime
            };
        }

        /// <summary>
        /// 自定义函数计算
        /// 支持格式：
        /// - DueTime-1h (完成时间前1小时)
        /// - DueTime+1h (完成时间后1小时)
        /// - DueTime-30m (完成时间前30分钟)
        /// - Date+9h (当天9点)
        /// - Date+12h (当天12点)
        /// - NextWorkday+9h (下一个工作日9点)
        /// - NextMonday+9h (下周一9点)
        /// - Tomorrow+9h (明天9点)
        /// </summary>
        private DateTime CalculateCustomFunction(string expression)
        {
            if (string.IsNullOrWhiteSpace(expression)) return DueTime;

            try
            {
                expression = expression.Trim().ToLower();

                // DueTime 计算
                if (expression.StartsWith("duetime"))
                {
                    return ParseDueTimeOffset(expression.Substring(7).Trim());
                }
                
                // Date 计算（当天）
                if (expression.StartsWith("date"))
                {
                    return ParseDateOffset(expression.Substring(4).Trim());
                }
                
                // Tomorrow 计算（明天）
                if (expression.StartsWith("tomorrow"))
                {
                    return DueTime.Date.AddDays(1).AddHours(ParseHour(expression.Substring(8).Trim()));
                }
                
                // NextWorkday 计算（下个工作日）
                if (expression.StartsWith("nextworkday"))
                {
                    return GetNextWorkday().AddHours(ParseHour(expression.Substring(11).Trim()));
                }
                
                // NextMonday 等计算
                if (expression.StartsWith("next"))
                {
                    return ParseNextDay(expression.Substring(4).Trim());
                }
                
                // 默认当作小时计算
                if (int.TryParse(expression.Replace("h", "").Replace("H", ""), out int hours))
                {
                    return DueTime.AddHours(-hours);
                }
            }
            catch { }

            return DueTime;
        }

        private DateTime ParseDueTimeOffset(string offset)
        {
            if (offset.StartsWith("-"))
            {
                var value = offset.Substring(1);
                if (value.EndsWith("m") || value.EndsWith("M"))
                {
                    return DueTime.AddMinutes(-int.Parse(value.Substring(0, value.Length - 1)));
                }
                if (value.EndsWith("h") || value.EndsWith("H"))
                {
                    return DueTime.AddHours(-int.Parse(value.Substring(0, value.Length - 1)));
                }
                if (value.EndsWith("d") || value.EndsWith("D"))
                {
                    return DueTime.AddDays(-int.Parse(value.Substring(0, value.Length - 1)));
                }
            }
            else if (offset.StartsWith("+"))
            {
                var value = offset.Substring(1);
                if (value.EndsWith("m") || value.EndsWith("M"))
                {
                    return DueTime.AddMinutes(int.Parse(value.Substring(0, value.Length - 1)));
                }
                if (value.EndsWith("h") || value.EndsWith("H"))
                {
                    return DueTime.AddHours(int.Parse(value.Substring(0, value.Length - 1)));
                }
                if (value.EndsWith("d") || value.EndsWith("D"))
                {
                    return DueTime.AddDays(int.Parse(value.Substring(0, value.Length - 1)));
                }
            }
            return DueTime;
        }

        private DateTime ParseDateOffset(string offset)
        {
            var hour = ParseHour(offset);
            return DueTime.Date.AddHours(hour);
        }

        private int ParseHour(string timeStr)
        {
            timeStr = timeStr.Replace("h", "").Replace("H", "").Trim();
            if (int.TryParse(timeStr, out int hour)) return hour;
            return 9;
        }

        private DateTime GetNextWorkday()
        {
            var date = DueTime.Date.AddDays(1);
            while (date.DayOfWeek == DayOfWeek.Saturday || date.DayOfWeek == DayOfWeek.Sunday)
            {
                date = date.AddDays(1);
            }
            return date;
        }

        private DateTime GetWeeklyReminder(int hour)
        {
            var date = DueTime.Date.AddDays(1);
            while ((int)date.DayOfWeek != (int)DueTime.DayOfWeek)
            {
                date = date.AddDays(1);
            }
            return date.AddHours(hour);
        }

        private DateTime ParseNextDay(string dayAndTime)
        {
            // nextmonday+9h, nextfriday+10h 等
            var dayNames = new Dictionary<string, DayOfWeek>
            {
                {"monday", DayOfWeek.Monday},
                {"tuesday", DayOfWeek.Tuesday},
                {"wednesday", DayOfWeek.Wednesday},
                {"thursday", DayOfWeek.Thursday},
                {"friday", DayOfWeek.Friday},
                {"saturday", DayOfWeek.Saturday},
                {"sunday", DayOfWeek.Sunday}
            };

            foreach (var kvp in dayNames)
            {
                if (dayAndTime.StartsWith(kvp.Key))
                {
                    var timeStr = dayAndTime.Substring(kvp.Key.Length);
                    var hour = ParseHour(timeStr);
                    var date = DueTime.Date.AddDays(1);
                    while (date.DayOfWeek != kvp.Value)
                    {
                        date = date.AddDays(1);
                    }
                    return date.AddHours(hour);
                }
            }

            return DueTime;
        }

        public string PriorityText => Priority switch
        {
            0 => "低",
            1 => "中",
            2 => "高",
            3 => "紧急",
            _ => "中"
        };

        public string PriorityColor => Priority switch
        {
            0 => "#4CAF50",
            1 => "#2196F3",
            2 => "#FF9800",
            3 => "#F44336",
            _ => "#9E9E9E"
        };

        public string RemainingText
        {
            get
            {
                if (IsCompleted) return "已完成";
                var remaining = DueTime - DateTime.Now;
                if (remaining.TotalSeconds < 0) return "已过期";
                if (remaining.TotalMinutes < 60) return $"{(int)remaining.TotalMinutes}分钟后";
                if (remaining.TotalHours < 24) return $"{(int)remaining.TotalHours}小时后";
                return $"{(int)remaining.TotalDays}天后";
            }
        }

        /// <summary>更新下次提醒时间</summary>
        public void UpdateNextReminderTime()
        {
            NextReminderTime = ScheduleHelper.CalculateNextReminderTime(Schedule, NextReminderTime);
        }

        /// <summary>获取定时描述</summary>
        public string GetScheduleDescription()
        {
            return Schedule.GetDescription();
        }

        /// <summary>获取定时类型文本</summary>
        public string GetScheduleTypeText()
        {
            return Schedule.Type switch
            {
                ScheduleType.Once => "单次",
                ScheduleType.Daily => "每日",
                ScheduleType.Weekly => "每周",
                ScheduleType.Monthly => "每月",
                ScheduleType.CronExpression => "Cron",
                _ => "未知"
            };
        }
    }
}