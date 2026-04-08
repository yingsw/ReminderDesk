using System;
using System.Collections.Generic;
using System.Windows;
using System.Windows.Media;

namespace ReminderDesk.Services
{
    /// <summary>
    /// 皮肤管理服务 - 支持多种风格切换
    /// </summary>
    public class SkinService
    {
        private static SkinService? _instance;
        public static SkinService Instance => _instance ??= new SkinService();

        public enum SkinType
        {
            Modern,  // 现代简约 - 紫蓝渐变
            Dark,    // 深色专业 - 科技感
            Fresh,   // 清新自然 - 绿色系
            Orange,  // 橙色活力 - 巨鼎主题
            Pink,    // 粉色温柔 - 柔美粉系
            Blue,    // 蓝色商务 - 专业稳重
            Gold,    // 金色奢华 - 高端典雅
            Cyan,    // 青色科技 - 未来感
            Rose     // 玫瑰红 - 热情活力
        }

        private SkinType _currentSkin = SkinType.Modern;
        public SkinType CurrentSkin => _currentSkin;

        public event EventHandler<SkinType>? SkinChanged;

        private readonly Dictionary<SkinType, string> _skinFiles = new()
        {
            { SkinType.Modern, "Themes/SkinModern.xaml" },
            { SkinType.Dark, "Themes/SkinDark.xaml" },
            { SkinType.Fresh, "Themes/SkinFresh.xaml" },
            { SkinType.Orange, "Themes/SkinOrange.xaml" },
            { SkinType.Pink, "Themes/SkinPink.xaml" },
            { SkinType.Blue, "Themes/SkinBlue.xaml" },
            { SkinType.Gold, "Themes/SkinGold.xaml" },
            { SkinType.Cyan, "Themes/SkinCyan.xaml" },
            { SkinType.Rose, "Themes/SkinRose.xaml" }
        };

        private readonly Dictionary<SkinType, string> _skinNames = new()
        {
            { SkinType.Modern, "现代简约" },
            { SkinType.Dark, "深色专业" },
            { SkinType.Fresh, "清新自然" },
            { SkinType.Orange, "橙色活力" },
            { SkinType.Pink, "粉色温柔" },
            { SkinType.Blue, "蓝色商务" },
            { SkinType.Gold, "金色奢华" },
            { SkinType.Cyan, "青色科技" },
            { SkinType.Rose, "玫瑰红" }
        };

        private readonly Dictionary<SkinType, string> _skinEmoji = new()
        {
            { SkinType.Modern, "💜" },
            { SkinType.Dark, "🖤" },
            { SkinType.Fresh, "💚" },
            { SkinType.Orange, "🧡" },
            { SkinType.Pink, "💗" },
            { SkinType.Blue, "💙" },
            { SkinType.Gold, "💛" },
            { SkinType.Cyan, "💎" },
            { SkinType.Rose, "🌹" }
        };

        private readonly Dictionary<SkinType, Color> _skinAccentColors = new()
        {
            { SkinType.Modern, Color.FromRgb(99, 102, 241) },   // #6366F1
            { SkinType.Dark, Color.FromRgb(0, 217, 255) },      // #00D9FF
            { SkinType.Fresh, Color.FromRgb(16, 185, 129) },    // #10B981
            { SkinType.Orange, Color.FromRgb(249, 115, 22) },   // #F97316
            { SkinType.Pink, Color.FromRgb(236, 72, 153) },     // #EC4899
            { SkinType.Blue, Color.FromRgb(37, 99, 235) },      // #2563EB
            { SkinType.Gold, Color.FromRgb(217, 119, 6) },      // #D97706
            { SkinType.Cyan, Color.FromRgb(6, 182, 212) },      // #06B6D4
            { SkinType.Rose, Color.FromRgb(225, 29, 72) }       // #E11D48
        };

        public string GetSkinName(SkinType skin) => _skinNames[skin];
        public string GetSkinEmoji(SkinType skin) => _skinEmoji[skin];
        public Color GetAccentColor(SkinType skin) => _skinAccentColors[skin];

        /// <summary>
        /// 切换皮肤
        /// </summary>
        public void ChangeSkin(SkinType newSkin)
        {
            if (newSkin == _currentSkin) return;

            var oldSkinFile = _skinFiles[_currentSkin];
            var oldResourceDict = Application.Current.Resources.MergedDictionaries
                .FirstOrDefault(d => d.Source?.OriginalString == oldSkinFile);
            if (oldResourceDict != null)
            {
                Application.Current.Resources.MergedDictionaries.Remove(oldResourceDict);
            }

            var newResourceDict = new ResourceDictionary
            {
                Source = new Uri(_skinFiles[newSkin], UriKind.Relative)
            };
            Application.Current.Resources.MergedDictionaries.Add(newResourceDict);

            _currentSkin = newSkin;
            SkinChanged?.Invoke(this, newSkin);
            SaveSkinPreference(newSkin);
        }

        /// <summary>
        /// 初始化皮肤（加载用户偏好）
        /// </summary>
        public void Initialize()
        {
            var savedSkin = LoadSkinPreference();
            var resourceDict = new ResourceDictionary
            {
                Source = new Uri(_skinFiles[savedSkin], UriKind.Relative)
            };
            Application.Current.Resources.MergedDictionaries.Add(resourceDict);
            _currentSkin = savedSkin;
        }

        private void SaveSkinPreference(SkinType skin)
        {
            try
            {
                var configPath = System.IO.Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                    "ReminderDesk",
                    "skin.cfg"
                );
                System.IO.Directory.CreateDirectory(System.IO.Path.GetDirectoryName(configPath)!);
                System.IO.File.WriteAllText(configPath, skin.ToString());
            }
            catch { }
        }

        private SkinType LoadSkinPreference()
        {
            try
            {
                var configPath = System.IO.Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                    "ReminderDesk",
                    "skin.cfg"
                );
                if (System.IO.File.Exists(configPath))
                {
                    var content = System.IO.File.ReadAllText(configPath);
                    return Enum.Parse<SkinType>(content);
                }
            }
            catch { }
            return SkinType.Modern;
        }
    }
}