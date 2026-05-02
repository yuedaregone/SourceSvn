# Pull Result Modal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show a modal dialog after SVN update listing changed files with status, author, and conflict highlights.

**Architecture:** Extend the existing `UpdateResult` struct to include per-file author info by merging log data after `svn update`. Create a new `PullResultModal.vue` component following the project's existing modal pattern. Wire it into `App.vue` via the existing `handlePull()` flow.

**Tech Stack:** Rust (serde), Vue 3 Composition API, TypeScript, Tauri 2.x invoke

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src-tauri/src/svn/models.rs` | Modify | Add `UpdateFileItem`, reshape `UpdateResult` |
| `src-tauri/src/svn/update.rs` | Modify | Parse status chars, merge log author info |
| `src-tauri/src/commands/svn.rs` | No change | Return type auto-follows `UpdateResult` |
| `src/types/svn.ts` | Modify | Add `UpdateFileItem`, reshape `UpdateResult` |
| `src/components/PullResultModal.vue` | Create | Modal component |
| `src/App.vue` | Modify | Wire modal state and handlePull logic |

---

### Task 1: Extend Rust `UpdateResult` model

**Files:**
- Modify: `src-tauri/src/svn/models.rs:108-114`

- [ ] **Step 1: Add `UpdateFileItem` struct and reshape `UpdateResult`**

In `src-tauri/src/svn/models.rs`, replace the existing `UpdateResult` block (lines 108-114) with:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileItem {
    pub path: String,
    pub status: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    pub revision: u64,
    pub files: Vec<UpdateFileItem>,
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors (update.rs and commands/svn.rs will show errors — that's expected, fixed in Task 2).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/svn/models.rs
git commit -m "feat(backend): add UpdateFileItem and reshape UpdateResult"
```

---

### Task 2: Extend `update.rs` — parse status and merge author info

**Files:**
- Modify: `src-tauri/src/svn/update.rs`

- [ ] **Step 1: Add `log` module import and rewrite `parse_update_output`**

Replace the entire content of `src-tauri/src/svn/update.rs` with:

