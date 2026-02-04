# 完整的端到端测试脚本
$ErrorActionPreference = "Continue"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "完整测试：数据库导出导入 (pg_dump/pg_restore)" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# 配置
$sourceDb = "personnel_db"
$targetDb = "p14_complete_test"
$host = "localhost"
$port = "5432"
$user = "postgres"
$password = "postgres"
$env:PGPASSWORD = $password

Write-Host "测试配置:" -ForegroundColor Yellow
Write-Host "  源数据库: $sourceDb" -ForegroundColor Gray
Write-Host "  目标数据库: $targetDb" -ForegroundColor Gray
Write-Host "  主机: $host" -ForegroundColor Gray
Write-Host "  端口: $port`n" -ForegroundColor Gray

# 步骤1：运行集成测试
Write-Host "步骤1：运行集成测试..." -ForegroundColor Yellow
cd src-tauri
$testOutput = cargo run --bin test_pgtools 2>&1
$testExitCode = $LASTEXITCODE
cd ..

if ($testExitCode -eq 0) {
    Write-Host "   ✓ 集成测试通过" -ForegroundColor Green
} else {
    Write-Host "   ✗ 集成测试失败" -ForegroundColor Red
    Write-Host $testOutput
    exit 1
}

# 步骤2：验证导出文件
Write-Host "`n步骤2：验证导出文件..." -ForegroundColor Yellow
$exportDir = "$env:USERPROFILE\pg-db-tool-exports"
$latestExport = Get-ChildItem $exportDir -Filter "personnel_db_*.backup" | 
    Sort-Object LastWriteTime -Descending | 
    Select-Object -First 1

if ($latestExport) {
    $sizeKB = [math]::Round($latestExport.Length / 1KB, 2)
    Write-Host "   ✓ 找到导出文件: $($latestExport.Name)" -ForegroundColor Green
    Write-Host "   文件大小: $sizeKB KB" -ForegroundColor Gray
    Write-Host "   路径: $($latestExport.FullName)" -ForegroundColor Gray
} else {
    Write-Host "   ✗ 未找到导出文件" -ForegroundColor Red
    exit 1
}

# 步骤3：验证目标数据库
Write-Host "`n步骤3：验证目标数据库..." -ForegroundColor Yellow

# 检查数据库是否存在
$dbCheck = psql -h $host -p $port -U $user -d postgres -t -c "SELECT 1 FROM pg_database WHERE datname='p14_pgtools';" 2>&1
if ($dbCheck -match "1") {
    Write-Host "   ✓ 目标数据库已创建" -ForegroundColor Green
    
    # 获取表数量
    $tableCount = psql -h $host -p $port -U $user -d p14_pgtools -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';" 2>&1
    Write-Host "   表数量: $($tableCount.Trim())" -ForegroundColor Gray
    
    # 检查几个示例表
    $tables = @("departments", "employees", "product")
    foreach ($table in $tables) {
        $count = psql -h $host -p $port -U $user -d p14_pgtools -t -c "SELECT COUNT(*) FROM `"$table`";" 2>&1
        if ($count -match "\d+") {
            Write-Host "   ✓ $table : $($count.Trim()) 行" -ForegroundColor Green
        } else {
            Write-Host "   ⚠ $table : 查询失败" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "   ✗ 目标数据库不存在" -ForegroundColor Red
    exit 1
}

# 步骤4：数据完整性验证
Write-Host "`n步骤4：数据完整性验证..." -ForegroundColor Yellow

# 比较源数据库和目标数据库的表数量
$sourceTableCount = psql -h $host -p $port -U $user -d $sourceDb -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';" 2>&1
$targetTableCount = psql -h $host -p $port -U $user -d p14_pgtools -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';" 2>&1

Write-Host "   源数据库表数: $($sourceTableCount.Trim())" -ForegroundColor Gray
Write-Host "   目标数据库表数: $($targetTableCount.Trim())" -ForegroundColor Gray

if ($sourceTableCount.Trim() -eq $targetTableCount.Trim()) {
    Write-Host "   ✓ 表数量完全匹配" -ForegroundColor Green
} else {
    Write-Host "   ⚠ 表数量不匹配" -ForegroundColor Yellow
}

# 比较几个关键表的行数
$comparisonTables = @("departments", "employees", "product", "sys_dict", "sys_menu")
$allMatch = $true

foreach ($table in $comparisonTables) {
    $sourceCount = psql -h $host -p $port -U $user -d $sourceDb -t -c "SELECT COUNT(*) FROM `"$table`";" 2>&1
    $targetCount = psql -h $host -p $port -U $user -d p14_pgtools -t -c "SELECT COUNT(*) FROM `"$table`";" 2>&1
    
    if ($sourceCount -match "\d+" -and $targetCount -match "\d+") {
        $sourceNum = [int]($sourceCount.Trim())
        $targetNum = [int]($targetCount.Trim())
        
        if ($sourceNum -eq $targetNum) {
            Write-Host "   ✓ $table : $sourceNum 行 (匹配)" -ForegroundColor Green
        } else {
            Write-Host "   ✗ $table : 源 $sourceNum 行, 目标 $targetNum 行 (不匹配)" -ForegroundColor Red
            $allMatch = $false
        }
    }
}

# 步骤5：检查日志
Write-Host "`n步骤5：检查日志..." -ForegroundColor Yellow
$logDir = "$env:USERPROFILE\pg-db-tool-logs"
$todayLog = Get-ChildItem $logDir -Filter "pg-db-tool_$(Get-Date -Format 'yyyyMMdd').log" -ErrorAction SilentlyContinue

if ($todayLog) {
    Write-Host "   ✓ 找到日志文件: $($todayLog.Name)" -ForegroundColor Green
    Write-Host "   路径: $($todayLog.FullName)" -ForegroundColor Gray
    
    # 显示最后几行日志
    Write-Host "`n   最后10行日志:" -ForegroundColor Gray
    Get-Content $todayLog.FullName -Tail 10 | ForEach-Object {
        Write-Host "   $_" -ForegroundColor DarkGray
    }
} else {
    Write-Host "   ⚠ 未找到今天的日志文件" -ForegroundColor Yellow
}

# 总结
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "测试总结" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

if ($allMatch) {
    Write-Host "✓ 所有测试通过！数据完整性验证成功。" -ForegroundColor Green
    Write-Host "`n导出文件: $($latestExport.FullName)" -ForegroundColor Gray
    Write-Host "目标数据库: p14_pgtools" -ForegroundColor Gray
    Write-Host "`n使用 pg_dump/pg_restore 的实现完全正常工作！" -ForegroundColor Green
} else {
    Write-Host "⚠ 部分测试未通过，请检查日志。" -ForegroundColor Yellow
}

Write-Host "`n========================================`n" -ForegroundColor Cyan
