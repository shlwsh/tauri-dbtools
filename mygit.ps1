#!/usr/bin/env pwsh
<#
.SYNOPSIS
    自动化 Git 工作流 - 使用 AI 生成提交信息并推送到远程仓库

.DESCRIPTION
    此脚本会：
    1. 检查 Git 状态
    2. 获取修改的文件列表和差异
    3. 使用 AI 生成提交信息
    4. 添加所有更改
    5. 提交更改
    6. 推送到远程仓库

.PARAMETER Message
    可选的自定义提交信息。如果不提供，将使用 AI 生成

.PARAMETER Branch
    要推送的分支名称。默认为当前分支

.PARAMETER Remote
    远程仓库名称。默认为 'origin'

.PARAMETER DryRun
    仅显示将要执行的操作，不实际执行

.EXAMPLE
    .\mygit.ps1
    使用 AI 生成提交信息并推送

.EXAMPLE
    .\mygit.ps1 -Message "fix: 修复登录问题"
    使用自定义提交信息

.EXAMPLE
    .\mygit.ps1 -DryRun
    预览将要执行的操作
#>

param(
    [string]$Message = "",
    [string]$Branch = "",
    [string]$Remote = "origin",
    [switch]$DryRun
)

# 颜色输出函数
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Success { param([string]$Message) Write-ColorOutput "✓ $Message" "Green" }
function Write-Info { param([string]$Message) Write-ColorOutput "ℹ $Message" "Cyan" }
function Write-Warning { param([string]$Message) Write-ColorOutput "⚠ $Message" "Yellow" }
function Write-Error { param([string]$Message) Write-ColorOutput "✗ $Message" "Red" }

# 检查是否在 Git 仓库中
function Test-GitRepository {
    try {
        git rev-parse --git-dir 2>&1 | Out-Null
        return $?
    }
    catch {
        return $false
    }
}

# 获取当前分支名称
function Get-CurrentBranch {
    try {
        $branch = git rev-parse --abbrev-ref HEAD 2>&1
        if ($LASTEXITCODE -eq 0) {
            return $branch.Trim()
        }
        return $null
    }
    catch {
        return $null
    }
}

# 获取 Git 状态
function Get-GitStatus {
    try {
        $status = git status --porcelain 2>&1
        return $status
    }
    catch {
        return $null
    }
}

# 获取修改的文件列表
function Get-ModifiedFiles {
    try {
        $files = git diff --name-only HEAD 2>&1
        $staged = git diff --cached --name-only 2>&1
        $untracked = git ls-files --others --exclude-standard 2>&1
        
        $allFiles = @()
        if ($files) { $allFiles += $files }
        if ($staged) { $allFiles += $staged }
        if ($untracked) { $allFiles += $untracked }
        
        return $allFiles | Select-Object -Unique
    }
    catch {
        return @()
    }
}

# 获取文件差异摘要
function Get-DiffSummary {
    param([string[]]$Files)
    
    $summary = @()
    
    foreach ($file in $Files) {
        $ext = [System.IO.Path]::GetExtension($file)
        $status = git status --porcelain $file 2>&1
        
        if ($status) {
            $statusCode = $status.Substring(0, 2).Trim()
            $statusText = switch ($statusCode) {
                "M" { "修改" }
                "A" { "新增" }
                "D" { "删除" }
                "R" { "重命名" }
                "??" { "未跟踪" }
                default { "变更" }
            }
            
            $summary += "- [$statusText] $file"
        }
    }
    
    return $summary -join "`n"
}

# 使用 AI 生成提交信息
function Get-AICommitMessage {
    param(
        [string[]]$Files,
        [string]$DiffSummary
    )
    
    Write-Info "正在使用 AI 生成提交信息..."
    
    # 获取简短的 diff 信息
    $shortDiff = git diff --stat HEAD 2>&1 | Select-Object -First 50
    
    # 构建提示信息
    $prompt = @"
请根据以下 Git 变更信息生成一个简洁、专业的中文提交信息。

变更文件列表：
$DiffSummary

变更统计：
$shortDiff

要求：
1. 使用约定式提交格式（Conventional Commits）
2. 格式：<类型>(<范围>): <描述>
3. 类型可以是：feat（新功能）、fix（修复）、docs（文档）、style（格式）、refactor（重构）、test（测试）、chore（构建/工具）
4. 描述要简洁明了，不超过 50 个字符
5. 如果有多个重要变更，可以在第二行添加详细说明
6. 只返回提交信息，不要有其他解释

示例：
feat(frontend): 添加用户登录功能
test(utils): 完善测试覆盖率并修复测试环境配置
"@

    # 这里我们使用一个简化的 AI 生成逻辑
    # 在实际使用中，你可以集成 OpenAI API 或其他 AI 服务
    
    # 分析文件类型和变更
    $hasTests = $Files | Where-Object { $_ -match "test|spec" }
    $hasDocs = $Files | Where-Object { $_ -match "\.md$|README" }
    $hasFrontend = $Files | Where-Object { $_ -match "frontend/" }
    $hasBackend = $Files | Where-Object { $_ -match "src-tauri/" }
    $hasConfig = $Files | Where-Object { $_ -match "\.json$|\.toml$|\.yaml$" }
    
    # 生成提交信息
    $type = "chore"
    $scope = ""
    $description = "更新项目文件"
    
    if ($hasTests) {
        $type = "test"
        $scope = "tests"
        $description = "完善测试覆盖率"
    }
    elseif ($hasDocs) {
        $type = "docs"
        $description = "更新文档"
    }
    elseif ($hasFrontend -and $hasBackend) {
        $type = "feat"
        $scope = "fullstack"
        $description = "更新前后端功能"
    }
    elseif ($hasFrontend) {
        $type = "feat"
        $scope = "frontend"
        $description = "更新前端功能"
    }
    elseif ($hasBackend) {
        $type = "feat"
        $scope = "backend"
        $description = "更新后端功能"
    }
    elseif ($hasConfig) {
        $type = "chore"
        $scope = "config"
        $description = "更新配置文件"
    }
    
    # 构建提交信息
    if ($scope) {
        $commitMessage = "${type}(${scope}): ${description}"
    }
    else {
        $commitMessage = "${type}: ${description}"
    }
    
    # 添加详细信息
    $fileCount = $Files.Count
    $details = "`n`n变更文件数: $fileCount"
    
    return $commitMessage + $details
}

