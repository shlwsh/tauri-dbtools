# 验证文件选择修复
$ErrorActionPreference = "Continue"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "验证文件选择功能修复" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# 1. 检查导出目录
Write-Host "1. 检查导出目录..." -ForegroundColor Yellow
$exportDir = "$env:USERPROFILE\pg-db-tool-exports"

if (Test-Path $exportDir) {
    Write-Host "   ✓ 导出目录存在: $exportDir" -ForegroundColor Green
    
    # 列出所有备份文件
    $backupFiles = Get-ChildItem $exportDir -Filter "*.backup" | Sort-Object LastWriteTime -Descending
    
    if ($backupFiles.Count -gt 0) {
        Write-Host "   ✓ 找到 $($backupFiles.Count) 个 .backup 文件:" -ForegroundColor Green
        foreach ($file in $backupFiles | Select-Object -First 5) {
            $sizeKB = [math]::Round($file.Length / 1KB, 2)
            Write-Host "     - $($file.Name) ($sizeKB KB)" -ForegroundColor Gray
        }
    } else {
        Write-Host "   ⚠ 未找到 .backup 文件" -ForegroundColor Yellow
        Write-Host "   提示: 请先在应用中导出一个数据库" -ForegroundColor Gray
    }
} else {
    Write-Host "   ✗ 导出目录不存在" -ForegroundColor Red
}

# 2. 检查前端代码
Write-Host "`n2. 检查前端代码修复..." -ForegroundColor Yellow
$appTsxPath = "frontend\src\App.tsx"

if (Test-Path $appTsxPath) {
    $content = Get-Content $appTsxPath -Raw
    
    if ($content -match "extensions:\s*\['backup'") {
        Write-Host "   ✓ 文件过滤器已更新，包含 'backup' 扩展名" -ForegroundColor Green
    } else {
        Write-Host "   ✗ 文件过滤器未更新" -ForegroundColor Red
    }
    
    if ($content -match "PostgreSQL Backup Files") {
        Write-Host "   ✓ 过滤器名称已更新为 'PostgreSQL Backup Files'" -ForegroundColor Green
    } else {
        Write-Host "   ⚠ 过滤器名称未更新" -ForegroundColor Yellow
    }
} else {
    Write-Host "   ✗ 找不到 App.tsx 文件" -ForegroundColor Red
}

# 3. 检查应用是否运行
Write-Host "`n3. 检查应用状态..." -ForegroundColor Yellow
$process = Get-Process -Name "pg-db-tool" -ErrorAction SilentlyContinue

if ($process) {
    Write-Host "   ✓ 应用正在运行 (PID: $($process.Id))" -ForegroundColor Green
    Write-Host "   访问: http://localhost:8201/" -ForegroundColor Gray
} else {
    Write-Host "   ⚠ 应用未运行" -ForegroundColor Yellow
    Write-Host "   运行: bun run dev" -ForegroundColor Gray
}

# 4. 检查日志
Write-Host "`n4. 检查最新日志..." -ForegroundColor Yellow
$logDir = "$env:USERPROFILE\pg-db-tool-logs"
$todayLog = "$logDir\pg-db-tool_$(Get-Date -Format 'yyyyMMdd').log"

if (Test-Path $todayLog) {
    Write-Host "   ✓ 找到今天的日志文件" -ForegroundColor Green
    Write-Host "   最后 5 行:" -ForegroundColor Gray
    Get-Content $todayLog -Tail 5 | ForEach-Object {
        Write-Host "     $_" -ForegroundColor DarkGray
    }
} else {
    Write-Host "   ⚠ 未找到今天的日志文件" -ForegroundColor Yellow
}

# 5. 测试建议
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "测试步骤" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n1. 打开应用: http://localhost:8201/" -ForegroundColor White
Write-Host "2. 导出一个数据库（如果还没有导出）" -ForegroundColor White
Write-Host "3. 在 'Import Database' 部分点击 'Browse Files'" -ForegroundColor White
Write-Host "4. 验证文件选择对话框显示:" -ForegroundColor White
Write-Host "   - 文件类型: PostgreSQL Backup Files" -ForegroundColor Gray
Write-Host "   - 能看到所有 .backup 文件" -ForegroundColor Gray
Write-Host "5. 选择一个文件并测试导入" -ForegroundColor White

Write-Host "`n如果仍然看不到文件，请:" -ForegroundColor Yellow
Write-Host "1. 刷新浏览器页面 (Ctrl+F5)" -ForegroundColor Gray
Write-Host "2. 检查导出目录中是否有 .backup 文件" -ForegroundColor Gray
Write-Host "3. 查看浏览器控制台是否有错误" -ForegroundColor Gray

Write-Host "`n========================================`n" -ForegroundColor Cyan
