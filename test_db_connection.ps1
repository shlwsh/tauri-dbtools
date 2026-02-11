# 测试数据库连接和 SQL 执行
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "PostgreSQL 数据库连接测试" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$env:PGPASSWORD = "postgres"
$host_name = "localhost"
$port = "5432"
$user = "postgres"

# 测试 1: 列出所有数据库
Write-Host "测试 1: 列出所有数据库" -ForegroundColor Yellow
$databases = psql -h $host_name -p $port -U $user -d postgres -t -c "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname;"
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ 成功" -ForegroundColor Green
    Write-Host "可用数据库:" -ForegroundColor White
    $databases | ForEach-Object { Write-Host "  - $($_.Trim())" -ForegroundColor Gray }
} else {
    Write-Host "✗ 失败" -ForegroundColor Red
    exit 1
}

Write-Host ""

# 测试 2: 连接到 personnel_db 数据库
Write-Host "测试 2: 连接到 personnel_db 数据库" -ForegroundColor Yellow
$result = psql -h $host_name -p $port -U $user -d personnel_db -c "SELECT 1 as test;" 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ 成功连接到 personnel_db" -ForegroundColor Green
} else {
    Write-Host "✗ 连接失败" -ForegroundColor Red
    Write-Host $result -ForegroundColor Red
    exit 1
}

Write-Host ""

# 测试 3: 列出 personnel_db 中的表
Write-Host "测试 3: 列出 personnel_db 中的表" -ForegroundColor Yellow
$tables = psql -h $host_name -p $port -U $user -d personnel_db -t -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;"
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ 成功" -ForegroundColor Green
    Write-Host "可用表:" -ForegroundColor White
    $tables | ForEach-Object { 
        $table = $_.Trim()
        if ($table) {
            Write-Host "  - $table" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "✗ 失败" -ForegroundColor Red
    exit 1
}

Write-Host ""

# 测试 4: 查询 employees 表
Write-Host "测试 4: 查询 employees 表" -ForegroundColor Yellow
$query_result = psql -h $host_name -p $port -U $user -d personnel_db -c "SELECT * FROM employees LIMIT 5;" 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ 查询成功" -ForegroundColor Green
    Write-Host $query_result -ForegroundColor Gray
} else {
    Write-Host "✗ 查询失败" -ForegroundColor Red
    Write-Host $query_result -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "测试完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
