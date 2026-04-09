using System.Windows;
using ReminderDesk.ViewModel;

namespace ReminderDesk
{
    /// <summary>
    /// 主窗口交互逻辑
    /// 实现窗口控制功能和界面初始化
    /// </summary>
    public partial class MainWindow : Window
    {
        private readonly MainViewModel _viewModel;

        public MainWindow()
        {
            InitializeComponent();
            _viewModel = new MainViewModel();
            DataContext = _viewModel;
            CenterWindow();
        }

        private void CenterWindow()
        {
            double screenWidth = SystemParameters.PrimaryScreenWidth;
            double screenHeight = SystemParameters.PrimaryScreenHeight;
            Left = (screenWidth / 2) - (Width / 2);
            Top = (screenHeight / 2) - (Height / 2);
        }

        private void Minimize_Click(object sender, RoutedEventArgs e)
        {
            WindowState = WindowState.Minimized;
        }

        private void Maximize_Click(object sender, RoutedEventArgs e)
        {
            if (WindowState == WindowState.Maximized)
            {
                WindowState = WindowState.Normal;
                Margin = new Thickness(10);
            }
            else
            {
                WindowState = WindowState.Maximized;
                Margin = new Thickness(0);
            }
        }

        private void Close_Click(object sender, RoutedEventArgs e)
        {
            Application.Current.Shutdown();
        }

        protected override void OnStateChanged(System.EventArgs e)
        {
            base.OnStateChanged(e);
            Margin = WindowState == WindowState.Maximized ? new Thickness(0) : new Thickness(10);
        }
    }
}