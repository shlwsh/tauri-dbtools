# PostgreSQL 备份恢复测试脚本
# 设置环境变量
$env:PG_HOST = "localhost"
$env:PG_PORT = "5432"
$env:PG_USER = "postgres"
$env:PG_PASSWORD = "postgres"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "PostgreSQL 备份恢复测试" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 测试数据库连接
Write-Host "1. 测试数据库连接..." -ForegroundColor Yellow
$env:PGPASSWORD = $env:PG_PASSWORD
$testConn = psql -h $env:PG_HOST -p $env:PG_PORT -U $env:PG_USER -d postgres -c "SELECT version();" 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✓ 数据库连接成功" -ForegroundColor Green
} else {
    Write-Host "   ✗ 数据库连接失败" -ForegroundColor Red
    Write-Host "   错误: $testConn" -ForegroundColor Red
    exit 1
}

# 检查源数据库是否存在
Write-Host ""
Write-Host "2. 检查源数据库 personnel_db..." -ForegroundColor Yellow
$checkDb = psql -h $env:PG_HOST -p $env:PG_PORT -U $env:PG_USER -d postgres -t -c "SELECT 1 FROM pg_database WHERE datname='personnel_db';" 2>&1
if ($checkDb -match "1") {
    Write-Host "   ✓ 数据库 personnel_db 存在" -ForegroundColor Green
} else {
    Write-Host "   ✗ 数据库 personnel_db 不存在" -ForegroundColor Red
    exit 1
}

# 获取表数量
$tableCount = psql -h $env:PG_HOST -p $env:PG_PORT -U $env:PG_USER -d personnel_db -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';" 2>&1
Write-Host "   数据库包含 $tableCount 个表" -ForegroundColor Cyan

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "请在应用中执行以下操作：" -ForegroundColor Yellow
Write-Host "1. 导出数据库 'personnel_db'" -ForegroundColor White
Write-Host "2. 导入到新数据库 'p14'" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "按任意键继续验证结果..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# 验证导入结果
Write-Host ""
Write-Host "3. 验证导入结果..." -ForegroundColor Yellow

# 检查目标数据库是否存在
$checkP14 = psql -h $env:PG_HOST -p $env:PG_PORT -U $env:PG_USER -d postgres -t -c "SELECT 1 FROM pg_database WHERE datname='p14';" 2>&1
if ($checkP14 -match "1") {
    Write-Host "   ✓ 数据库 p14 已创建" -ForegroundColor Green
} else {
    Write-Host "   ✗ 数据库 p14 不存在" -ForegroundColor Red
    exit 1
}

# 获取目标数据库表数量
$p14TableCount = psql -h $env:PG_HOST -p $env:PG_PORT -U $env:PG_USER -d p14 -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';" 2>&1
Write-Host "   数据库 p14 包含 $p14TableCount 个表" -ForegroundColor Cyan

# 比较表数量
if ($tableCount -eq $p14TableCount) {
    Write-Host "   ✓ 表数量匹配" -ForegroundColor Green
} else {
    Write-Host "   ✗ 表数量不匹配 (源: $tableCount, 目标: $p14TableCount)" -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "测试完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
