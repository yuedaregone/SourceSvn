# UI Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 4 UI issues: post-commit log refresh, remove toolbar commit button, restructure local changes view layout, and fix tab persistence on restart.

**Architecture:** All changes are in 3 Vue frontend files. No Rust backend changes. The tab persistence bug is caused by `saveSession()` not awaiting the async `saveConfig()` call.

**Tech Stack:** Vue 3, TypeScript, Pinia, Tauri 2.x

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/components/Toolbar.vue` | Global toolbar — remove commit button |
| `src/views/LocalChangesView.vue` | Local changes panel — restructure buttons, emit refresh after commit |
| `src/App.vue` | App shell — remove toolbar @commit, fix saveSession async, fix close handler |

---

### Task 1: Remove commit button from Toolbar

**Files:**
- Modify: `src/components/Toolbar.vue`

- [ ] **Step 1: Remove commit button from template**

In `src/components/Toolbar.vue`, remove the commit button (line 4) and the `commit` emit:

```html
<template>
  <div class="toolbar">
    <button @click="$emit('pull')" :disabled="loading">拉取</button>
    <button @click="$emit('refresh')" :disabled="loading">刷新</button>
    <span v-if="loading" class="loading-indicator">处理中...</span>
  </div>
</template>
```

```html
<script setup lang="ts">
defineProps<{
  loading: boolean
}>()

defineEmits<{
  pull: []
  refresh: []
}>()
</script>
```

- [ ] **Step 2: Verify in App.vue that Toolbar usage is updated**

In `src/App.vue`, the Toolbar component currently binds `@commit="handleCommit"`. Remove that line:

```html
    <Toolbar
      v-if="tabs.length > 0"
      :loading="currentTabStore?.loading ?? false"
      @pull="handlePull"
      @refresh="handleRefresh"
    />
```

- [ ] **Step 3: Commit**

```bash
git add src/components/Toolbar.vue src/App.vue
git commit -m "refactor: remove commit button from global toolbar"
```

---

### Task 2: Restructure LocalChangesView layout

**Files:**
- Modify: `src/views/LocalChangesView.vue`

- [ ] **Step 1: Update template — remove 拉取/提交 from header, move 刷新 to commit-actions**

Replace the entire `<template>` section in `src/views/LocalChangesView.vue`:

```html
<template>
  <div class="local-changes-view">
    <div class="left-panel">
      <div class="file-list-header">
        <label class="select-all">
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          <span>全选</span>
        </label>
        <span class="selected-count">已选 {{ selectedPaths.size }} 个文件</span>
      </div>
      <div class="file-list">
        <div
          v-for="file in store.localChanges"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedFile === file.path }"
          @click="selectFile(file)"
        >
          <input
            type="checkbox"
            :checked="selectedPaths.has(file.path)"
            @click.stop="toggleFile(file.path)"
          />
          <span class="status-badge" :class="file.status">{{ file.status[0].toUpperCase() }}</span>
          <span class="file-path">{{ file.path }}</span>
        </div>
        <div v-if="store.localChanges.length === 0" class="empty-list">无本地修改</div>
      </div>
      <div class="commit-section">
        <textarea
          v-model="commitMessage"
          placeholder="提交信息..."
          rows="3"
          class="commit-input"
        ></textarea>
        <div class="commit-stats" v-if="diffStats">
          <span class="stat-add">+{{ diffStats.added }}</span>
          <span class="stat-del">-{{ diffStats.removed }}</span>
        </div>
        <div class="commit-actions">
          <button @click="generateAiMessage" :disabled="aiLoading || selectedPaths.size === 0" class="ai-btn">
            {{ aiLoading ? '生成中...' : 'AI 生成注释' }}
          </button>
          <button @click="$emit('refresh')" class="action-btn">刷新</button>
          <button @click="cancelCommit" class="cancel-btn">取消</button>
          <button @click="submitCommit" :disabled="!canCommit" class="commit-btn">提交</button>
        </div>
        <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
      </div>
    </div>
    <div class="right-panel">
      <pre v-if="diffContent" class="diff-content"><template v-for="(line, i) in coloredLines" :key="i"><span :class="lineClass(line)">{{ line }}</span>
</template></pre>
      <div v-else class="diff-placeholder">点击文件查看差异</div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Update emits declaration — remove pull and commit**

In the `<script setup>` section, change the emits to only include `refresh`:

```typescript
defineEmits<{
  refresh: []
}>()
```

- [ ] **Step 3: Update submitCommit to emit refresh after success**

Replace the `submitCommit` function:

```typescript
async function submitCommit() {
  if (!canCommit.value) return
  errorMessage.value = ''
  try {
    await invoke('svn_commit', {
      path: props.store.repoPath,
      message: commitMessage.value,
      files: Array.from(selectedPaths.value),
    })
    commitMessage.value = ''
    selectedPaths.value = new Set()
    selectedFile.value = ''
    diffContent.value = ''
    await props.store.refreshLocalChanges()
    emit('refresh')
  } catch (e) {
    errorMessage.value = `提交失败: ${e}`
  }
}
```

