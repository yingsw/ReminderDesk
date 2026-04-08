using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using ReminderDesk.Models;
using ReminderDesk.Services;

namespace ReminderDesk
{
    public partial class MainWindow : Window, INotifyPropertyChanged
    {
        private readonly DatabaseService _db;
        private readonly ReminderCheckService _reminderCheck;
        private ObservableCollection<ReminderItem> _reminders = new();
        private ReminderItem? _selectedReminder;
        private string _newDescription = "";
        private int _newPriority = 1;
        private DateTime _newDueDate = DateTime.Today.AddDays(1);
        private int _newDueTimeIndex = 8; // 默认18:00
        private int _newReminderFunctionIndex = 0;
        private string _reminderPreview = "";
        private bool _isBuiltInReminder = true;
        private bool _isCustomReminder = false;
        private string _customFunctionExpression = "DueTime-1h";

        private readonly string[] BuiltInFunctions = new[]
        {
            "完成时间提醒", "提前5分钟", "提前10分钟", "提前15分钟", "提前20分钟",
            "提前30分钟", "提前45分钟", "提前1小时", "提前2小时", "提前3小时",
            "提前6小时", "提前12小时", "提前1天", "提前2天", "提前3天", "提前1周",
            "当天早上7点", "当天早上8点", "当天早上9点", "当天早上10点",
            "当天中午12点", "当天中午13点", "当天傍晚17点", "当天傍晚18点", "当天傍晚19点",
            "当天晚上20点", "当天晚上21点", "第二天早上8点", "第二天早上9点"
        };

        private readonly int[] DueTimeHours = { 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22 };

        public event PropertyChangedEventHandler? PropertyChanged;

        public ObservableCollection<ReminderItem> Reminders
        {
            get => _reminders;
            set { _reminders = value; OnPropertyChanged(); }
        }

        public ReminderItem? SelectedReminder
        {
            get => _selectedReminder;
            set { _selectedReminder = value; OnPropertyChanged(); }
        }

        public string NewDescription
        {
            get => _newDescription;
            set { _newDescription = value; OnPropertyChanged(); UpdateReminderPreview(); }
        }

        public int NewPriority
        {
            get => _newPriority;
            set { _newPriority = value; OnPropertyChanged(); }
        }

        public DateTime NewDueDate
        {
            get => _newDueDate;
            set { _newDueDate = value; OnPropertyChanged(); UpdateReminderPreview(); }
        }

        public int NewDueTimeIndex
        {
            get => _newDueTimeIndex;
            set { _newDueTimeIndex = value; OnPropertyChanged(); UpdateReminderPreview(); }
        }

        public int NewReminderFunctionIndex
        {
            get => _newReminderFunctionIndex;
            set { _newReminderFunctionIndex = value; OnPropertyChanged(); UpdateReminderPreview(); }
        }

        public string ReminderPreview
        {
            get => _reminderPreview;
            set { _reminderPreview = value; OnPropertyChanged(); }
        }

        public bool IsBuiltInReminder
        {
            get => _isBuiltInReminder;
            set { _isBuiltInReminder = value; OnPropertyChanged(); UpdateReminderPreview(); }
        }

        public bool IsCustomReminder
        {
            get => _isCustomReminder;
            set { _isCustomReminder = value; OnPropertyChanged(); UpdateReminderPreview(); }
        }

        public string CustomFunctionExpression
        {
            get => _customFunctionExpression;
            set { _customFunctionExpression = value; OnPropertyChanged(); UpdateReminderPreview(); }
        }

        public int PendingCount => Reminders.Count(r => !r.IsCompleted);
        public int CompletedCount => Reminders.Count(r => r.IsCompleted);

        public ICommand AddTaskCommand { get; }
        public ICommand DeleteTaskCommand { get; }

        public MainWindow()
        {
            InitializeComponent();
            DataContext = this;

            _db = new DatabaseService();
            _reminderCheck = new ReminderCheckService(_db);
            _reminderCheck.OnReminderTriggered += OnReminderTriggered;

            AddTaskCommand = new RelayCommand(AddTask);
            DeleteTaskCommand = new RelayCommand(DeleteTask);

            LoadReminders();
            _reminderCheck.Start();

            DueDatePicker.SelectedDate = DateTime.Today.AddDays(1);
            DueTimeComboBox.SelectedIndex = 8; // 默认18:00
            ReminderFunctionComboBox.SelectedIndex = 0;
            UpdateReminderPreview();
            
            // 设置皮肤按钮初始文本
            SkinButton.Content = $"{SkinService.Instance.GetSkinEmoji(SkinService.Instance.CurrentSkin)} {SkinService.Instance.GetSkinName(SkinService.Instance.CurrentSkin)}";
        }

        private void LoadReminders()
        {
            var list = _db.LoadAllReminders();
            Reminders = new ObservableCollection<ReminderItem>(list);
            OnPropertyChanged(nameof(PendingCount));
            OnPropertyChanged(nameof(CompletedCount));
        }

        private DateTime CalculateDueTime()
        {
            var hour = DueTimeHours[NewDueTimeIndex];
            return NewDueDate.Date.AddHours(hour);
        }

