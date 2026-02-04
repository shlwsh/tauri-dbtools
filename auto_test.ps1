# 自动测试脚本
$ErrorActionPreference = "Continue"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "自动测试数据库导出导入" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# 1. 测试导出（使用 Rust 代码）
Write-Host "1. 测试导出 personnel_db..." -ForegroundColor Yellow

$exportDir = "$env:USERPROFILE\pg-db-tool-exports"
if (-not (Test-Path $exportDir)) {
    New-Item -ItemType Directory -Path $exportDir | Out-Null
}

# 获取最新的导出文件
$latestExport = Get-ChildItem $exportDir -Filter "personnel_db_*.sql.gz" | Sort-Object LastWriteTime -Descending | Select-Object -First 1

if ($latestExport) {
    Write-Host "   找到导出文件: $($latestExport.Name)" -ForegroundColor Green
    Write-Host "   文件大小: $([math]::Round($latestExport.Length/1KB, 2)) KB" -ForegroundColor Cyan
    
    # 2. 解压并查看前20行
    Write-Host "`n2. 查看导出文件内容..." -ForegroundColor Yellow
    $reader = [System.IO.File]::OpenRead($latestExport.FullName)
    $gzip = New-Object System.IO.Compression.GZipStream($reader, [System.IO.Compression.CompressionMode]::Decompress)
    $streamReader = New-Object System.IO.StreamReader($gzip)
    
    $lineCount = 0
    while (($line = $streamReader.ReadLine()) -and ($lineCount -lt 20)) {
        $lineCount++
        Write-Host "   $line" -ForegroundColor Gray
    }
    
    $streamReader.Close()
    $gzip.Close()
    $reader.Close()
    
    Write-Host "`n   显示了前 $lineCount 行" -ForegroundColor Cyan
} else {
    Write-Host "   ✗ 未找到导出文件" -ForegroundColor Red
    Write-Host "   请先在应用中导出 personnel_db" -ForegroundColor Yellow
    exit
}

# 3. 测试导入
Write-Host "`n3. 测试导入到 p14..." -ForegroundColor Yellow

$env:PGPASSWORD = "postgres"

# 检查 p14 是否存在
$checkP14 = psql -h localhost -U postgres -d postgres -t -c "SELECT 1 FROM pg_database WHERE datname='p14';" 2>&1

if ($checkP14 -match "1") {
    Write-Host "   数据库 p14 已存在" -ForegroundColor Cyan
    
    # 获取表数量
    $tableCount = psql -h localhost -U postgres -d p14 -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';" 2>&1
    Write-Host "   包含 $tableCount 个表" -ForegroundColor Cyan
    
    # 获取一些示例数据
    Write-Host "`n4. 验证数据..." -ForegroundColor Yellow
    $deptCount = psql -h localhost -U postgres -d p14 -t -c "SELECT COUNT(*) FROM departments;" 2>&1
    Write-Host "   departments 表: $deptCount 行" -ForegroundColor Cyan
    
    $empCount = psql -h localhost -U postgres -d p14 -t -c "SELECT COUNT(*) FROM employees;" 2>&1
    Write-Host "   employees 表: $empCount 行" -ForegroundColor Cyan
} else {
    Write-Host "   ✗ 数据库 p14 不存在" -ForegroundColor Red
    Write-Host "   请在应用中导入数据" -ForegroundColor Yellow
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "测试完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
