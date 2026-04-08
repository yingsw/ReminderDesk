using System;
using System.Windows;
using ReminderDesk.Services;

namespace ReminderDesk;

public partial class App : Application
{
    protected override void OnStartup(StartupEventArgs e)
    {
        // 初始化皮肤服务
        SkinService.Instance.Initialize();
        
        base.OnStartup(e);

        var mainWindow = new MainWindow();
        mainWindow.Show();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        base.OnExit(e);
    }
}