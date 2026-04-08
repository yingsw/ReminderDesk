using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;
using System.Windows.Media;

namespace ReminderDesk.Converters
{
    /// <summary>
    /// 优先级颜色转换器
    /// </summary>
    public class PriorityColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is int priority)
            {
                return priority switch
                {
                    3 => new SolidColorBrush(Color.FromRgb(244, 63, 54)),   // 红色 - 紧急
                    2 => new SolidColorBrush(Color.FromRgb(249, 115, 22)),  // 橙色 - 高
                    1 => new SolidColorBrush(Color.FromRgb(59, 130, 246)),  // 蓝色 - 中
                    0 => new SolidColorBrush(Color.FromRgb(34, 197, 94)),   // 绿色 - 低
                    _ => new SolidColorBrush(Color.FromRgb(156, 163, 175)) // 灰色 - 默认
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
    /// 优先级背景颜色转换器
    /// </summary>
    public class PriorityBgColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is int priority)
            {
                return priority switch
                {
                    3 => new SolidColorBrush(Color.FromRgb(254, 202, 202)), // 红色
                    2 => new SolidColorBrush(Color.FromRgb(253, 186, 116)),// 橙色
                    1 => new SolidColorBrush(Color.FromRgb(191, 219, 254)),// 蓝色
                    0 => new SolidColorBrush(Color.FromRgb(187, 247, 208)), // 绿色
                    _ => new SolidColorBrush(Color.FromRgb(229, 231, 235)) // 灰色
                };
            }
            return new SolidColorBrush(Colors.LightGray);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// 完成状态文本颜色转换器
    /// </summary>
    public class CompletedTextConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool isCompleted && isCompleted)
            {
                return new SolidColorBrush(Color.FromRgb(156, 163, 175)); // 灰色 - 已完成
            }
            return new SolidColorBrush(Color.FromRgb(31, 41, 55)); // 深色 - 未完成
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// 紧急程度颜色转换器
    /// </summary>
    public class UrgencyColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool hasReminded && hasReminded)
            {
                return new SolidColorBrush(Color.FromRgb(34, 197, 94)); // 绿色 - 已提醒
            }

            // 检查是否快到期
            return new SolidColorBrush(Color.FromRgb(107, 114, 128)); // 默认灰色
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// 布尔值反转转换器
    /// </summary>
    public class InverseBooleanConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool boolValue)
            {
                return !boolValue;
            }
            return false;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool boolValue)
            {
                return !boolValue;
            }
            return false;
        }
    }

    /// <summary>
    /// 布尔值转可见性转换器
    /// </summary>
    public class BooleanToVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool boolValue)
            {
                return boolValue ? Visibility.Visible : Visibility.Collapsed;
            }
            return Visibility.Collapsed;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is Visibility visibility)
            {
                return visibility == Visibility.Visible;
            }
            return false;
        }
    }
}