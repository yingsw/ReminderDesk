@echo off
echo ========================================
echo ReminderDesk 构建和签名脚本
echo ========================================

cd /d "%~dp0"

echo [1] 构建前端...
call npm run build

echo [2] 构建 Tauri 应用...
call npm run tauri build

echo [3] 签名可执行文件...
set CERT_THUMBPRINT=95B9B64BE18A83938D06530C39F1892B46ADD01E
set EXE_PATH=src-tauri\target\release\task_reminder.exe

if exist "%EXE_PATH%" (
    signtool sign /sha1 %CERT_THUMBPRINT% /fd sha256 "%EXE_PATH%"
    echo 已签名: %EXE_PATH%
) else (
    echo 可执行文件不存在
)

echo [4] 签名安装程序...
set INSTALLER_PATH=src-tauri\target\release\bundle\nsis\任务提醒助手_0.1.0_x64-setup.exe

if exist "%INSTALLER_PATH%" (
    signtool sign /sha1 %CERT_THUMBPRINT% /fd sha256 "%INSTALLER_PATH%"
    echo 已签名: %INSTALLER_PATH%
    echo.
    echo ========================================
    echo 构建完成！安装程序位置:
    echo %INSTALLER_PATH%
    echo ========================================
) else (
    echo 安装程序不存在
)

pause