```rust
use crate::common::AppError;
use crate::svn::models::{UpdateFileItem, UpdateResult};

/// Parse svn update output into (revision, files_without_authors).
/// Each file entry is (path, status_char).
fn parse_update_output(output: &str) -> Result<(u64, Vec<(String, String)>), AppError> {
    let mut revision: u64 = 0;
    let mut files = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // "Updated revision 105."
        if let Some(rest) = trimmed.strip_prefix("Updated revision ") {
            if let Some(rev_str) = rest.strip_suffix('.') {
                if let Ok(rev) = rev_str.parse::<u64>() {
                    revision = rev;
                }
            }
        }

        // Status lines: "A    new_file.rs" or "U   +  existing.rs"
        if trimmed.len() >= 2 {
            let status_char = trimmed.as_bytes()[0];
            let path_part = if trimmed.as_bytes().get(1) == Some(&b' ') {
                let after_status = &trimmed[1..];
                if let Some(pos) = after_status.find(|c: char| c.is_alphabetic()) {
                    after_status[pos..].trim()
                } else {
                    after_status.trim()
                }
            } else {
                &trimmed[1..]
            };

            let path = path_part.trim();
            if path.is_empty() || path.starts_with("Updating") || path.starts_with("Summary") {
                continue;
            }

            match status_char {
                b'A' | b'U' | b'M' | b'C' => {
                    let status = (status_char as char).to_string();
                    files.push((path.to_string(), status));
                }
                _ => {}
            }
        }
    }

    Ok((revision, files))
}

/// Query svn log for a single revision to get the author per changed file.
fn fetch_authors_for_revision(
    path: &str,
    revision: u64,
    timeout_secs: u64,
) -> Result<std::collections::HashMap<String, String>, AppError> {
    let rev_str = revision.to_string();
    let xml = crate::svn::run_svn_utf8_async(
        &["log", "--xml", "-v", "-r", &rev_str, path],
        timeout_secs,
    )?;
    let entries = crate::svn::log::parse_log_xml(&xml)?;

    let mut author_map = std::collections::HashMap::new();
    if let Some(entry) = entries.into_iter().next() {
        if let Some(changed_paths) = entry.changed_paths {
            for cp in changed_paths {
                author_map.insert(cp.path, entry.author.clone());
            }
        }
    }
    Ok(author_map)
}

pub async fn svn_update(path: &str, timeout_secs: u64) -> Result<UpdateResult, AppError> {
    let output = crate::svn::run_svn_async_in_dir(&["update"], timeout_secs, Some(path)).await?;
    let (revision, raw_files) = parse_update_output(&output)?;

    if raw_files.is_empty() || revision == 0 {
        return Ok(UpdateResult {
            revision,
            files: Vec::new(),
        });
    }

    let author_map = fetch_authors_for_revision(path, revision, timeout_secs)
        .unwrap_or_default();

    let files = raw_files
        .into_iter()
        .map(|(path, status)| {
            let author = author_map.get(&path).cloned().unwrap_or_default();
            UpdateFileItem {
                path,
                status,
                author,
            }
        })
        .collect();

    Ok(UpdateResult { revision, files })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_update_output_with_changes() {
        let output = "Updating '.':\nA    new_file.rs\nU    existing.rs\nUpdated revision 105.\n";
        let (revision, files) = parse_update_output(output).unwrap();
        assert_eq!(revision, 105);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], ("new_file.rs".to_string(), "A".to_string()));
        assert_eq!(files[1], ("existing.rs".to_string(), "U".to_string()));
    }

    #[test]
    fn test_parse_update_output_no_changes() {
        let output = "Updating '.':\nAt revision 100.\n";
        let (revision, files) = parse_update_output(output).unwrap();
        assert_eq!(revision, 0);
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_update_output_conflicts() {
        let output = "Updating '.':\nC    conflict.rs\nA    ok.rs\nUpdated revision 200.\n";
        let (revision, files) = parse_update_output(output).unwrap();
        assert_eq!(revision, 200);
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|(p, s)| p == "conflict.rs" && s == "C"));
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles cleanly.

- [ ] **Step 3: Run existing tests**

Run: `cd src-tauri && cargo test svn::update`
Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/svn/update.rs
git commit -m "feat(backend): merge log author info into UpdateResult"
```

---

### Task 3: Extend frontend TypeScript types

**Files:**
- Modify: `src/types/svn.ts:63-68`

- [ ] **Step 1: Replace the `UpdateResult` interface**

In `src/types/svn.ts`, replace lines 63-68 with:

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

- [ ] **Step 2: Commit**

```bash
git add src/types/svn.ts
git commit -m "feat(types): extend UpdateResult with UpdateFileItem"
```

---

### Task 4: Create `PullResultModal.vue`

**Files:**
- Create: `src/components/PullResultModal.vue`

- [ ] **Step 1: Create the component**

Create `src/components/PullResultModal.vue` with the following content:

