@echo off
echo ========================================
echo ReminderDesk 构建和签名脚本
echo ========================================

cd /d "%~dp0"

echo [1] 构建应用...
call npm run tauri build

echo [2] 签名可执行文件...
set CERT_THUMBPRINT=95B9B64BE18A83938D06530C39F1892B46ADD01E
set SIGNTOOL="C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"

%SIGNTOOL% sign /sha1 %CERT_THUMBPRINT% /fd sha256 "src-tauri\target\release\reminder-app.exe"
echo 已签名 exe

echo [3] 签名安装程序...
%SIGNTOOL% sign /sha1 %CERT_THUMBPRINT% /fd sha256 "src-tauri\target\release\bundle\nsis\任务提醒助手_0.1.0_x64-setup.exe"
echo 已签名安装包

echo [4] 复制到桌面...
copy "src-tauri\target\release\bundle\nsis\任务提醒助手_0.1.0_x64-setup.exe" "%USERPROFILE%\Desktop\"

echo ========================================
echo 构建完成！
echo 安装包已复制到桌面
echo ========================================

pause