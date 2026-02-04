# MyGit 快速开始

## 🚀 一分钟上手

### 推荐方式：使用 Bun（跨平台）⭐

```bash
# 1. 直接运行（AI 自动生成提交信息）
bun run mygit

# 2. 使用自定义提交信息
bun run mygit "feat: 添加新功能"

# 3. 预览模式（不实际执行）
bun run mygit --dry-run

# 4. 指定分支
bun run mygit --branch develop
```

### Windows 用户（PowerShell）

```powershell
# 1. 直接运行（AI 自动生成提交信息）
.\mygit.ps1

# 2. 使用自定义提交信息
.\mygit.ps1 -Message "feat: 添加新功能"

# 3. 预览模式（不实际执行）
.\mygit.ps1 -DryRun
```

### Linux/macOS 用户（Bash）

```bash
# 1. 添加执行权限（首次使用）
chmod +x mygit.sh

# 2. 直接运行（AI 自动生成提交信息）
./mygit.sh

# 3. 使用自定义提交信息
./mygit.sh "feat: 添加新功能"

# 4. 预览模式（不实际执行）
./mygit.sh --dry-run
```

## 📋 常用场景

### 场景 1: 日常开发提交（推荐）

```bash
# 修改了一些文件后
bun run mygit
# 脚本会自动：
# 1. 检测变更
# 2. 生成提交信息
# 3. 询问确认
# 4. 提交并推送
```

### 场景 2: 快速提交

```bash
# 不想等待 AI 生成，直接使用自定义信息
bun run mygit "fix: 修复登录bug"
```

### 场景 3: 检查变更

```bash
# 先看看会提交什么
bun run mygit --dry-run
```

### 场景 4: 推送到特定分支

```bash
# 推送到 develop 分支
bun run mygit --branch develop
```

## 🎯 AI 生成的提交信息示例

脚本会根据你的文件变更自动生成合适的提交信息：

| 变更类型 | 生成的提交信息 |
|---------|---------------|
| 修改测试文件 | `test(tests): 完善测试覆盖率` |
| 修改文档 | `docs: 更新文档` |
| 修改前端代码 | `feat(frontend): 更新前端功能` |
| 修改后端代码 | `feat(backend): 更新后端功能` |
| 修改配置文件 | `chore(config): 更新配置文件` |

## ⚡ 工作流程

```
你修改文件 → 运行 bun run mygit → AI 生成提交信息 → 确认 → 自动提交推送 ✅
```

## 💡 提示

1. **首次使用建议用预览模式**
   ```bash
   bun run mygit --dry-run
   ```

2. **不满意 AI 生成的信息？**
   - 在确认时选择 `n`
   - 然后输入自定义信息

3. **想要更详细的信息？**
   - 查看 `MYGIT_USAGE.md` 获取完整文档

## 🔧 故障排除

### Bun 未安装

```bash
# 安装 Bun
curl -fsSL https://bun.sh/install | bash

# 或者使用 npm
npm install -g bun
```

### Windows: 无法运行 PowerShell 脚本

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Linux/macOS: 权限不足

```bash
chmod +x mygit.sh
```

### 推送失败

检查网络连接和 Git 配置：
```bash
git remote -v
git fetch origin
```

## 📚 更多信息

- 完整文档: `MYGIT_USAGE.md`
- 约定式提交规范: https://www.conventionalcommits.org/

---

**就这么简单！开始使用 MyGit 提升你的 Git 工作效率吧！** 🎉
