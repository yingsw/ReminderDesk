using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Windows;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ReminderDesk.Models;
using ReminderDesk.Services;

namespace ReminderDesk.ViewModel
{
    /// <summary>
    /// Main window ViewModel - manages reminder task list and business logic
    /// </summary>
    public partial class MainViewModel : ObservableObject
    {
        /// <summary>Reminder task collection</summary>
        [ObservableProperty]
        private ObservableCollection<ReminderItem> _reminders = new();

        /// <summary>Currently selected task</summary>
        [ObservableProperty]
        private ReminderItem? _selectedReminder;

        /// <summary>New task title</summary>
        [ObservableProperty]
        private string _newTaskTitle = string.Empty;

        /// <summary>New task description</summary>
        [ObservableProperty]
        private string _newTaskDescription = string.Empty;

        /// <summary>New task reminder time</summary>
        [ObservableProperty]
        private DateTime _newTaskReminderTime = DateTime.Now.AddHours(1);

        /// <summary>New task priority</summary>
        [ObservableProperty]
        private int _newTaskPriority = 1;

        /// <summary>New task schedule type</summary>
        [ObservableProperty]
        private ScheduleType _newTaskScheduleType = ScheduleType.Once;

        /// <summary>New task scheduled hour</summary>
        [ObservableProperty]
        private int _newTaskHour = 9;

        /// <summary>New task scheduled minute</summary>
        [ObservableProperty]
        private int _newTaskMinute = 0;

        /// <summary>New task weekdays (for Weekly)</summary>
        [ObservableProperty]
        private string _newTaskWeekDays = "1";

        /// <summary>New task month days (for Monthly)</summary>
        [ObservableProperty]
        private string _newTaskMonthDays = "1";

        /// <summary>New task Cron expression</summary>
        [ObservableProperty]
        private string _newTaskCronExpression = "0 30 9 * * 1-5";

        /// <summary>Pending task count</summary>
        [ObservableProperty]
        private int _pendingCount;

        /// <summary>Completed task count</summary>
        [ObservableProperty]
        private int _completedCount;

        /// <summary>Is scheduler running</summary>
        [ObservableProperty]
        private bool _schedulerRunning;

        /// <summary>Next check time display</summary>
        [ObservableProperty]
        private string _nextCheckTime = string.Empty;

        /// <summary>Scheduler service</summary>
        private SchedulerService _scheduler;

        /// <summary>Database service</summary>
        private readonly DatabaseService _databaseService;

        /// <summary>Available schedule types</summary>
        public ScheduleType[] AvailableScheduleTypes => Enum.GetValues<ScheduleType>();

        /// <summary>Week day options</summary>
        public string[] WeekDayOptions => new[] { "0=Sun", "1=Mon", "2=Tue", "3=Wed", "4=Thu", "5=Fri", "6=Sat" };

        /// <summary>Cron examples</summary>
        public string[] CronExampleList => ScheduleHelper.GetCronExamples().ToArray();

        /// <summary>Constructor - initialize scheduler and sample data</summary>
        public MainViewModel()
        {
            _databaseService = new DatabaseService();
            _scheduler = new SchedulerService(Reminders.ToList(), checkIntervalSeconds: 10);
            _scheduler.ReminderTriggered += OnReminderTriggered;

            LoadSampleData();
            UpdateStatistics();
            UpdateAllNextReminderTimes();
            StartScheduler();
        }

