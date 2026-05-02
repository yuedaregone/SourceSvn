# 拉取结果弹窗设计

## 概述

在 SourceSvn 的拉取（svn update）操作完成后，展示一个居中 Modal 弹窗，列出所有变更文件的状态、路径、修改者，并提供 Diff 查看入口。冲突文件按类型（文件冲突 / Tree 冲突）区分显示。

## 触发逻辑

- **有更新或冲突**：弹出 `PullResultModal`
- **已是最新版本**（无文件变更）：显示 Toast 提示「已是最新版本」

## 后端变更

### 1. 扩展 `UpdateResult` 结构

**文件**: `src-tauri/src/svn/models.rs`

当前 `UpdateResult` 只包含文件路径列表，不含每文件的修改者信息。

```rust
pub struct UpdateResult {
    pub revision: u64,
    pub updated_files: Vec<String>,
    pub merged_files: Vec<String>,
    pub conflicts: Vec<String>,
}
```

需新增 `UpdateFileItem` 结构，并将列表字段改为结构体列表：

```rust
pub struct UpdateFileItem {
    pub path: String,
    pub status: String,  // "A" | "U" | "M" | "C"
    pub author: String,
}

pub struct UpdateResult {
    pub revision: u64,
    pub files: Vec<UpdateFileItem>,
}
```

### 2. 冲突类型区分

SVN update 输出中，冲突有两种形式：

- **文件冲突**（`C`）：两处修改冲突，有合并基，可通过三方合并工具解决
- **Tree 冲突**：目录结构冲突（文件被删除但本地有修改，或重命名冲突），SVN update 输出中通常以 `C` + `!` 或特殊标记出现

解析策略：初版实现先统一分到 `C` 类型，前端统一标为文件冲突（tree conflict 场景较少，后续可迭代）。

### 3. 获取修改者信息

**文件**: `src-tauri/src/svn/update.rs`

在 `svn_update` 函数中，执行完 `svn update` 后，若有文件变更，调用：

```bash
svn log -r {revision} --verbose
```

解析 log 输出中的 `Changed paths:` 段落，提取每文件的作者。将 author 与 `UpdateResult.files` 合并。

若 revision 为 0（已是最新），跳过 log 查询。

### 4. 更新 Tauri command

**文件**: `src-tauri/src/commands/svn.rs`

`svn_update` command 签名不变，返回值自然变为新的 `UpdateResult` 结构。

## 前端变更

### 1. 扩展 TypeScript 类型

**文件**: `src/types/svn.ts`

```typescript
export interface UpdateFileItem {
  path: string
  status: 'A' | 'U' | 'M' | 'C'
  author: string
}

export interface UpdateResult {
  revision: number
  files: UpdateFileItem[]
}
```

### 2. 新增 `PullResultModal.vue`

**文件**: `src/components/PullResultModal.vue`

Props:
- `visible: boolean`
- `result: UpdateResult | null`

Events:
- `close`

布局（从上到下）：

1. **Header**：图标 + 标题「拉取结果」+ 版本号徽标 + 关闭按钮
2. **统计栏**：冲突数、合并数、更新数，带彩色圆点
3. **文件表格**（可滚动）：
   - 列：状态徽标 | 文件路径 | 修改者 | 操作按钮
   - 默认按状态排序：冲突置顶（红色左边框高亮），其次合并（M），最后更新（A/U）
   - 冲突行（`C`）：红色徽标 + 「解决」按钮（点击调用第三方合并工具，当前版本留空）
   - 合并行（`M`）：黄色徽标 + 「查看」按钮（打开 DiffViewer）
   - 更新行（`A`/`U`）：绿色徽标 + 「查看」按钮
4. **Footer**：「关闭」按钮

### 3. 修改 `App.vue` 的 `handlePull()`

**文件**: `src/App.vue`

```typescript
async function handlePull() {
  if (!currentTabStore.value) return
  try {
    const result = await invoke<UpdateResult>('svn_update', { path: currentTabStore.value.repoPath })
    if (result.files.length === 0) {
      toastStore.show('已是最新版本', 'info')
    } else {
      pullResult.value = result
      showPullResult.value = true
    }
    refreshCurrentView()
  } catch (e) {
    console.error('Pull failed:', e)
    toastStore.show('拉取失败', 'error')
  }
}
```

新增 state：
```typescript
const showPullResult = ref(false)
const pullResult = ref<UpdateResult | null>(null)
```

模板中添加 `<PullResultModal>` 组件。

## 文件清单

| 文件 | 变更类型 |
|------|----------|
| `src-tauri/src/svn/models.rs` | 修改 — 新增 `UpdateFileItem`，修改 `UpdateResult` |
| `src-tauri/src/svn/update.rs` | 修改 — 扩展解析逻辑，合并 log 查询 |
| `src-tauri/src/commands/svn.rs` | 无需修改（返回值类型自动跟随） |
| `src/types/svn.ts` | 修改 — 新增 `UpdateFileItem`，修改 `UpdateResult` |
| `src/components/PullResultModal.vue` | 新增 |
| `src/App.vue` | 修改 — 修改 `handlePull()`，添加弹窗状态 |

## 不做的事

- 不实现「解决冲突」功能（按钮点击后留空，仅 console.log）
- 不区分 Tree 冲突（v1 统一为 `C` 文件冲突）
- 不实现从弹窗内点击文件跳转到 DiffViewer（避免弹窗嵌套复杂度，后续迭代）
- 不修改 AiReviewPanel 等现有组件