```vue
<template>
  <div v-if="visible" class="dialog-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <div class="modal-header-left">
          <div class="modal-icon">
            <Download :size="16" />
          </div>
          <span class="modal-title">拉取结果</span>
          <span class="modal-rev" v-if="result">r{{ result.revision }}</span>
        </div>
        <button class="close-btn" @click="$emit('close')">&times;</button>
      </div>

      <div class="stats-bar" v-if="result">
        <div class="stat-item">
          <span class="stat-dot conflict" />
          <span class="stat-label">冲突</span>
          <span class="stat-value conflict-value">{{ conflictCount }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-dot merged" />
          <span class="stat-label">合并</span>
          <span class="stat-value merged-value">{{ mergedCount }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-dot added" />
          <span class="stat-label">更新</span>
          <span class="stat-value added-value">{{ updatedCount }}</span>
        </div>
      </div>

      <div class="file-table-wrapper" v-if="result">
        <table class="file-table">
          <thead>
            <tr>
              <th style="width: 44px"></th>
              <th style="width: 44px">状态</th>
              <th>文件路径</th>
              <th style="width: 100px">修改者</th>
              <th style="width: 72px; text-align: center">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(file, idx) in sortedFiles"
              :key="file.path"
              :class="{ 'row-conflict': file.status === 'C' }"
            >
              <td />
              <td>
                <span class="status-badge" :class="statusClass(file.status)">
                  {{ file.status }}
                </span>
              </td>
              <td class="file-path" :title="file.path">{{ file.path }}</td>
              <td class="file-author">{{ file.author }}</td>
              <td style="text-align: center">
                <button
                  v-if="file.status === 'C'"
                  class="diff-btn resolve-btn"
                  @click="handleResolve(file)"
                >
                  解决
                </button>
                <button
                  v-else
                  class="diff-btn"
                  @click="handleViewDiff(file)"
                >
                  查看
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="modal-footer">
        <button class="btn btn-primary" @click="$emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Download } from 'lucide-vue-next'
import type { UpdateResult, UpdateFileItem } from '../types/svn'

const props = defineProps<{
  visible: boolean
  result: UpdateResult | null
}>()

const emit = defineEmits<{
  close: []
}>()

const STATUS_ORDER: Record<string, number> = { C: 0, M: 1, A: 2, U: 2 }

const sortedFiles = computed(() => {
  if (!props.result) return []
  return [...props.result.files].sort(
    (a, b) => (STATUS_ORDER[a.status] ?? 9) - (STATUS_ORDER[b.status] ?? 9),
  )
})

const conflictCount = computed(
  () => props.result?.files.filter((f) => f.status === 'C').length ?? 0,
)
const mergedCount = computed(
  () => props.result?.files.filter((f) => f.status === 'M').length ?? 0,
)
const updatedCount = computed(
  () => props.result?.files.filter((f) => f.status === 'A' || f.status === 'U').length ?? 0,
)

function statusClass(status: string) {
  if (status === 'C') return 'conflict'
  if (status === 'M') return 'merged'
  return 'added'
}

function handleResolve(_file: UpdateFileItem) {
  // TODO: open third-party merge tool
  console.log('resolve conflict:', _file.path)
}

function handleViewDiff(_file: UpdateFileItem) {
  // TODO: open DiffViewer (deferred to avoid nested modal complexity)
  console.log('view diff:', _file.path)
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--overlay-bg, rgba(0, 0, 0, 0.5));
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}

.modal {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 12px;
  width: 560px;
  max-height: 460px;
  box-shadow: var(--shadow, 0 8px 32px rgba(0, 0, 0, 0.4));
  display: flex;
  flex-direction: column;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.96); }
  to { opacity: 1; transform: scale(1); }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px 12px;
  border-bottom: 1px solid var(--border);
}

.modal-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--accent, #7c6ff7);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.modal-title {
  font-size: 15px;
  font-weight: 600;
}

.modal-rev {
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--bg-tertiary, #33334d);
  padding: 2px 8px;
  border-radius: 4px;
}

.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-muted, #707090);
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  transition: all 0.15s;
}
.close-btn:hover {
  background: var(--bg-tertiary, #33334d);
  color: var(--text-primary);
}

.stats-bar {
  display: flex;
  gap: 16px;
  padding: 10px 20px;
  background: var(--bg-secondary, #2a2a3d);
  border-bottom: 1px solid var(--border);
  font-size: 13px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.stat-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.stat-dot.conflict { background: var(--red, #f87171); }
.stat-dot.merged { background: var(--yellow, #fbbf24); }
.stat-dot.added { background: var(--green, #4ade80); }

.stat-label { color: var(--text-secondary); }
.stat-value { font-weight: 600; }
.conflict-value { color: var(--red, #f87171); }
.merged-value { color: var(--yellow, #fbbf24); }
.added-value { color: var(--green, #4ade80); }

.file-table-wrapper {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.file-table-wrapper::-webkit-scrollbar { width: 6px; }
.file-table-wrapper::-webkit-scrollbar-track { background: transparent; }
.file-table-wrapper::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}

.file-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.file-table th {
  position: sticky;
  top: 0;
  background: var(--bg-secondary, #2a2a3d);
  text-align: left;
  padding: 8px 12px;
  font-weight: 500;
  color: var(--text-muted, #707090);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border);
  z-index: 1;
}

.file-table td {
  padding: 9px 12px;
  border-bottom: 1px solid rgba(61, 61, 92, 0.4);
  vertical-align: middle;
}

.file-table tr:hover { background: var(--bg-secondary, #2a2a3d); }

.file-table tr.row-conflict {
  background: rgba(248, 113, 113, 0.08);
}
.file-table tr.row-conflict:hover {
  background: rgba(248, 113, 113, 0.14);
}
.file-table tr.row-conflict td:first-child {
  box-shadow: inset 3px 0 0 var(--red, #f87171);
}

.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 5px;
  font-size: 11px;
  font-weight: 700;
}
.status-badge.conflict {
  background: rgba(248, 113, 113, 0.15);
  color: var(--red, #f87171);
}
.status-badge.merged {
  background: rgba(251, 191, 36, 0.15);
  color: var(--yellow, #fbbf24);
}
.status-badge.added {
  background: rgba(74, 222, 128, 0.15);
  color: var(--green, #4ade80);
}

.file-path {
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', monospace;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
}

.file-author {
  color: var(--text-secondary);
  white-space: nowrap;
}

.diff-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 10px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-secondary);
  border-radius: 5px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}
.diff-btn:hover {
  background: var(--accent, #7c6ff7);
  border-color: var(--accent, #7c6ff7);
  color: white;
}

.resolve-btn {
  border-color: rgba(248, 113, 113, 0.4);
  color: var(--red, #f87171);
}
.resolve-btn:hover {
  background: var(--red, #f87171);
  border-color: var(--red, #f87171);
  color: white;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
  gap: 10px;
}

.btn {
  padding: 7px 18px;
  border-radius: 7px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border);
  transition: all 0.15s;
}

.btn-primary {
  background: var(--accent, #7c6ff7);
  border-color: var(--accent, #7c6ff7);
  color: white;
}
.btn-primary:hover {
  background: var(--accent-hover, #9b90f9);
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/PullResultModal.vue
git commit -m "feat(ui): add PullResultModal component"
```