        /// <summary>Load sample data for demo</summary>
        private void LoadSampleData()
        {
            // Single reminder example
            Reminders.Add(new ReminderItem
            {
                Title = "Complete Project Report",
                Description = "Submit Q1 project progress report",
                ReminderTime = DateTime.Now.AddMinutes(30),
                NextReminderTime = DateTime.Now.AddMinutes(30),
                Priority = 2,
                Schedule = ScheduleHelper.CreateOnce(DateTime.Now.AddMinutes(30))
            });

            // Daily reminder example
            var dailyReminder = new ReminderItem
            {
                Title = "Daily Morning Meeting",
                Description = "Team morning meeting at 9am every day",
                Priority = 1,
                Schedule = ScheduleHelper.CreateDaily(9, 0)
            };
            dailyReminder.UpdateNextReminderTime();
            Reminders.Add(dailyReminder);

            // Weekly reminder example
            var weeklyReminder = new ReminderItem
            {
                Title = "Weekly Summary",
                Description = "Weekly summary at 5pm every Friday",
                Priority = 2,
                Schedule = ScheduleHelper.CreateWeekly("5", 17, 0)
            };
            weeklyReminder.UpdateNextReminderTime();
            Reminders.Add(weeklyReminder);

            // Monthly reminder example
            var monthlyReminder = new ReminderItem
            {
                Title = "Monthly Report",
                Description = "Monthly report on 1st and 15th",
                Priority = 3,
                Schedule = ScheduleHelper.CreateMonthly("1,15", 9, 0)
            };
            monthlyReminder.UpdateNextReminderTime();
            Reminders.Add(monthlyReminder);

            // Cron expression example (workdays 9:30)
            var cronReminder = new ReminderItem
            {
                Title = "Cron Example",
                Description = "Reminder at 9:30 on workdays (Mon-Fri)",
                Priority = 1,
                Schedule = ScheduleHelper.CreateCron("0 30 9 * * 1-5")
            };
            cronReminder.UpdateNextReminderTime();
            Reminders.Add(cronReminder);

            // Completed task example
            Reminders.Add(new ReminderItem
            {
                Title = "Reply Email",
                Description = "Reply to customer technical inquiry",
                ReminderTime = DateTime.Now.AddHours(5),
                Priority = 0,
                IsCompleted = true,
                Schedule = new ScheduleConfig { Type = ScheduleType.Once, IsEnabled = false }
            });
        }

        /// <summary>Start scheduler</summary>
        [RelayCommand]
        private void StartScheduler()
        {
            _scheduler.Start();
            SchedulerRunning = true;
            NextCheckTime = "Scheduler running...";
        }

        /// <summary>Stop scheduler</summary>
        [RelayCommand]
        private void StopScheduler()
        {
            _scheduler.Stop();
            SchedulerRunning = false;
            NextCheckTime = "Scheduler stopped";
        }

        /// <summary>Scheduler reminder triggered callback</summary>
        private void OnReminderTriggered(object? sender, ReminderItem reminder)
        {
            System.Windows.Application.Current?.Dispatcher.Invoke(() =>
            {
                UpdateStatistics();

                if (reminder.Schedule.Type != ScheduleType.Once)
                {
                    OnPropertyChanged(nameof(Reminders));
                }
            });
        }

        /// <summary>Update all next reminder times</summary>
        [RelayCommand]
        private void UpdateAllNextReminderTimes()
        {
            foreach (var reminder in Reminders)
            {
                reminder.UpdateNextReminderTime();
            }
        }

        /// <summary>Update statistics</summary>
        private void UpdateStatistics()
        {
            PendingCount = Reminders.Count(r => !r.IsCompleted && r.Schedule.IsEnabled);
            CompletedCount = Reminders.Count(r => r.IsCompleted || !r.Schedule.IsEnabled);
        }