        private void UpdateReminderPreview()
        {
            var dueTime = CalculateDueTime();
            DateTime reminderTime;

            if (IsCustomReminder)
            {
                var item = new ReminderItem { DueTime = dueTime, ReminderType = 1, ReminderFunction = CustomFunctionExpression };
                item.CalculateReminderTime();
                reminderTime = item.ReminderTime;
            }
            else
            {
                var functionName = BuiltInFunctions[NewReminderFunctionIndex];
                var item = new ReminderItem { DueTime = dueTime, ReminderFunction = functionName };
                item.CalculateReminderTime();
                reminderTime = item.ReminderTime;
            }

            ReminderPreview = $"{reminderTime:MM/dd HH:mm} (完成时间: {dueTime:MM/dd HH:mm})";
        }

        private void ReminderType_Changed(object sender, RoutedEventArgs e)
        {
            if (BuiltInPanel == null || CustomPanel == null) return;
            
            if (IsBuiltInReminder)
            {
                BuiltInPanel.Visibility = Visibility.Visible;
                CustomPanel.Visibility = Visibility.Collapsed;
            }
            else
            {
                BuiltInPanel.Visibility = Visibility.Collapsed;
                CustomPanel.Visibility = Visibility.Visible;
            }
            UpdateReminderPreview();
        }

        private void AddTask()
        {
            if (string.IsNullOrWhiteSpace(NewDescription))
            {
                MessageBox.Show("请输入任务描述", "提示", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            var dueTime = CalculateDueTime();
            DateTime reminderTime;
            string reminderFunction;
            int reminderType;

            if (IsCustomReminder)
            {
                reminderType = 1;
                reminderFunction = CustomFunctionExpression;
                var item = new ReminderItem { DueTime = dueTime, ReminderType = 1, ReminderFunction = reminderFunction };
                item.CalculateReminderTime();
                reminderTime = item.ReminderTime;
            }
            else
            {
                reminderType = 0;
                reminderFunction = BuiltInFunctions[NewReminderFunctionIndex];
                var item = new ReminderItem { DueTime = dueTime, ReminderFunction = reminderFunction };
                item.CalculateReminderTime();
                reminderTime = item.ReminderTime;
            }

            var itemDb = new ReminderItem
            {
                Description = NewDescription,
                Priority = NewPriority,
                DueTime = dueTime,
                ReminderTime = reminderTime,
                ReminderType = reminderType,
                ReminderFunction = reminderFunction
            };

            _db.SaveReminder(itemDb);
            LoadReminders();

            NewDescription = "";
            PriorityComboBox.SelectedIndex = 1;
            DueDatePicker.SelectedDate = DateTime.Today.AddDays(1);
            DueTimeComboBox.SelectedIndex = 8;
            ReminderFunctionComboBox.SelectedIndex = 0;
        }

        private void DeleteTask()
        {
            if (SelectedReminder == null) return;
            var result = MessageBox.Show("确定要删除任务吗？", "确认删除", MessageBoxButton.YesNo, MessageBoxImage.Question);
            if (result == MessageBoxResult.Yes)
            {
                _db.DeleteReminder(SelectedReminder.Id);
                LoadReminders();
            }
        }

        private void OnReminderTriggered(ReminderItem item)
        {
            Dispatcher.Invoke(LoadReminders);
        }

        private void CompleteCheckBox_Click(object sender, RoutedEventArgs e)
        {
            if (SelectedReminder != null)
            {
                _db.SaveReminder(SelectedReminder);
                OnPropertyChanged(nameof(PendingCount));
                OnPropertyChanged(nameof(CompletedCount));
            }
        }

        private void DeleteButton_Click(object sender, RoutedEventArgs e)
        {
            if (sender is FrameworkElement fe && fe.Tag is Guid id)
            {
                var item = Reminders.FirstOrDefault(r => r.Id == id);
                if (item != null)
                {
                    var result = MessageBox.Show("确定要删除任务吗？", "确认删除", MessageBoxButton.YesNo, MessageBoxImage.Question);
                    if (result == MessageBoxResult.Yes)
                    {
                        _db.DeleteReminder(id);
                        LoadReminders();
                    }
                }
            }
        }

        /// <summary>
        /// 皮肤切换按钮点击
        /// </summary>
        private void SkinButton_Click(object sender, RoutedEventArgs e)
        {
            var menu = new ContextMenu();
            
            foreach (SkinService.SkinType skin in Enum.GetValues(typeof(SkinService.SkinType)))
            {
                var item = new MenuItem
                {
                    Header = $"{SkinService.Instance.GetSkinEmoji(skin)} {SkinService.Instance.GetSkinName(skin)}",
                    Tag = skin,
                    IsCheckable = true,
                    IsChecked = skin == SkinService.Instance.CurrentSkin
                };
                item.Click += SkinMenuItem_Click;
                menu.Items.Add(item);
            }
            
            menu.IsOpen = true;
        }

        private void SkinMenuItem_Click(object sender, RoutedEventArgs e)
        {
            if (sender is MenuItem mi && mi.Tag is SkinService.SkinType skin)
            {
                SkinService.Instance.ChangeSkin(skin);
                SkinButton.Content = $"{SkinService.Instance.GetSkinEmoji(skin)} {SkinService.Instance.GetSkinName(skin)}";
            }
        }

        protected override void OnClosed(EventArgs e)
        {
            _reminderCheck.Stop();
            _db.Dispose();
            base.OnClosed(e);
        }

        protected void OnPropertyChanged([CallerMemberName] string? name = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
        }
    }

    public class RelayCommand : ICommand
    {
        private readonly Action _execute;
        public RelayCommand(Action execute) => _execute = execute;
        public event EventHandler? CanExecuteChanged;
        public bool CanExecute(object? parameter) => true;
        public void Execute(object? parameter) => _execute();
    }
}