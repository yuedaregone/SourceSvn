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

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FileStatus, DiffTarget } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    localChanges: FileStatus[]
    refreshLocalChanges: () => Promise<void>
  }
}>()

const emit = defineEmits<{
  refresh: []
}>()

const selectedPaths = ref(new Set<string>())
const selectedFile = ref('')
const commitMessage = ref('')
const diffContent = ref('')
const aiLoading = ref(false)
const errorMessage = ref('')

const allSelected = computed(
  () =>
    props.store.localChanges.length > 0 &&
    props.store.localChanges.every((f) => selectedPaths.value.has(f.path)),
)

const canCommit = computed(
  () => selectedPaths.value.size > 0 && commitMessage.value.trim().length > 0,
)

const coloredLines = computed(() => {
  if (!diffContent.value) return []
  return diffContent.value.split('\n')
})

const diffStats = computed(() => {
  if (!diffContent.value) return null
  let added = 0
  let removed = 0
  for (const line of diffContent.value.split('\n')) {
    if (line.startsWith('+') && !line.startsWith('+++')) added++
    else if (line.startsWith('-') && !line.startsWith('---')) removed++
  }
  return { added, removed }
})

function lineClass(line: string) {
  if (line.startsWith('+') && !line.startsWith('+++')) return 'diff-add'
  if (line.startsWith('-') && !line.startsWith('---')) return 'diff-del'
  if (line.startsWith('@@')) return 'diff-hunk'
  return ''
}

function toggleAll() {
  if (allSelected.value) {
    selectedPaths.value = new Set()
  } else {
    selectedPaths.value = new Set(props.store.localChanges.map((f) => f.path))
  }
}

function toggleFile(path: string) {
  const next = new Set(selectedPaths.value)
  if (next.has(path)) {
    next.delete(path)
  } else {
    next.add(path)
  }
  selectedPaths.value = next
}

async function selectFile(file: FileStatus) {
  errorMessage.value = ''
  selectedFile.value = file.path
  selectedPaths.value = new Set([file.path])
  try {
    if (file.status === 'unversioned') {
      diffContent.value = await invoke<string>('diff_unversioned_file', {
        repoPath: props.store.repoPath,
        filePath: file.path,
      })
    } else {
      const target: DiffTarget = { type: 'File', data: { path: file.path } }
      diffContent.value = await invoke<string>('svn_diff', {
        path: props.store.repoPath,
        target,
      })
    }
  } catch (e) {
    diffContent.value = ''
    errorMessage.value = `获取差异失败: ${e}`
  }
}

async function generateAiMessage() {
  if (selectedPaths.value.size === 0) return
  aiLoading.value = true
  errorMessage.value = ''
  try {
    const firstPath = Array.from(selectedPaths.value)[0]
    const target: DiffTarget = { type: 'File', data: { path: firstPath } }
    const diff = await invoke<string>('svn_diff', {
      path: props.store.repoPath,
      target,
    })
    commitMessage.value = await invoke<string>('generate_commit_message', { diff })
  } catch (e) {
    errorMessage.value = `AI 生成失败: ${e}`
  } finally {
    aiLoading.value = false
  }
}

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

function cancelCommit() {
  commitMessage.value = ''
  selectedPaths.value = new Set()
  selectedFile.value = ''
  diffContent.value = ''
}

onMounted(() => {
  props.store.refreshLocalChanges()
})
</script>

<style scoped>
.local-changes-view {
  display: flex;
  height: 100%;
  gap: 12px;
}
.left-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.right-panel {
  flex: 1;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: auto;
  min-width: 0;
  background: var(--bg-primary);
}
.file-list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 13px;
}
.select-all {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}
.selected-count {
  color: var(--text-secondary);
}
.header-actions {
  margin-left: auto;
  display: flex;
  gap: 6px;
}
.action-btn {
  padding: 4px 12px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
}
.action-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn.primary {
  background: var(--accent-color);
  color: #fff;
  border-color: var(--accent-color);
}
.action-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
}
.file-list {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  min-height: 0;
  background: var(--bg-primary);
}
.file-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid var(--border-light);
}
.file-item:hover {
  background: var(--bg-hover);
}
.file-item.selected {
  background: var(--bg-active);
}
.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: bold;
  color: #fff;
  flex-shrink: 0;
}
.status-badge.modified { background: var(--warning-color); }
.status-badge.added { background: var(--success-color); }
.status-badge.deleted { background: var(--danger-color); }
.status-badge.unversioned { background: var(--text-muted); }
.status-badge.missing { background: #ff7a45; }
.status-badge.conflicted { background: #f5222d; }
.file-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
  font-size: 12px;
  color: var(--text-primary);
}
.empty-list {
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}
.commit-section {
  margin-top: 12px;
}
.commit-input {
  width: 100%;
  padding: 8px;
  border: 1px solid var(--border-input);
  border-radius: 4px;
  font-size: 13px;
  resize: vertical;
  font-family: inherit;
  box-sizing: border-box;
  background: var(--bg-primary);
  color: var(--text-primary);
}
.commit-input:focus {
  border-color: var(--accent-color);
  outline: none;
}
.commit-stats {
  margin-top: 6px;
  font-size: 12px;
  display: flex;
  gap: 8px;
}
.stat-add {
  color: var(--success-color);
  font-weight: 500;
}
.stat-del {
  color: var(--danger-color);
  font-weight: 500;
}
.commit-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
  justify-content: flex-end;
}
.commit-actions button {
  padding: 6px 16px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
}
.commit-btn {
  background: var(--accent-color) !important;
  color: #fff !important;
  border-color: var(--accent-color) !important;
}
.commit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.ai-btn {
  margin-right: auto;
}
.error-message {
  margin-top: 8px;
  padding: 6px 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--danger-color);
  border-radius: 4px;
  color: var(--danger-color);
  font-size: 12px;
}
.diff-content {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  white-space: pre;
  padding: 12px;
  margin: 0;
  line-height: 1.6;
}
.diff-add {
  background: var(--diff-add-bg);
  color: var(--diff-add-text);
}
.diff-del {
  background: var(--diff-del-bg);
  color: var(--diff-del-text);
}
.diff-hunk {
  background: var(--diff-hunk-bg);
  color: var(--text-secondary);
}
.diff-placeholder {
  color: var(--text-muted);
  text-align: center;
  margin-top: 40px;
  font-size: 13px;
}
</style>
