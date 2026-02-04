# MyGit 更新日志

## v2.0.0 - 2026-02-04

### 🎉 重大更新：跨平台支持

#### 新增功能

- ✨ **Bun/TypeScript 版本** - 新增 `mygit.ts`，支持跨平台运行
- 🚀 **统一命令** - 使用 `bun run mygit` 在所有平台上运行
- 📦 **集成到 package.json** - 添加 npm 脚本支持
- 🎨 **彩色输出** - 改进的命令行界面

#### 使用方式

```bash
# 推荐方式（跨平台）
bun run mygit

# 或者使用平台特定版本
.\mygit.ps1      # Windows
./mygit.sh       # Linux/macOS
```

#### 技术栈

- **运行时**: Bun
- **语言**: TypeScript
- **依赖**: 无（使用 Bun 内置功能）

#### 优势

1. **跨平台** - 一套代码，所有平台运行
2. **快速** - Bun 提供极快的启动速度
3. **现代** - 使用 TypeScript 和最新的 JavaScript 特性
4. **简单** - 统一的命令行接口

---

## v1.0.0 - 2026-02-04

### 初始版本

#### 功能

- ✅ AI 智能生成提交信息
- ✅ 自动检测文件变更
- ✅ 约定式提交规范
- ✅ 一键提交推送
- ✅ 预览模式
- ✅ 自定义提交信息
- ✅ 指定分支和远程仓库

#### 平台支持

- Windows (PowerShell)
- Linux/macOS (Bash)

#### 文件

- `mygit.ps1` - PowerShell 版本
- `mygit.sh` - Bash 版本
- 完整文档

---

## 升级指南

### 从 v1.0.0 升级到 v2.0.0

#### 1. 确保安装了 Bun

```bash
# 检查 Bun 是否已安装
bun --version

# 如果未安装，使用以下命令安装
curl -fsSL https://bun.sh/install | bash
```

#### 2. 更新使用方式

**旧方式**:
```bash
.\mygit.ps1          # Windows
./mygit.sh           # Linux/macOS
```

**新方式（推荐）**:
```bash
bun run mygit        # 所有平台
```

#### 3. 保持兼容性

v2.0.0 完全向后兼容，你仍然可以使用旧的脚本：
- `mygit.ps1` 继续在 Windows 上工作
- `mygit.sh` 继续在 Linux/macOS 上工作

---

## 路线图

### v2.1.0 (计划中)

- [ ] 集成真实的 AI API（OpenAI/Claude）
- [ ] 支持多语言提交信息
- [ ] 提交信息模板系统
- [ ] 交互式提交信息编辑器

### v2.2.0 (计划中)

- [ ] Git hooks 集成
- [ ] 提交历史分析
- [ ] 自动生成 CHANGELOG
- [ ] 团队协作功能

### v3.0.0 (未来)

- [ ] GUI 界面
- [ ] VS Code 扩展
- [ ] 云端同步配置
- [ ] 高级 AI 功能

---

## 反馈和建议

如果你有任何问题、建议或功能请求，请：

1. 提交 Issue
2. 发起 Pull Request
3. 联系维护者

感谢使用 MyGit！🎉