Add the `emit` variable at the top of the script (after the existing `defineEmits`):

```typescript
const emit = defineEmits<{
  refresh: []
}>()
```

- [ ] **Step 4: Remove unused imports if any**

The `DiffTarget` import is still used in `selectFile`, so no imports need removing.

- [ ] **Step 5: Commit**

```bash
git add src/views/LocalChangesView.vue
git commit -m "refactor: restructure LocalChangesView layout and emit refresh after commit"
```

---

### Task 3: Fix App.vue — remove toolbar @commit, fix saveSession, fix close handler

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Remove @commit from LocalChangesView usage**

In `src/App.vue`, update the LocalChangesView component to only bind `@refresh`:

```html
        <LocalChangesView
          v-if="currentTabStore?.activeView === 'localChanges'"
          :store="currentTabStore!"
          @refresh="handleRefresh"
        />
```

- [ ] **Step 2: Make saveSession async and await saveConfig**

Replace the `saveSession` function:

```typescript
async function saveSession() {
  if (!configStore.config) return
  configStore.config.session.openTabs = tabs.value
  configStore.config.session.activeTabIndex = activeTabIndex.value
  await configStore.saveConfig()
}
```

- [ ] **Step 3: Update all callers of saveSession to await it**

In `openRepo`:
```typescript
function openRepo(path: string) {
  showAddRepo.value = false
  tabs.value.push({ repoPath: path, activeView: 'log' })
  activeTabIndex.value = tabs.value.length - 1
  addRecentRepo(path)
  refreshCurrentView()
  saveSession()
}
```
→ Add `await` and make it async:
```typescript
async function openRepo(path: string) {
  showAddRepo.value = false
  tabs.value.push({ repoPath: path, activeView: 'log' })
  activeTabIndex.value = tabs.value.length - 1
  addRecentRepo(path)
  refreshCurrentView()
  await saveSession()
}
```

Note: `openRepo` is called from `@openRepo="openRepo"` in the template. Since Vue template event handlers don't need async, we leave the template as-is — the async function still runs, just without blocking the template.

In `closeTab`:
```typescript
function closeTab(index: number) {
  const key = `${index}`
  if (tabStores.value[key]) {
    delete tabStores.value[key]
  }
  tabs.value.splice(index, 1)
  if (activeTabIndex.value >= tabs.value.length) {
    activeTabIndex.value = Math.max(0, tabs.value.length - 1)
  }
  saveSession()
}
```
→ Make async and await:
```typescript
async function closeTab(index: number) {
  const key = `${index}`
  if (tabStores.value[key]) {
    delete tabStores.value[key]
  }
  tabs.value.splice(index, 1)
  if (activeTabIndex.value >= tabs.value.length) {
    activeTabIndex.value = Math.max(0, tabs.value.length - 1)
  }
  await saveSession()
}
```

In `switchView`:
```typescript
function switchView(view: ActiveView) {
  if (!currentTabStore.value) return
  currentTabStore.value.activeView = view
  const tab = tabs.value[activeTabIndex.value]
  if (tab) {
    tab.activeView = view
    saveSession()
  }
  refreshCurrentView()
}
```
→ Make async and await:
```typescript
async function switchView(view: ActiveView) {
  if (!currentTabStore.value) return
  currentTabStore.value.activeView = view
  const tab = tabs.value[activeTabIndex.value]
  if (tab) {
    tab.activeView = view
    await saveSession()
  }
  refreshCurrentView()
}
```

- [ ] **Step 4: Fix the close handler to await saveSession before exiting**

Replace the close handler setup in `onMounted`:

```typescript
  window.addEventListener('tauri://close-requested', saveSession)
```
→
```typescript
  window.addEventListener('tauri://close-requested', async () => {
    await saveSession()
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    getCurrentWindow().destroy()
  })
```

- [ ] **Step 5: Verify build compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no new errors (only pre-existing warnings).

- [ ] **Step 6: Commit**

```bash
git add src/App.vue
git commit -m "fix: await saveSession to ensure tabs persist on app close"
```

---

### Task 4: End-to-end verification

- [ ] **Step 1: Build and run the app**

```bash
cd src-tauri && cargo tauri dev
```

- [ ] **Step 2: Verify requirement 1 — commit refreshes log**

1. Open a repo with local changes
2. Switch to local changes view, select a file, write a commit message, click 提交
3. Switch to log view → the new commit should appear immediately without clicking 拉取

- [ ] **Step 3: Verify requirement 2 — no commit button in toolbar**

1. Confirm the top toolbar only shows 拉取 and 刷新 (no 提交 button)

- [ ] **Step 4: Verify requirement 3 — restructured layout**

1. In local changes view, confirm header only shows 全选 and 已选N个文件
2. Confirm commit section shows: AI生成注释, 刷新, 取消, 提交 buttons

- [ ] **Step 5: Verify requirement 4 — tab persistence**

1. Open 2 repos as tabs
2. Close the app (click X)
3. Reopen the app → both tabs should be restored with the correct active tab

- [ ] **Step 6: Final commit if any fixes were needed**
