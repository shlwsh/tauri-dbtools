$file = "src-tauri\test_export.sql.gz"
$reader = [System.IO.File]::OpenRead($file)
$gzip = New-Object System.IO.Compression.GZipStream($reader, [System.IO.Compression.CompressionMode]::Decompress)
$streamReader = New-Object System.IO.StreamReader($gzip, [System.Text.Encoding]::UTF8)

$lineNum = 0
$foundData = $false

while ($null -ne ($line = $streamReader.ReadLine())) {
    $lineNum++
    
    if ($line -match "-- Data for table: bun_datasets") {
        $foundData = $true
        Write-Host "=== Found at line $lineNum ===" -ForegroundColor Green
    }
    
    if ($foundData) {
        Write-Host "$lineNum : $line"
        if ($lineNum -gt ($lineNum + 15)) {
            break
        }
    }
    
    if ($lineNum -gt 500) {
        break
    }
}

$streamReader.Close()
$gzip.Close()
$reader.Close()
