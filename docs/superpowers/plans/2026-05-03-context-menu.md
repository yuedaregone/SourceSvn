# Context Menu Development Plan

**Date:** 2026-05-03
**Spec:** `docs/superpowers/specs/2026-05-03-context-menu-design.md`

## 1. Backend — 新增 SVN 命令

**文件:** `src-tauri/src/svn/mod.rs`, `src-tauri/src/commands/svn.rs`, `src-tauri/src/svn/models.rs`

新增 5 个 Tauri command，每个包含完整的 XML 解析/执行逻辑：

| 命令 | 函数签名 | 返回类型 |
|------|---------|---------|
| `svn_revert` | `svn_revert(path: String, paths: Vec<String>)` | `Vec<String>` — 已还原的文件列表 |
| `svn_add` | `svn_add(path: String, paths: Vec<String>)` | `Vec<String>` — 已添加的文件列表 |
| `svn_delete` | `svn_delete(path: String, paths: Vec<String>, keep_local: bool)` | `Vec<String>` — 已删除的文件列表 |
| `svn_blame` | `svn_blame(path: String, revision: Option<i32>)` | `Vec<BlameEntry>` — 新 model |
| `svn_update_to_revision` | `svn_update_to_revision(path: String, revision: i32)` | `UpdateResult` — 复用现有 model |

- `BlameEntry`: `{ revision: i32, author: String, line: String }`
- 所有命令复用 `run_svn_async_in_dir`
- 在 `lib.rs` 的 `invoke_handler` 中注册新命令

**验证:** `cargo build` 编译通过

---

## 2. Backend — 新增文件删除命令

**文件:** `src-tauri/src/commands/svn.rs`（或新建 `commands/fs.rs`）

新增 `delete_files_from_disk(path: String, paths: Vec<String>)` 命令，直接调用 `std::fs::remove_file` 删除本地文件（不经过 SVN），返回已删除路径列表。

**验证:** `cargo build` 编译通过

---

## 3. Config — 添加 externalEditor 字段

**文件:** `src-tauri/src/common/mod.rs`, `src/types/config.ts`

- Rust `GeneralConfig`（或新建 `EditorConfig`）: 增加 `external_editor: Option<String>` 字段
- TS `AppConfig`: 增加 `externalEditor?: string` 字段
- Default: `None` / `undefined`

**验证:** `cargo build` 通过，TypeScript 编译无类型错误

---

## 4. Frontend — 通用 ContextMenu 组件

**新建文件:** `src/components/ContextMenu.vue`

```typescript
interface MenuItem {
  label: string
  icon?: Component
  action?: () => void
  disabled?: boolean
  divider?: boolean
}
```

- Props: `visible`, `x`, `y`, `items: MenuItem[]`
- 点击菜单外区域自动关闭
- 点击菜单项后自动关闭
- 支持 `max-height` + 滚动（菜单过长时）
- 菜单出现时若超出视口边界自动调整位置

**验证:** 手动测试 — 在任意空白区域右键可弹出测试菜单

---

## 5. i18n — 新增 contextMenu 翻译

**文件:** `src/locales/en-US.ts`, `src/locales/zh-CN.ts`

新增 `contextMenu` 命名空间，覆盖所有菜单项：
- diff, revert, revertConfirm, add, delete, deleteScheduleOnly, deleteKeepLocal, deleteFromDisk
- openWithEditor, showInExplorer
- copyPath, copyAbsPath, copySuccess
- showLog, showBlame, updateToRevision, revertToRevision, cleanup
- closeTab, closeOtherTabs, closeTabsToRight, copyRepoPath, openInExplorer
- viewDiff, rename, applyShelve

**验证:** TypeScript 编译通过

---

## 6. LocalChangesView — 文件列表右键菜单

**文件:** `src/views/LocalChangesView.vue`

- 在文件列表项上绑定 `@contextmenu.prevent`
- 菜单项根据文件状态动态禁用（如：unversioned 文件禁用 Revert）
- Diff: 选中文件时触发 `emit('viewDiff', file)` 现有逻辑
- Revert: 调用 `confirmBeforeRevert` 配置决定是否弹确认框 → `invoke('svn_revert')` → `refreshLocalChanges()`
- Add: `invoke('svn_add')` → `refreshLocalChanges()`
- Delete (schedule): `invoke('svn_delete', { keepLocal: false })` → `refreshLocalChanges()`
- Delete (keep local): `invoke('svn_delete', { keepLocal: true })` → `refreshLocalChanges()`
- Delete (disk): `invoke('delete_files_from_disk')` → `refreshLocalChanges()`
- Open with Editor: 从 config 读取 `externalEditor` → `shell.open()`，未配置则用系统默认
- Show in Explorer: `shell.open(文件所在目录)`
- Copy Path / Absolute Path: `navigator.clipboard.writeText()` → Toast 成功
- Select All / Deselect All: 操作现有 `selectedFiles` 状态

**验证:** 右键文件列表项弹出菜单，各操作可执行

---

## 7. FileBrowserView — 目录树/文件右键菜单

**文件:** `src/views/FileBrowserView.vue`

- 在目录树节点上绑定 `@contextmenu.prevent`
- 区分文件节点和目录节点菜单项（Blame 仅文件，Cleanup 仅目录）
- Show Log: 切换到 LogView（未来可扩展为过滤特定文件）
- Show Blame: `invoke('svn_blame')` → 新建简单弹窗/面板显示 blame 结果
- Update to Revision: 弹输入框让用户输入版本号 → `invoke('svn_update_to_revision')`
- 其他项同理复用工具函数

**验证:** 文件/目录节点各自弹出正确菜单

---

## 8. LogView — 日志条目右键菜单

**文件:** `src/views/LogView.vue`

- 在日志条目行上绑定 `@contextmenu.prevent`
- Show Changes: 触发现有展开逻辑
- Copy Revision Number: `navigator.clipboard.writeText(revision)`
- Update to Revision: `invoke('svn_update_to_revision', { revision: entry.revision })`
- Revert to Revision: `invoke('svn_update_to_revision', { revision: entry.revision - 1 })` + 确认对话框

**验证:** 日志行右键菜单可操作

---

## 9. ShelveView — Shelve 条目右键菜单

**文件:** `src/views/ShelveView.vue`

- 在 shelve 条目行上绑定 `@contextmenu.prevent`
- Apply / Delete: 复用现有函数
- View Diff: 读取 patch 文件内容，打开 DiffViewer 弹窗
- Rename: 弹输入框，文件重命名后刷新列表

**验证:** Shelve 条目右键菜单可操作

---

## 10. GlobalTabBar — 标签页右键菜单

**文件:** `src/components/GlobalTabBar.vue`

- 在标签按钮上绑定 `@contextmenu.prevent`
- Close Tab: 复用现有关闭逻辑
- Close Other Tabs: 遍历 `session.openTabs` 移除非当前标签
- Close Tabs to the Right: 截取当前标签之后的部分
- Copy Repository Path: 复制 `tab.repoPath`
- Open in Explorer: `shell.open(tab.repoPath)`

**验证:** 标签右键菜单关闭功能正常

---

## 执行顺序

```
1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10
```

每个步骤完成后执行对应的验证，确认通过再进行下一步。
