# SourceSvn MVP 实现设计规格

## 概述

实现 SourceSvn MVP 版本的全部功能：基于 Tauri + Vue 3 + TypeScript + Rust 的轻量级 SVN 图形客户端。

**设计依据**：现有 8 个设计文档（FRAMEWORK.md、API.md、MODELS.md、UI_PROTOTYPE.md、I18N.md、DEVELOPMENT.md、CODE_STYLE.md、TESTING.md）

**实现策略**：后端优先（方案 A），按模块逐步实现

## 1. 项目架构

```
┌─────────────────────────────────────────────────────┐
│                 前端 (Vue 3 + TypeScript)             │
│  App.vue                                            │
│  ├── GlobalTabBar.vue    — 页签管理                  │
│  ├── Toolbar.vue         — 仓库操作按钮              │
│  ├── IconNavBar.vue      — 4个图标导航               │
│  └── <动态视图>                                      │
│      ├── LogView.vue                                │
│      ├── LocalChangesView.vue（整合提交）            │
│      ├── FileBrowserView.vue                        │
│      └── ShelveView.vue                             │
├─────────────────────────────────────────────────────┤
│                 Tauri 桥接层                          │
│  invoke() / listen()                                │
├─────────────────────────────────────────────────────┤
│                 后端 (Rust)                           │
│  ├── commands/          — Tauri 命令（17个）         │
│  ├── svn/               — SVN 服务（6个子模块）      │
│  ├── ai/                — AI 服务（trait + OpenAI）   │
│  ├── shelve/            — Shelve 管理                │
│  ├── config/            — 配置管理（confy）           │
│  └── common/            — 共享类型 + 错误处理         │
└─────────────────────────────────────────────────────┘
```

**关键决策**：
- 后端无状态：所有命令接收 `path` 参数，不持有仓库上下文
- 单实例视图：右侧只有一个动态组件，切换页签刷新数据
- 配置持久化：confy + toml，关闭时保存窗口几何和页签状态

## 2. 后端模块设计

### 2.1 SVN 服务模块

**子模块**：
- `run_svn()` — 底层命令执行，统一超时/错误处理
- `parse_status_xml()` / `parse_log_xml()` / `parse_list_xml()` / `parse_info_xml()` — XML 解析
- `status.rs` — `svn status --xml`
- `log.rs` — `svn log --xml`
- `diff.rs` — `svn diff` / `svn diff -r`
- `commit.rs` — `svn commit` / `svn update` / `svn checkout` / `svn list` / `svn cat`

**Tauri 命令**（9个）：
1. `svn_status(path) -> Vec<FileStatus>`
2. `svn_info(path) -> RepoInfo`
3. `svn_log(path, limit?, from_rev?) -> Vec<LogEntry>`
4. `svn_diff(path, target) -> String`
5. `svn_commit(path, message, files) -> CommitResult`
6. `svn_list(path, revision?, recursive) -> Vec<DirEntry>`
7. `svn_cat(path, revision?) -> String`
8. `svn_checkout(url, dest) -> ()`
9. `svn_update(path) -> UpdateResult`

### 2.2 Shelve 管理模块

**存储路径**：`~/.sourcesvn/shelves/<repo_hash>/`（SHA256 前 12 位）

**Tauri 命令**（4个）：
1. `shelve_save(path, name) -> ()`
2. `shelve_list(path) -> Vec<ShelveInfo>`
3. `shelve_apply(path, name) -> ()`
4. `shelve_delete(path, name) -> ()`

### 2.3 AI 服务模块

**架构**：
- `AiProvider` trait — 可插拔设计
- `OpenAiProvider` — OpenAI 兼容 API，支持流式响应
- 重试策略：指数退避（1s→2s→4s），最多 3 次

**Tauri 命令**（2个）：
1. `generate_commit_message(diff) -> String`
2. `review_changes(diff) -> ()`（通过事件流推送 `review_chunk`）

### 2.4 配置管理模块

**持久化**：confy + toml

