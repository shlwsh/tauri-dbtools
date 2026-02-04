# MyGit 自动化工作流使用说明

## 概述

MyGit 是一个自动化的 Git 工作流脚本，可以：
1. 自动检测文件变更
2. 使用 AI 生成提交信息
3. 自动提交并推送到远程仓库

## 文件说明

- `mygit.ps1` - Windows PowerShell 版本
- `mygit.sh` - Linux/macOS Bash 版本

## 安装

### Windows (PowerShell)

```powershell
# 无需安装，直接使用
.\mygit.ps1
```

### Linux/macOS (Bash)

```bash
# 添加执行权限
chmod +x mygit.sh

# 运行脚本
./mygit.sh
```

## 使用方法

### 基本用法

#### 1. 自动生成提交信息（推荐）

```powershell
# Windows
.\mygit.ps1

# Linux/macOS
./mygit.sh
```

脚本会：
- 检测所有文件变更
- 分析变更类型
- 自动生成符合约定式提交规范的提交信息
- 询问是否使用生成的信息
- 提交并推送到远程仓库

#### 2. 使用自定义提交信息

```powershell
# Windows
.\mygit.ps1 -Message "feat: 添加新功能"

# Linux/macOS
./mygit.sh "feat: 添加新功能"
```

#### 3. 预览模式（不执行实际操作）

```powershell
# Windows
.\mygit.ps1 -DryRun

# Linux/macOS
./mygit.sh --dry-run
```

### 高级用法

#### 指定远程仓库和分支

```powershell
# Windows
.\mygit.ps1 -Remote "origin" -Branch "main"

# Linux/macOS
./mygit.sh --remote origin --branch main
```

#### 组合参数

```powershell
# Windows
.\mygit.ps1 -Message "fix: 修复bug" -Branch "develop" -DryRun

# Linux/macOS
./mygit.sh "fix: 修复bug" --branch develop --dry-run
```

## AI 提交信息生成规则

脚本会根据文件变更自动判断提交类型：

### 提交类型

| 类型 | 说明 | 触发条件 |
|------|------|----------|
| `test` | 测试相关 | 修改了测试文件 (test/spec) |
| `docs` | 文档更新 | 修改了 Markdown 文件 |
| `feat` | 新功能 | 修改了前端或后端代码 |
| `chore` | 构建/配置 | 修改了配置文件 |

### 范围（Scope）

| 范围 | 说明 | 触发条件 |
|------|------|----------|
| `tests` | 测试 | 测试文件变更 |
| `frontend` | 前端 | frontend/ 目录变更 |
| `backend` | 后端 | src-tauri/ 目录变更 |
| `fullstack` | 全栈 | 前后端都有变更 |
| `config` | 配置 | 配置文件变更 |

### 提交信息格式

```
<类型>(<范围>): <描述>

变更文件数: X
```

示例：
```
test(tests): 完善测试覆盖率

变更文件数: 8
```

## 工作流程

```
┌─────────────────────┐
│  检查 Git 仓库      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  检测文件变更       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  分析变更类型       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  生成提交信息       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  用户确认           │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  git add .          │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  git commit         │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  git push           │
└─────────────────────┘
```

## 输出示例

```
ℹ === Git 自动化工作流 ===

✓ Git 仓库检查通过
ℹ 当前分支: main
ℹ 目标分支: main
ℹ 远程仓库: origin

✓ 检测到文件变更
ℹ 变更文件数: 8

ℹ 变更摘要:
- [新增] frontend/test/setup.ts
- [新增] frontend/bunfig.toml
- [新增] frontend/src/__tests__/properties.spec.ts
- [新增] frontend/src/utils/__tests__/format.spec.ts
- [新增] frontend/src/utils/__tests__/validation.spec.ts
- [新增] frontend/src/stores/__tests__/database.spec.ts
- [修改] frontend/src/utils/storage.ts
- [新增] frontend/TESTING.md

✓ AI 生成的提交信息:
test(tests): 完善测试覆盖率

变更文件数: 8

是否使用此提交信息？(Y/n): Y

ℹ === 开始执行 Git 操作 ===

ℹ 步骤 1/3: 添加所有更改...
✓ 已添加所有更改
ℹ 步骤 2/3: 提交更改...
✓ 提交成功
ℹ 步骤 3/3: 推送到远程仓库...
✓ 推送成功

✓ === 所有操作完成 ===
ℹ 提交信息: test(tests): 完善测试覆盖率
ℹ 分支: main
ℹ 远程: origin
```

## 约定式提交规范

本脚本遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 格式

```
<类型>[可选的作用域]: <描述>

[可选的正文]

[可选的脚注]
```

### 常用类型

- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式（不影响代码运行）
- `refactor`: 重构（既不是新功能也不是修复 bug）
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动
- `perf`: 性能优化
- `ci`: CI 配置文件和脚本的变动

## 故障排除

### 问题 1: 权限不足

**Windows:**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Linux/macOS:**
```bash
chmod +x mygit.sh
```

### 问题 2: Git 推送失败

检查：
1. 是否配置了远程仓库
2. 是否有推送权限
3. 网络连接是否正常

```bash
# 检查远程仓库
git remote -v

# 测试连接
git fetch origin
```

### 问题 3: 没有检测到变更

确保：
1. 文件已保存
2. 不在 .gitignore 中
3. 工作区不是干净的

```bash
# 检查状态
git status
```

## 最佳实践

### 1. 提交前检查

使用预览模式查看将要执行的操作：
```powershell
.\mygit.ps1 -DryRun
```

### 2. 小步提交

频繁提交小的、逻辑相关的变更，而不是一次提交大量变更。

### 3. 有意义的提交信息

即使使用 AI 生成，也要确保提交信息准确描述了变更内容。

### 4. 定期推送

不要在本地积累太多提交，定期推送到远程仓库。

### 5. 分支管理

在功能分支上开发，完成后合并到主分支：
```powershell
# 创建并切换到功能分支
git checkout -b feature/new-feature

# 开发完成后使用 mygit
.\mygit.ps1 -Branch "feature/new-feature"
```

## 集成到工作流

### 添加到 Git 别名

```bash
# 添加到 ~/.gitconfig
[alias]
    mygit = "!pwsh -File ./mygit.ps1"
    
# 使用
git mygit
```

### 添加到 VS Code 任务

在 `.vscode/tasks.json` 中添加：
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "MyGit: 自动提交",
            "type": "shell",
            "command": "pwsh",
            "args": ["-File", "${workspaceFolder}/mygit.ps1"],
            "problemMatcher": []
        }
    ]
}
```

## 扩展功能

### 集成真实的 AI API

如果你想使用真实的 AI API（如 OpenAI）生成更智能的提交信息，可以修改脚本中的 `Get-AICommitMessage` 函数：

```powershell
function Get-AICommitMessage {
    param([string[]]$Files, [string]$DiffSummary)
    
    # 获取 diff
    $diff = git diff HEAD | Select-Object -First 100
    
    # 调用 OpenAI API
    $apiKey = $env:OPENAI_API_KEY
    $prompt = "根据以下 Git diff 生成提交信息：`n$diff"
    
    # 发送 API 请求...
    # 返回生成的提交信息
}
```

## 参考资料

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Git 文档](https://git-scm.com/doc)
- [PowerShell 文档](https://docs.microsoft.com/powershell/)

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
