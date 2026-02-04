# 文件选择对话框测试

## 问题
选择文件时，导出的 `.backup` 文件看不到，文件类型过滤不正确。

## 原因
文件选择器的过滤器配置为 `['gz', 'sql']`，但现在使用 pg_dump 导出的文件格式是 `.backup`。

## 解决方案
已更新 `frontend/src/App.tsx` 中的文件过滤器配置：

```typescript
filters: [{
  name: 'PostgreSQL Backup Files',
  extensions: ['backup', 'sql', 'gz']  // 添加了 'backup'
}]
```

## 测试步骤

### 1. 导出数据库
1. 在应用中选择一个数据库（如 `personnel_db`）
2. 点击 "Export Database" 按钮
3. 等待导出完成
4. 记下导出文件路径（显示在成功消息中）

### 2. 验证导出文件
打开导出目录：
- Windows: `C:\Users\{用户名}\pg-db-tool-exports\`
- 应该看到类似 `personnel_db_20260204_152903.backup` 的文件

### 3. 测试文件选择
1. 在应用的 "Import Database" 部分
2. 点击 "Browse Files" 按钮
3. 文件选择对话框应该显示：
   - 文件类型过滤器：**PostgreSQL Backup Files**
   - 支持的扩展名：`.backup`, `.sql`, `.gz`
4. 导航到导出目录
5. 应该能看到所有 `.backup` 文件

### 4. 测试导入
1. 选择一个 `.backup` 文件
2. 输入目标数据库名称（如 `test_import`）
3. 点击 "Import Database" 按钮
4. 等待导入完成
5. 检查数据库列表，应该看到新创建的数据库

## 支持的文件格式

现在文件选择器支持以下格式：

1. **`.backup`** - PostgreSQL 自定义格式（推荐）
   - pg_dump -F c 生成
   - 压缩格式
   - 包含所有数据和结构

2. **`.sql`** - 纯 SQL 文本格式
   - pg_dump -F p 生成
   - 未压缩
   - 可读性好

3. **`.gz`** - 压缩的 SQL 文件
   - 通常是 .sql.gz
   - 需要解压后才能查看

## 当前导出格式

应用当前使用的导出格式：
```bash
pg_dump -F c  # 自定义格式（.backup）
```

特点：
- ✅ 自动压缩
- ✅ 快速导入
- ✅ 支持并行恢复
- ✅ 可选择性恢复

## 如果仍然看不到文件

### 检查 1：确认文件存在
```powershell
# Windows
Get-ChildItem "$env:USERPROFILE\pg-db-tool-exports" -Filter "*.backup"

# 应该显示所有 .backup 文件
```

### 检查 2：确认文件扩展名
```powershell
# 查看文件详细信息
Get-ChildItem "$env:USERPROFILE\pg-db-tool-exports" | Select-Object Name, Extension, Length
```

### 检查 3：手动测试文件选择器
在浏览器开发者工具控制台中运行：
```javascript
import { open } from '@tauri-apps/plugin-dialog';

const result = await open({
  multiple: false,
  filters: [{
    name: 'PostgreSQL Backup Files',
    extensions: ['backup', 'sql', 'gz']
  }]
});

console.log('Selected file:', result);
```

### 检查 4：查看应用日志
```powershell
# 查看今天的日志
Get-Content "$env:USERPROFILE\pg-db-tool-logs\pg-db-tool_$(Get-Date -Format 'yyyyMMdd').log" -Tail 20
```

## 预期结果

修复后，文件选择对话框应该：
1. ✅ 显示 "PostgreSQL Backup Files" 作为文件类型
2. ✅ 能够看到所有 `.backup` 文件
3. ✅ 能够看到所有 `.sql` 文件
4. ✅ 能够看到所有 `.gz` 文件
5. ✅ 默认过滤器选中 "PostgreSQL Backup Files"

## 额外说明

如果需要支持其他格式，可以修改 `frontend/src/App.tsx` 中的过滤器配置：

```typescript
filters: [
  {
    name: 'PostgreSQL Backup Files',
    extensions: ['backup']
  },
  {
    name: 'SQL Files',
    extensions: ['sql']
  },
  {
    name: 'Compressed Files',
    extensions: ['gz']
  },
  {
    name: 'All Files',
    extensions: ['*']
  }
]
```

这样用户可以在多个过滤器之间切换。