**配置结构**（11个子结构体）：
- WindowConfig, AppearanceConfig, SessionConfig, SvnConfig
- AiConfig, DiffConfig, LogConfig, CommitConfig
- FileBrowserConfig, BehaviorConfig, AdvancedConfig

**Tauri 命令**（2个）：
1. `get_config() -> AppConfig`
2. `set_config(config) -> ()`

**版本迁移**：ConfigMigration trait，保留旧值，新项用默认值

## 3. 前端组件设计

### 3.1 状态管理

- `configStore` — 全局配置（AppConfig）
- `tabStore` — 每个页签独立 store（动态 id: `repo-${index}`）
- 视图切换策略："切换即请求"，每次切换重新调用 API

### 3.2 组件树

```
App.vue
├── GlobalTabBar.vue      — 页签栏 + 设置按钮 + 新建页签
├── Toolbar.vue           — 拉取/提交/刷新按钮
├── IconNavBar.vue        — 4个图标（日志/修改/浏览/Shelve）
└── <动态视图>
    ├── LogView.vue       — 表格 + 筛选 + 分页 + 详情展开
    ├── LocalChangesView.vue — 文件列表 + 提交信息 + 差异预览
    ├── FileBrowserView.vue  — 树形目录 + 文件内容预览
    └── ShelveView.vue       — Shelve 列表 + 保存/应用/删除
├── DiffViewer.vue        — 模态框，unified/side-by-side
└── AiReviewPanel.vue     — 流式输出面板
```

### 3.3 交互要点

- 页签：单击切换，双击关闭
- 图标栏：固定 48px 宽度，hover 显示 tooltip
- 本地修改视图：左侧文件列表+提交信息，右侧差异预览
- 差异查看器：模态框方式打开
- 设置页：模态对话框，页签式配置区域

## 4. 实现顺序

### 阶段 1：项目初始化
- create-tauri-app 创建脚手架
- 调整目录结构（按 DEVELOPMENT.md）
- 配置 ESLint/Prettier/rustfmt/.gitignore
- 安装依赖（pinia, reqwest, confy 等）

### 阶段 2：后端基础
- common 模块：共享类型（FileStatus, LogEntry, RepoInfo 等）
- 错误类型定义（AppError → String）

### 阶段 3：SVN 服务
- run_svn() 底层执行
- XML 解析器（status/log/list/info）
- 所有 SVN 相关 Tauri 命令

### 阶段 4：Shelve + AI + Config
- Shelve 管理模块
- AiProvider trait + OpenAiProvider
- AppConfig 结构体 + confy 持久化

### 阶段 5：Tauri 命令层
- 17 个 Tauri 命令（桥接后端到前端）
- 错误码约定

### 阶段 6：前端基础
- TypeScript 类型定义（与 Rust 对应）
- Pinia stores（configStore, tabStore）
- App.vue 骨架

### 阶段 7：前端组件
- GlobalTabBar + IconNavBar + Toolbar
- LogView + LocalChangesView + FileBrowserView + ShelveView
- DiffViewer + AiReviewPanel + SettingsPage

### 阶段 8：集成 + 优化
- 端到端测试
- 性能优化
- 错误处理完善

## 5. 技术栈

- **前端**：Vue 3.5+, TypeScript 5.7+, Vite 6.0+, Pinia
- **后端**：Rust 1.86+, Tauri 2.4+, reqwest, confy, quick-xml
- **工具**：ESLint 9.x, Prettier 3.x, rustfmt, Clippy

## 6. 关键约束

- i18n：MVP 阶段硬编码中文，预留接口
- 最小窗口：800x600
- SVN 最低版本：1.10+（支持 shelve）
- 错误处理：统一 `Result<T, String>` + toast

## 7. 参考文档

- FRAMEWORK.md — 架构设计
- API.md — Tauri 命令 API 契约
- MODELS.md — 数据模型定义
- UI_PROTOTYPE.md — UI 原型图
- DEVELOPMENT.md — 开发环境搭建指南
- CODE_STYLE.md — 代码规范
- TESTING.md — 测试策略