        /// <summary>Add new task command</summary>
        [RelayCommand]
        private void AddTask()
        {
            if (string.IsNullOrWhiteSpace(NewTaskTitle))
            {
                MessageBox.Show("Please enter task title", "Info", MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            ScheduleConfig schedule;
            switch (NewTaskScheduleType)
            {
                case ScheduleType.Once:
                    schedule = ScheduleHelper.CreateOnce(NewTaskReminderTime);
                    break;
                case ScheduleType.Daily:
                    schedule = ScheduleHelper.CreateDaily(NewTaskHour, NewTaskMinute);
                    break;
                case ScheduleType.Weekly:
                    schedule = ScheduleHelper.CreateWeekly(NewTaskWeekDays, NewTaskHour, NewTaskMinute);
                    break;
                case ScheduleType.Monthly:
                    schedule = ScheduleHelper.CreateMonthly(NewTaskMonthDays, NewTaskHour, NewTaskMinute);
                    break;
                case ScheduleType.CronExpression:
                    if (!ScheduleHelper.ValidateCronExpression(NewTaskCronExpression))
                    {
                        MessageBox.Show("Invalid Cron expression", "Error", MessageBoxButton.OK, MessageBoxImage.Error);
                        return;
                    }
                    schedule = ScheduleHelper.CreateCron(NewTaskCronExpression);
                    break;
                default:
                    schedule = new ScheduleConfig { Type = ScheduleType.Once };
                    break;
            }

            var newReminder = new ReminderItem
            {
                Title = NewTaskTitle,
                Description = NewTaskDescription,
                ReminderTime = NewTaskReminderTime,
                Priority = NewTaskPriority,
                Schedule = schedule
            };

            newReminder.UpdateNextReminderTime();

            Reminders.Add(newReminder);
            UpdateStatistics();

            // Clear inputs
            NewTaskTitle = string.Empty;
            NewTaskDescription = string.Empty;
            NewTaskReminderTime = DateTime.Now.AddHours(1);
            NewTaskPriority = 1;
            NewTaskScheduleType = ScheduleType.Once;
            NewTaskHour = 9;
            NewTaskMinute = 0;
            NewTaskWeekDays = "1";
            NewTaskMonthDays = "1";
            NewTaskCronExpression = "0 30 9 * * 1-5";

            System.Diagnostics.Debug.WriteLine($"[ReminderDesk] Task added: {newReminder.Title}");
        }

        /// <summary>Complete task command</summary>
        [RelayCommand]
        private void CompleteTask()
        {
            if (SelectedReminder == null)
            {
                MessageBox.Show("Please select a task first", "Info", MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            SelectedReminder.IsCompleted = true;

            if (SelectedReminder.Schedule.Type == ScheduleType.Once)
            {
                SelectedReminder.Schedule.IsEnabled = false;
            }

            UpdateStatistics();
            System.Diagnostics.Debug.WriteLine($"[ReminderDesk] Task completed: {SelectedReminder.Title}");
        }

        /// <summary>Delete task command</summary>
        [RelayCommand]
        private void DeleteTask()
        {
            if (SelectedReminder == null)
            {
                MessageBox.Show("Please select a task first", "Info", MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            var result = MessageBox.Show(
                $"Delete task \"{SelectedReminder.Title}\"?\nType: {SelectedReminder.GetScheduleTypeText()}",
                "Confirm Delete",
                MessageBoxButton.YesNo,
                MessageBoxImage.Question);

            if (result == MessageBoxResult.Yes)
            {
                Reminders.Remove(SelectedReminder);
                SelectedReminder = null;
                UpdateStatistics();
            }
        }

        /// <summary>Toggle schedule pause/resume</summary>
        [RelayCommand]
        private void ToggleSchedule()
        {
            if (SelectedReminder == null)
            {
                MessageBox.Show("Please select a task first", "Info", MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            SelectedReminder.Schedule.IsEnabled = !SelectedReminder.Schedule.IsEnabled;

            if (SelectedReminder.Schedule.IsEnabled)
            {
                SelectedReminder.UpdateNextReminderTime();
                System.Diagnostics.Debug.WriteLine($"[ReminderDesk] Schedule resumed: {SelectedReminder.Title}");
            }
            else
            {
                System.Diagnostics.Debug.WriteLine($"[ReminderDesk] Schedule paused: {SelectedReminder.Title}");
            }

            UpdateStatistics();
        }

        /// <summary>Trigger task immediately (for testing)</summary>
        [RelayCommand]
        private void TriggerNow()
        {
            if (SelectedReminder == null)
            {
                MessageBox.Show("Please select a task first", "Info", MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            _scheduler.TriggerNow(SelectedReminder);
            UpdateStatistics();
        }

        /// <summary>Show Cron examples</summary>
        [RelayCommand]
        private void ShowCronExamples()
        {
            var examples = string.Join("\n", ScheduleHelper.CronExamples.Select(e => $"{e.Key}: {e.Value}"));
            MessageBox.Show(examples, "Cron Examples", MessageBoxButton.OK, MessageBoxImage.Information);
        }

        /// <summary>Cleanup resources</summary>
        public void Cleanup()
        {
            _scheduler?.Dispose();
            _databaseService?.Dispose();
        }
    }
}