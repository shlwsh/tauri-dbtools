# PostgreSQL Database Tool - Vue3 Frontend

这是 PostgreSQL Database Tool 的 Vue3 前端实现，提供现代化的用户界面和丰富的数据库管理功能。

## 技术栈

- **Bun 1.0+** - 快速的 JavaScript 运行时和包管理器
- **Vue 3.4+** - 使用 Composition API 和 `<script setup>` 语法
- **TypeScript 5.0+** - 提供类型安全
- **Vite 5.0+** - 快速的开发服务器和构建工具
- **Naive UI** - 现代化的 Vue3 UI 组件库
- **Pinia** - Vue3 官方推荐的状态管理库
- **Vue Router 4** - 路由管理
- **Bun Test** - 内置测试框架

## 快速开始

### 安装依赖

```bash
bun install
```

### 开发模式

```bash
bun run dev
```

应用将在 `http://localhost:8200` 启动。

### 构建生产版本

```bash
bun run build
```

构建产物将输出到 `dist` 目录。

### 运行测试

```bash
# 运行所有测试
bun test

# 监听模式
bun test --watch

# 生成覆盖率报告
bun test --coverage
```

### 代码检查和格式化

```bash
# ESLint 检查和修复
bun run lint

# Prettier 格式化
bun run format
```

## 项目结构

```
frontend/
├── src/
│   ├── assets/              # 静态资源
│   │   └── styles/          # 全局样式
│   │       ├── variables.css    # CSS 变量
│   │       ├── themes.css       # 主题样式
│   │       └── global.css       # 全局样式
│   │
│   ├── components/          # 通用组件
│   │   ├── common/          # 基础通用组件
│   │   ├── database/        # 数据库相关组件
│   │   └── explorer/        # 资源管理器组件
│   │
│   ├── views/               # 页面组件
│   │   ├── Home.vue
│   │   ├── DatabaseExport.vue
│   │   ├── DatabaseImport.vue
│   │   ├── DatabaseExplorer.vue
│   │   └── Settings.vue
│   │
│   ├── api/                 # API 调用层
│   │   ├── base.ts          # 基础 API 调用
│   │   ├── config.ts        # 配置相关 API
│   │   ├── database.ts      # 数据库相关 API
│   │   └── explorer.ts      # 资源管理器 API
│   │
│   ├── stores/              # Pinia 状态管理
│   │   ├── theme.ts         # 主题状态
│   │   ├── config.ts        # 配置状态
│   │   └── database.ts      # 数据库状态
│   │
│   ├── composables/         # 组合式函数
│   │   ├── useTheme.ts      # 主题相关
│   │   ├── useConfig.ts     # 配置相关
│   │   └── useNotification.ts # 通知相关
│   │
│   ├── types/               # TypeScript 类型定义
│   │   ├── common.ts        # 通用类型
│   │   ├── config.ts        # 配置类型
│   │   └── database.ts      # 数据库类型
│   │
│   ├── utils/               # 工具函数
│   │   ├── storage.ts       # 本地存储
│   │   ├── validation.ts    # 表单验证
│   │   └── format.ts        # 格式化工具
│   │
│   ├── router/              # 路由配置
│   │   └── index.ts
│   │
│   ├── App.vue              # 根组件
│   └── main.ts              # 应用入口
│
├── public/                  # 公共资源
├── index.html
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
└── package.json
```

## 命名约定

### 文件命名
- **组件文件**: PascalCase（如 `AppLayout.vue`）
- **工具文件**: camelCase（如 `storage.ts`）
- **类型文件**: camelCase（如 `database.ts`）

### 代码命名
- **组件名**: PascalCase
- **函数名**: camelCase
- **常量**: UPPER_SNAKE_CASE
- **接口/类型**: PascalCase

## 开发指南

### 安装依赖

```bash
bun install
```

### 开发模式

```bash
bun run dev
```

应用将在 `http://localhost:8200` 启动。

### 构建生产版本

```bash
bun run build
```

构建产物将输出到 `dist` 目录。

### 运行测试

```bash
# 运行所有测试
bun test

# 监听模式
bun test --watch

# 生成覆盖率报告
bun test --coverage
```

### 代码检查和格式化

```bash
# ESLint 检查和修复
bun run lint

# Prettier 格式化
bun run format
```

## 功能模块

### 1. 主题系统
- 支持亮色和暗色主题
- 主题偏好自动保存到本地存储
- 应用启动时自动加载上次选择的主题

### 2. 配置管理
- 管理多个数据库连接配置
- 支持创建、编辑、删除连接配置
- 支持设置默认连接
- 配置自动保存到本地存储

### 3. 数据库导出
- 选择连接和数据库
- 导出数据库到文件
- 显示导出结果和文件路径

### 4. 数据库导入
- 选择连接和备份文件
- 支持 .backup、.sql 和 .gz 格式
- 导入数据库
- 显示导入结果

### 5. 数据库资源管理器
- 浏览数据库、表和数据
- 支持分页显示
- 支持 CRUD 操作（创建、读取、更新、删除记录）

## 与后端集成

前端通过 Tauri API 与 Rust 后端通信。所有 API 调用都通过 `src/api/base.ts` 中的 `invokeCommand` 函数进行，该函数提供统一的错误处理。

### API 调用示例

```typescript
import { invokeCommand } from '@/api/base';

const response = await invokeCommand<string[]>('list_databases', {
  connection: connectionConfig,
});

if (response.success) {
  console.log('Databases:', response.data);
} else {
  console.error('Error:', response.message);
}
```

## 状态管理

使用 Pinia 进行状态管理，主要的 store 包括：

- **themeStore**: 管理主题状态
- **configStore**: 管理连接配置
- **databaseStore**: 管理当前数据库状态

### Store 使用示例

```typescript
import { useThemeStore } from '@/stores/theme';

const themeStore = useThemeStore();
themeStore.toggleTheme();
```

## 路由

应用使用 Vue Router 进行页面导航，路由配置在 `src/router/index.ts` 中。

### 可用路由

- `/` - 首页
- `/export` - 数据库导出
- `/import` - 数据库导入
- `/explorer` - 数据库资源管理器
- `/settings` - 配置管理

## 样式系统

使用 CSS 变量进行主题定制，所有颜色、间距、圆角等都定义在 `src/assets/styles/variables.css` 中。

### 使用 CSS 变量

```css
.my-component {
  background-color: var(--color-background);
  color: var(--color-text);
  padding: var(--spacing-md);
  border-radius: var(--radius-md);
}
```

## 测试策略

项目采用双重测试策略：

1. **单元测试**: 验证特定示例、边缘情况和错误条件
2. **属性测试**: 验证跨所有输入的通用属性

所有测试文件使用 `.spec.ts` 或 `.test.ts` 后缀。

## 贡献指南

1. 遵循项目的命名约定和目录结构
2. 为新功能编写测试
3. 使用 TypeScript 类型注解
4. 运行 `npm run lint` 和 `npm run format` 确保代码质量
5. 更新相关文档

## 许可证

MIT
