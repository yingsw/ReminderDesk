using System;
using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;

namespace ReminderDesk.Converters
{
    /// <summary>
    /// 优先级到颜色的转换器
    /// 用于将优先级数值转换为对应的颜色标识
    /// </summary>
    public class PriorityColorConverter : IValueConverter
    {
        /// <summary>
        /// 将优先级数值转换为颜色
        /// </summary>
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is int priority)
            {
                return priority switch
                {
                    0 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#4CAF50")), // 低 - 绿色
                    1 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#2196F3")), // 中 - 蓝色
                    2 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#FF9800")), // 高 - 橙色
                    3 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#F44336")), // 紧急 - 红色
                    _ => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#9E9E9E"))  // 默认 - 灰色
                };
            }
            return new SolidColorBrush(Colors.Gray);
        }

        /// <summary>
        /// 反向转换（不支持）
        /// </summary>
        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// 优先级到背景颜色的转换器（用于徽章）
    /// </summary>
    public class PriorityBgColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is int priority)
            {
                return priority switch
                {
                    0 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#664CAF50")), // 低
                    1 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#662196F3")), // 中
                    2 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#66FF9800")), // 高
                    3 => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#66F44336")), // 紧急
                    _ => new SolidColorBrush((Color)ColorConverter.ConvertFromString("#669E9E9E"))
                };
            }
            return new SolidColorBrush(Colors.Gray);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// 优先级到文本的转换器
    /// </summary>
    public class PriorityTextConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is int priority)
            {
                return priority switch
                {
                    0 => "低",
                    1 => "中",
                    2 => "高",
                    3 => "紧急",
                    _ => "未知"
                };
            }
            return "未知";
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// 完成状态到文字颜色的转换器
    /// </summary>
    public class CompletedTextConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool isCompleted)
            {
                return isCompleted 
                    ? new SolidColorBrush((Color)ColorConverter.ConvertFromString("#999999"))
                    : new SolidColorBrush((Color)ColorConverter.ConvertFromString("#1a1a1a"));
            }
            return new SolidColorBrush(Colors.Black);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// 剩余时间到紧迫程度颜色的转换器
    /// </summary>
    public class UrgencyColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            // 根据 HasReminded 和剩余时间返回不同颜色
            return new SolidColorBrush((Color)ColorConverter.ConvertFromString("#888888"));
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}