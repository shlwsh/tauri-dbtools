# PostgreSQL Database Export/Import Tool

一个基于 Tauri 框架的 PostgreSQL 数据库导出和导入工具，支持跨平台独立分发。

## 项目结构

```
pg-db-tool/
├── frontend/         # Bun + React 前端应用
│   ├── src/
│   │   ├── App.tsx   # 主应用组件
│   │   ├── api.ts    # Tauri API 调用封装
│   │   └── index.tsx # 入口文件
│   ├── index.html    # HTML 模板
│   └── package.json  # 依赖配置
├── src-tauri/        # Tauri 后端（Rust）
│   ├── src/
│   │   ├── lib.rs    # Tauri 命令和主逻辑
│   │   └── main.rs   # 入口文件
│   ├── Cargo.toml    # Rust 依赖配置
│   └── tauri.conf.json # Tauri 配置
├── .env              # 数据库配置
└── README.md         # 使用文档
```

## 功能特性

- 数据库导出：将整个 PostgreSQL 数据库导出为 SQL 文件
- 数据库导入：从 SQL 文件导入到指定数据库
- 数据库列表：查看所有可用的数据库
- 原生文件对话框：使用系统原生文件选择器
- 跨平台支持：支持 macOS、Linux 和 Windows
- 独立分发：编译为单个可执行文件，无需额外依赖

## 环境要求

- Rust 1.70+
- Bun 1.0+
- PostgreSQL 12+
- pg_dump 和 psql 命令行工具

## 配置

编辑项目根目录的 `.env` 文件，配置数据库连接信息：

```env
PG_HOST=localhost
PG_PORT=5432
PG_USER=postgres
PG_PASSWORD=postgres
```

## 快速开始

### 开发模式

```bash
# 安装依赖
bun install

# 启动开发服务器
bun run tauri:dev
```

或者使用 Makefile：

```bash
make dev
```

### 构建应用

```bash
# 构建应用
bun run tauri:build
```

或者使用 Makefile：

```bash
make build
```

构建完成后，可执行文件位于：
- macOS: `src-tauri/target/release/bundle/macos/`
- Linux: `src-tauri/target/release/bundle/deb/` 或 `src-tauri/target/release/bundle/appimage/`
- Windows: `src-tauri/target/release/bundle/msi/` 或 `src-tauri/target/release/bundle/nsis/`

## 使用说明

### 导出数据库

1. 启动应用
2. 在"Export Database"部分选择要导出的数据库
3. 点击"Export Database"按钮
4. 导出的 SQL 文件将保存在用户主目录的 `pg-db-tool-exports` 文件夹中

### 导入数据库

1. 在"Import Database"部分输入目标数据库名称
2. 点击"Browse Files"按钮选择 SQL 文件
3. 点击"Import Database"按钮
4. 系统会自动创建数据库并导入数据

## Tauri 命令

应用提供以下 Tauri 命令：

- `export_database(database: String)` - 导出数据库
- `import_database(file_path: String, database: String)` - 导入数据库
- `list_databases()` - 列出所有数据库
- `check_health()` - 检查服务状态
- `get_export_dir_path()` - 获取导出目录路径

## 跨平台编译

### macOS

```bash
bun run tauri:build
```

### Linux

```bash
bun run tauri:build
```

### Windows

```bash
bun run tauri:build
```

## 注意事项

1. 确保系统已安装 `pg_dump` 和 `psql` 命令行工具
2. 导入数据库时，如果目标数据库已存在，将会被删除并重新创建
3. 导出文件命名格式：`{database_name}_{timestamp}.sql`
4. 确保应用有足够的权限访问 PostgreSQL 数据库
5. 导出文件保存在用户主目录的 `pg-db-tool-exports` 文件夹中

## 故障排除

### 应用无法启动
- 检查 PostgreSQL 服务是否运行
- 验证 `.env` 文件中的数据库连接配置是否正确
- 确保 `pg_dump` 和 `psql` 命令在系统 PATH 中

### 导出/导入失败
- 检查数据库用户权限
- 确保有足够的磁盘空间
- 验证 SQL 文件格式是否正确

### 构建失败
- 确保已安装所有系统依赖
- macOS: 需要安装 Xcode 命令行工具
- Linux: 需要安装 webkit2gtk 和其他依赖
- Windows: 需要安装 WebView2

## 技术栈

- **前端**: React + Bun
- **后端**: Rust + Tauri
- **数据库**: PostgreSQL
- **构建工具**: Tauri CLI

## 许可证

MIT License