---

### Task 5: Wire modal into `App.vue`

**Files:**
- Modify: `src/App.vue:74-107, 264-269`

- [ ] **Step 1: Add imports and state**

In `src/App.vue`, add to the `<script setup>` imports section (after line 92):

```typescript
import PullResultModal from './components/PullResultModal.vue'
import { useToastStore } from './stores/toastStore'
import type { UpdateResult } from './types/svn'
```

Add state variables (after line 107, near the other `show*` refs):

```typescript
const showPullResult = ref(false)
const pullResult = ref<UpdateResult | null>(null)
```

- [ ] **Step 2: Add modal to template**

In `src/App.vue` template, add before `<Toast />` (before line 70):

```html
    <PullResultModal
      :visible="showPullResult"
      :result="pullResult"
      @close="showPullResult = false"
    />
```

- [ ] **Step 3: Rewrite `handlePull` function**

Replace the existing `handlePull` function (lines 264-269) with:

```typescript
async function handlePull() {
  if (!currentTabStore.value) return
  try {
    const result = await invoke<UpdateResult>('svn_update', {
      path: currentTabStore.value.repoPath,
    })
    if (result.files.length === 0) {
      useToastStore().info('已是最新版本')
    } else {
      pullResult.value = result
      showPullResult.value = true
    }
    refreshCurrentView()
  } catch (e) {
    console.error('Pull failed:', e)
    useToastStore().error('拉取失败')
  }
}
```

- [ ] **Step 4: Verify it compiles**

Run: `npm run build`
Expected: No TypeScript errors.

- [ ] **Step 5: Commit**

```bash
git add src/App.vue
git commit -m "feat: wire PullResultModal into handlePull flow"
```

---

### Task 6: Smoke test end-to-end

- [ ] **Step 1: Run dev server and test manually**

Run: `npm run tauri dev`

Test scenarios:
1. Click pull button with no remote changes → Toast "已是最新版本" appears
2. Click pull button with remote changes → Modal appears with file list
3. Verify conflict rows have red left border and "解决" button
4. Verify merged rows have yellow badge and "查看" button
5. Verify updated rows have green badge and "查看" button
6. Click "关闭" → modal closes
7. Click overlay background → modal closes

- [ ] **Step 2: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass including the new update tests.

- [ ] **Step 3: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: address smoke test findings"
```
