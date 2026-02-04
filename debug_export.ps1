# 调试导出文件的脚本
param(
    [string]$ExportFile = ""
)

if ($ExportFile -eq "") {
    Write-Host "用法: .\debug_export.ps1 -ExportFile <导出文件路径>" -ForegroundColor Yellow
    Write-Host "示例: .\debug_export.ps1 -ExportFile 'C:\Users\Administrator\pg-db-tool-exports\personnel_db_20260204_142012.sql.gz'" -ForegroundColor Cyan
    exit
}

if (-not (Test-Path $ExportFile)) {
    Write-Host "文件不存在: $ExportFile" -ForegroundColor Red
    exit
}

Write-Host "分析导出文件: $ExportFile" -ForegroundColor Cyan
Write-Host ""

# 解压并查看前50行
Write-Host "文件前50行内容:" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Gray

$reader = [System.IO.File]::OpenRead($ExportFile)
$gzip = New-Object System.IO.Compression.GZipStream($reader, [System.IO.Compression.CompressionMode]::Decompress)
$streamReader = New-Object System.IO.StreamReader($gzip)

$lineCount = 0
while (($line = $streamReader.ReadLine()) -and ($lineCount -lt 50)) {
    $lineCount++
    Write-Host "$lineCount : $line"
}

$streamReader.Close()
$gzip.Close()
$reader.Close()

Write-Host "========================================" -ForegroundColor Gray
Write-Host "显示了前 $lineCount 行" -ForegroundColor Cyan
