# Hook 快速参考指南

## 🎯 auto-continue-tasks Hook 速查表

### Token 使用阈值

| 范围 | Token 使用 | 状态 | Hook 行为 |
|------|-----------|------|-----------|
| 🟢 绿色 | < 80% (< 160K) | 正常 | 继续执行任务 |
| 🟡 黄色 | 80-95% (160K-190K) | 警告 | 生成摘要后继续 |
| 🔴 红色 | >= 95% (>= 190K) | 危险 | 停止并指引新会话 |

### 决策流程图（简化版）

```
Agent 停止
    ↓
检查 Token 使用
    ↓
├─ >= 95%? → 🛑 停止 + 生成摘要 + 新会话指引
├─ 80-95%? → ⚠️ 生成摘要 + 继续执行
└─ < 80%?  → ✅ 正常继续执行
    ↓
检查死循环（最近3次都是Hook触发？）
    ↓
├─ 是 → 🔄 停止 + 警告
└─ 否 → 继续
    ↓
检查 tasks.md
    ↓
├─ 所有完成 → ✅ 结束
└─ 有未完成 → 🚀 继续执行
```

### 常见输出

#### ✅ 正常完成
```
✅ 所有任务已完成（67/67）
Spec: database-advanced-features 已全部完成！
```

#### 🚀 继续执行
```
📊 Spec 任务状态检查
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Spec: database-advanced-features
进度: [3/67] 任务已完成
Token 使用: 45K/200K (22.5%)

接下来执行：
  [ ] 4.1 - 创建SQLEditor组件基础结构

🚀 继续执行...
```

#### ⚠️ 上下文警告
```
⚠️ 上下文接近限制
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Spec: database-advanced-features
进度: [35/67] 任务已完成
Token 使用: 170K/200K (85%)

📝 生成上下文摘要...
✅ 摘要已保存：.kiro/specs/database-advanced-features/CONTEXT_SUMMARY.md
```

#### 🛑 需要新会话
```
🛑 上下文已满，需要新会话
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Token 使用: 192K/200K (96%)

📌 请在新会话中继续：
发送消息："继续执行 database-advanced-features 的剩余任务，从任务 16.2 开始"
```

#### 🔄 死循环检测
```
🔄 检测到循环执行
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
最近 3 次执行都由 Hook 触发，可能陷入循环。
⚠️ 请手动检查问题并在新会话中继续。
```

## 🔧 故障排除

### 问题：Hook 没有触发

**检查清单**:
- [ ] Hook 是否启用？(`enabled: true`)
- [ ] JSON 格式是否正确？
- [ ] 文件名是否正确？(`.kiro.hook` 后缀)

**解决方案**:
```bash
# 验证 JSON 格式
cat .kiro/hooks/verify-task-completion.kiro.hook | jq .

# 检查 enabled 字段
grep "enabled" .kiro/hooks/verify-task-completion.kiro.hook
```

### 问题：Hook 触发但不继续执行

**可能原因**:
1. Token 使用已达到 95%
2. 检测到死循环
3. 没有找到 tasks.md 文件
4. 所有任务已完成

**调试步骤**:
1. 查看 Hook 输出的状态信息
2. 检查 token 使用百分比
3. 确认 `.kiro/specs/*/tasks.md` 存在
4. 验证任务状态标记格式

### 问题：陷入死循环

**识别标志**:
- 连续 3 次都是 Hook 触发
- 任务状态没有更新
- Token 使用快速增长

**解决方案**:
1. Hook 会自动检测并停止
2. 查看生成的 CONTEXT_SUMMARY.md
3. 在新会话中手动继续
4. 检查任务是否有问题

### 问题：上下文摘要文件未生成

**检查**:
- Token 使用是否 >= 80%？
- 是否有写入权限？
- `.kiro/specs/[spec-name]/` 目录是否存在？

**手动生成**:
```bash
# 复制模板
cp .kiro/specs/CONTEXT_SUMMARY_TEMPLATE.md \
   .kiro/specs/[spec-name]/CONTEXT_SUMMARY.md

# 编辑内容
# 填写当前进度和任务信息
```

## 📋 任务状态格式

### 正确格式 ✅

```markdown
- [ ] 未完成的任务
- [x] 已完成的任务
- [ ] 父任务
  - [x] 子任务 1
  - [ ] 子任务 2
```

### 错误格式 ❌

```markdown
- [] 缺少空格
- [X] 大写 X（应该用小写）
-[ ] 缺少空格
* [ ] 使用星号（应该用减号）
```

## 🎨 自定义 Hook

### 修改 Token 阈值

编辑 Hook 文件中的阈值：

```json
{
  "prompt": "...Token < 80%...Token 80-95%...Token >= 95%..."
}
```

修改为：
```json
{
  "prompt": "...Token < 70%...Token 70-90%...Token >= 90%..."
}
```

### 修改死循环检测次数

默认检测最近 3 次，可以修改为 5 次：

```json
{
  "prompt": "...检查最近 3 条消息..."
}
```

修改为：
```json
{
  "prompt": "...检查最近 5 条消息..."
}
```

### 禁用自动继续

如果只想检查状态而不自动继续：

```json
{
  "enabled": false
}
```

或修改 prompt 移除"立即继续执行"的指令。

## 📊 监控和日志

### 查看 Hook 执行历史

在 Kiro IDE 中：
1. 打开命令面板 (Ctrl+Shift+P)
2. 搜索 "Show Agent Logs"
3. 查找 "Hook triggered" 消息

### Token 使用监控

Hook 会在输出中显示 token 使用情况：
```
Token 使用: 170K/200K (85%)
```

### 进度跟踪

查看 tasks.md 文件中的任务状态：
```bash
# 统计已完成任务
grep -c "\[x\]" .kiro/specs/*/tasks.md

# 统计未完成任务
grep -c "\[ \]" .kiro/specs/*/tasks.md
```

## 🚀 最佳实践

### 1. 定期检查进度
- 每完成 5-10 个任务后查看 token 使用
- 在接近 80% 时准备新会话

### 2. 保持任务粒度适中
- 每个任务不要太大（避免单个任务消耗过多 token）
- 不要太小（避免任务过多导致管理复杂）

### 3. 及时更新任务状态
- 完成任务后立即更新 tasks.md
- 使用正确的标记格式

### 4. 利用上下文摘要
- 在新会话开始时先阅读摘要
- 摘要包含所有关键信息
- 避免重复已完成的工作

### 5. 监控 Hook 行为
- 注意 Hook 输出的状态信息
- 如果发现异常及时干预
- 必要时手动控制执行流程

## 📞 获取帮助

### 文档资源
- [完整 README](.kiro/hooks/README.md)
- [上下文摘要模板](.kiro/specs/CONTEXT_SUMMARY_TEMPLATE.md)
- [Kiro 官方文档](https://docs.kiro.ai)

### 常见问题
- Hook 配置问题 → 检查 JSON 格式
- 任务不继续 → 查看 token 使用和死循环检测
- 摘要未生成 → 检查目录权限和 token 阈值

### 调试技巧
1. 手动复制 Hook prompt 测试
2. 查看 Agent 日志
3. 验证 tasks.md 格式
4. 检查文件权限

---

**版本**: v3.0  
**最后更新**: 2026-02-10  
**维护者**: Kiro Team