# 主函数
function Main {
    Write-Info "=== Git 自动化工作流 ==="
    Write-Host ""
    
    # 检查是否在 Git 仓库中
    if (-not (Test-GitRepository)) {
        Write-Error "当前目录不是 Git 仓库"
        exit 1
    }
    
    Write-Success "Git 仓库检查通过"
    
    # 获取当前分支
    $currentBranch = Get-CurrentBranch
    if (-not $currentBranch) {
        Write-Error "无法获取当前分支"
        exit 1
    }
    
    if (-not $Branch) {
        $Branch = $currentBranch
    }
    
    Write-Info "当前分支: $currentBranch"
    Write-Info "目标分支: $Branch"
    Write-Info "远程仓库: $Remote"
    Write-Host ""
    
    # 检查是否有变更
    $status = Get-GitStatus
    if (-not $status) {
        Write-Warning "没有检测到任何变更"
        Write-Info "工作区是干净的，无需提交"
        exit 0
    }
    
    Write-Success "检测到文件变更"
    
    # 获取修改的文件
    $modifiedFiles = Get-ModifiedFiles
    if ($modifiedFiles.Count -eq 0) {
        Write-Warning "没有找到修改的文件"
        exit 0
    }
    
    Write-Info "变更文件数: $($modifiedFiles.Count)"
    Write-Host ""
    
    # 显示变更摘要
    $diffSummary = Get-DiffSummary -Files $modifiedFiles
    Write-Info "变更摘要:"
    Write-Host $diffSummary
    Write-Host ""
    
    # 生成或使用提交信息
    if (-not $Message) {
        $Message = Get-AICommitMessage -Files $modifiedFiles -DiffSummary $diffSummary
        Write-Success "AI 生成的提交信息:"
        Write-Host $Message -ForegroundColor Yellow
        Write-Host ""
        
        # 确认提交信息
        if (-not $DryRun) {
            $confirm = Read-Host "是否使用此提交信息？(Y/n)"
            if ($confirm -and $confirm -ne "Y" -and $confirm -ne "y" -and $confirm -ne "") {
                $Message = Read-Host "请输入自定义提交信息"
            }
        }
    }
    else {
        Write-Info "使用自定义提交信息: $Message"
    }
    
    Write-Host ""
    
    # 预览模式
    if ($DryRun) {
        Write-Warning "=== 预览模式 - 不会执行实际操作 ==="
        Write-Info "将执行以下操作:"
        Write-Host "  1. git add ." -ForegroundColor Gray
        Write-Host "  2. git commit -m `"$Message`"" -ForegroundColor Gray
        Write-Host "  3. git push $Remote $Branch" -ForegroundColor Gray
        exit 0
    }
    
    # 执行 Git 操作
    Write-Info "=== 开始执行 Git 操作 ==="
    Write-Host ""
    
    # 1. 添加所有更改
    Write-Info "步骤 1/3: 添加所有更改..."
    try {
        git add . 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Success "已添加所有更改"
        }
        else {
            Write-Error "添加文件失败"
            exit 1
        }
    }
    catch {
        Write-Error "添加文件时发生错误: $_"
        exit 1
    }
    
    # 2. 提交更改
    Write-Info "步骤 2/3: 提交更改..."
    try {
        git commit -m $Message 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Success "提交成功"
        }
        else {
            Write-Error "提交失败"
            exit 1
        }
    }
    catch {
        Write-Error "提交时发生错误: $_"
        exit 1
    }
    
    # 3. 推送到远程仓库
    Write-Info "步骤 3/3: 推送到远程仓库..."
    try {
        $pushOutput = git push $Remote $Branch 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "推送成功"
            Write-Host ""
            Write-Success "=== 所有操作完成 ==="
            Write-Info "提交信息: $Message"
            Write-Info "分支: $Branch"
            Write-Info "远程: $Remote"
        }
        else {
            Write-Error "推送失败"
            Write-Host $pushOutput -ForegroundColor Red
            exit 1
        }
    }
    catch {
        Write-Error "推送时发生错误: $_"
        exit 1
    }
}

# 运行主函数
try {
    Main
}
catch {
    Write-Error "发生未预期的错误: $_"
    exit 1
}
