@echo off
echo 正在关闭旧的应用进程...
taskkill /F /IM pg-db-tool.exe 2>nul
timeout /t 2 /nobreak >nul

echo 正在启动应用...
cd src-tauri
cargo tauri dev
