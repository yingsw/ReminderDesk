# ReminderDesk - 任务提醒助手

一款基于 **Tauri 2.0 + Svelte 5** 构建的现代化桌面任务提醒应用。

## 功能特性

### 核心功能
- **任务管理** - 创建、编辑、删除、完成任务
- **优先级** - 四级优先级（低、中、高、紧急）
- **分类管理** - 自定义分类，支持颜色标识
- **智能提醒** - 25种预设提醒时间 + 自定义公式

### 提醒方式

**内置函数**：
- 提前提醒：5分钟、10分钟、15分钟、30分钟、1小时、2小时、3小时、6小时、12小时、1天、2天、3天、1周
- 当天提醒：早上7/8/9点、中午12点、傍晚17/18点、晚上20点
- 隔天提醒：第二天早上8/9点

**自定义公式**：
| 表达式 | 说明 |
|--------|------|
| `DueTime-1h` | 完成时间前1小时 |
| `DueTime-30m` | 完成时间前30分钟 |
| `Date+9h` | 当天早上9点 |
| `Tomorrow+8h` | 次日早上8点 |

单位：`m` = 分钟，`h` = 小时，`d` = 天

### 数据管理
- **导出** - JSON 格式导出，便于备份
- **导入** - 支持文件导入或粘贴导入，可选合并模式
- **本地存储** - SQLite 数据库，数据安全可靠

### 界面特性
- **现代 UI** - 渐变紫色主题，简洁美观
- **系统托盘** - 最小化到托盘运行
- **窗口记忆** - 自动保存窗口尺寸和位置
- **分页显示** - 支持 10/15/20/30/50 条每页

## 技术栈

| 技术 | 版本 | 说明 |
|------|------|------|
| Tauri | 2.0 | Rust 后端框架 |
| Svelte | 5.0 | 响应式前端框架 |
| SQLite | - | 本地数据存储 |
| Vite | 6.0 | 构建工具 |

## 安装使用

### 从发布包安装
下载最新的安装包，双击安装即可。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yingsw/ReminderDesk.git
cd ReminderDesk

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建发布版本
npm run tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/nsis/` 目录。

## 项目结构

```
ReminderDesk/
├── src/                    # Svelte 前端
│   ├── App.svelte          # 主应用组件
│   ├── app.css             # 样式文件
│   └── main.js             # 入口文件
├── src-tauri/              # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs         # 应用入口
│   │   ├── database.rs     # 数据库操作
│   │   ├── reminder.rs     # 任务管理 API
│   │   ├── scheduler.rs    # 定时提醒
│   │   ├── settings.rs     # 设置管理
│   │   └── tray.rs         # 系统托盘
│   ├── icons/              # 应用图标
│   └── tauri.conf.json     # Tauri 配置
├── package.json            # Node 依赖配置
├── vite.config.js          # Vite 构建配置
└── svelte.config.js        # Svelte 配置
```

## 系统要求

- Windows 10/11
- 无需额外依赖，安装包已包含所有运行环境

## 开发信息

- **开发商**：浙江巨鼎包装有限公司
- **开发者**：应圣卫
- **版本**：0.1.0

## 许可证

MIT License