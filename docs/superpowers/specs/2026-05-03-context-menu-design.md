# Context Menu Design Spec

**Date:** 2026-05-03
**Status:** Approved

## Overview

为 SourceSvn 添加全面的右键上下文菜单，覆盖日常 SVN 操作。当前项目没有任何右键菜单实现，且部分常用 SVN 命令在后端缺失。

---

## 需要新增的后端命令

| 命令 | SVN CLI | 说明 | 优先级 |
|------|---------|------|--------|
| `svn_revert` | `svn revert <paths>` | 撤销本地修改 | 高 |
| `svn_add` | `svn add <paths>` | 将未版本化文件加入版本控制 | 高 |
| `svn_delete` | `svn delete [--keep-local] <paths>` | 调度删除（支持保留本地文件） | 高 |
| `svn_blame` | `svn blame <path>` | 文件逐行注释 | 中 |
| `svn_update_to_revision` | `svn update -r <rev> <path>` | 更新工作副本到指定版本 | 中 |

所有命令复用现有 `run_svn_async_in_dir` 运行器和 GBK 编码处理逻辑。

---

## 右键菜单位置和内容

### 1. LocalChangesView — 文件列表右键（最核心）

| 菜单项 | 说明 | 图标 | 需要新后端 |
|--------|------|------|-----------|
| Diff | 查看选中文件的差异 | — | 否 |
| ─── 分隔线 ─── | | | |
| Revert | 撤销本地修改，需二次确认 | RotateCcw | 是 |
| Add | 将未版本化文件加入版本控制 | Plus | 是 |
| Delete | 调度删除，区分"仅磁盘删除"和"版本控制删除" | Trash2 | 是 |
| ─── 分隔线 ─── | | | |
| Open with Editor | 用外部编辑器打开 | ExternalLink | 否 |
| Show in Explorer | 在系统文件管理器中打开 | FolderOpen | 否 |
| ─── 分隔线 ─── | | | |
| Copy Path | 复制文件相对路径 | Copy | 否 |
| Copy Absolute Path | 复制文件绝对路径 | Copy | 否 |
| ─── 分隔线 ─── | | | |
| Select All | 选中所有文件 | CheckSquare | 否 |
| Deselect All | 取消所有选中 | Square | 否 |

**Delete 子菜单行为：**
- "Delete (schedule only)" → `svn delete <path>` — 从版本控制调度删除，下次提交生效
- "Delete (keep local file)" → `svn delete --keep-local <path>` — 从版本控制调度删除但保留本地文件
- "Remove from disk" → 直接调用 Tauri fs API 删除文件，不涉及 SVN 命令

---

### 2. FileBrowserView — 目录树/文件右键

**文件节点菜单：**

| 菜单项 | 说明 | 需要新后端 |
|--------|------|-----------|
| Open with Editor | 打开文件 | 否 |
| Show in Explorer | 打开文件所在目录 | 否 |
| ─── 分隔线 ─── | | |
| Show Log | 查看该文件的 SVN 日志（跳转 LogView） | 否 |
| Show Blame | 查看文件逐行注释 | 是 |
| ─── 分隔线 ─── | | |
| Copy Path | 复制相对路径 | 否 |
| Copy Absolute Path | 复制绝对路径 | 否 |

**目录节点菜单：**

| 菜单项 | 说明 | 需要新后端 |
|--------|------|-----------|
| Show in Explorer | 打开目录 | 否 |
| ─── 分隔线 ─── | | |
| Show Log | 查看目录日志 | 否 |
| Update to Revision... | 更新到指定版本 | 是 |
| ─── 分隔线 ─── | | |
| Cleanup | 清理此目录 | 否 |
| Copy Path | 复制路径 | 否 |
| Copy Absolute Path | 复制绝对路径 | 否 |

---

### 3. LogView — 日志条目右键

| 菜单项 | 说明 | 需要新后端 |
|--------|------|-----------|
| Show Changes | 查看该版本的变更（展开详情） | 否 |
| Copy Revision Number | 复制版本号 | 否 |
| ─── 分隔线 ─── | | |
| Update to Revision... | 将 WC 更新到此版本 | 是 |
| Revert to Revision | 回退 WC 到此版本（svn merge -r HEAD:<rev>） | 是 |

---

### 4. ShelveView — Shelve 条目右键

| 菜单项 | 说明 | 需要新后端 |
|--------|------|-----------|
| Apply | 应用此补丁 | 否 |
| View Diff | 查看补丁差异 | 否（读 patch 文件） |
| ─── 分隔线 ─── | | |
| Rename | 重命名 | 是（文件重命名） |
| Delete | 删除此补丁 | 否 |

---

### 5. GlobalTabBar — 标签页右键

| 菜单项 | 说明 |
|--------|------|
| Close Tab | 关闭当前标签 |
| Close Other Tabs | 关闭其他标签 |
| Close Tabs to the Right | 关闭右侧标签 |
| ─── 分隔线 ─── | |
| Copy Repository Path | 复制仓库路径 |
| Open in Explorer | 打开仓库目录 |

---

## 前端组件设计

### ContextMenu 组件

新建 `src/components/ContextMenu.vue`，通用右键菜单组件：

- Props: `items: MenuItem[]`, `visible: boolean`, `x: number`, `y: number`
- MenuItem 类型: `{ label, icon?, action?, divider?, disabled?, submenu? }`
- 使用 Tauri `@tauri-apps/plugin-shell` 的 `open()` 实现外部打开
- 使用 Tauri `@tauri-apps/api/clipboard` 实现复制路径
- 使用 Tauri `@tauri-apps/plugin-dialog` 实现确认对话框
- 使用 Tauri `@tauri-apps/plugin-fs` 实现磁盘删除

---

## Settings 扩展

在 `AppConfig` 的 `general` 字段增加：

```typescript
externalEditor?: string  // 外部编辑器可执行文件路径，空字符串则使用系统默认关联
```

对应 Rust `GeneralConfig` 同步添加。

---

## i18n 需求

新增键值覆盖所有菜单项文本（中英双语），放在 `contextMenu` 命名空间下。
