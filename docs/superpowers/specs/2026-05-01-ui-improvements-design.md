# UI Improvements Design

Four changes to the SourceSvn UI layer (Vue frontend + minor Rust backend for tab persistence).

---

## 1. Commit后自动刷新日志视图

**Problem:** `submitCommit()` only calls `refreshLocalChanges()`. When switching to log view, stale data shows.

**Fix:** `LocalChangesView.submitCommit()` emits `refresh` event after successful commit. App.vue handles it via existing `handleRefresh()` which calls `refreshCurrentView()`.

**Files:** `LocalChangesView.vue` (add emit), `App.vue` (already handles @refresh).

---

## 2. 工具栏移除提交按钮

**Problem:** Top toolbar has 拉取/提交/刷新. 提交按钮属于本地修改视图，不应出现在全局工具栏。

**Fix:** Remove the commit button and its `@commit` emit from `Toolbar.vue`. Remove `@commit="handleCommit"` from App.vue toolbar usage.

**Files:** `Toolbar.vue`, `App.vue`.

---

## 3. 本地修改视图按钮重新布局

**Current layout:**
- Header: [全选] [已选N个] [拉取] [提交] [刷新]
- Commit section: textarea, stats, [AI生成注释] [取消] [提交]

**New layout:**
- Header: [全选] [已选N个]
- Commit section: textarea, stats, [AI生成注释] [刷新] [取消] [提交]

Changes:
- Remove 拉取 and 提交 from header
- Add 刷新 button to commit-actions row (between AI and 取消)
- Remove the standalone `@pull`/`@commit` emits from the component (App.vue no longer passes them)

**Files:** `LocalChangesView.vue`.

---

## 4. 页签持久化修复

**Problem:** `saveSession()` calls `configStore.saveConfig()` without `await`. `saveConfig()` is async (`invoke('set_config', ...)`). When the app window closes, the process may exit before the async write completes.

**Fix:** Make `saveSession` async and `await configStore.saveConfig()`. Also add `await` in the callers (`openRepo`, `switchView`, `closeTab`). For the `tauri://close-requested` listener, use `event.preventDefault()` + manual `app.exit()` after save completes (Tauri 2.x pattern for graceful shutdown).

**Files:** `App.vue` (make saveSession async, update callers, fix close handler).

---

## Summary of changed files

| File | Changes |
|------|---------|
| `Toolbar.vue` | Remove commit button |
| `LocalChangesView.vue` | Restructure header (remove 拉取/提交), move 刷新 to commit-actions, add emit after commit |
| `App.vue` | Remove toolbar @commit, handle @refresh from LocalChangesView, make saveSession async, fix close handler |

No Rust backend changes needed (tab persistence logic is correct, only the async save timing was broken).
