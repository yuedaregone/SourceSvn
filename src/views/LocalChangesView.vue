<template>
  <div class="local-changes-view">
    <div class="left-panel">
      <div class="file-list-header">
        <label class="select-all">
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          <span>全选</span>
        </label>
        <span class="selected-count">已选 {{ selectedPaths.size }} 个文件</span>
        <div class="header-actions">
          <button @click="$emit('pull')" class="action-btn">拉取</button>
          <button @click="$emit('commit')" class="action-btn primary" :disabled="!canCommit">提交</button>
          <button @click="$emit('refresh')" class="action-btn">刷新</button>
        </div>
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

defineEmits<{
  refresh: []
  pull: []
  commit: []
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
    const target: DiffTarget = { type: 'File', data: { path: file.path } }
    diffContent.value = await invoke<string>('svn_diff', {
      path: props.store.repoPath,
      target,
    })
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
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  overflow: auto;
  min-width: 0;
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
  color: #666;
}
.header-actions {
  margin-left: auto;
  display: flex;
  gap: 6px;
}
.action-btn {
  padding: 4px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.action-btn:hover:not(:disabled) {
  border-color: #1890ff;
  color: #1890ff;
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn.primary {
  background: #1890ff;
  color: #fff;
  border-color: #1890ff;
}
.action-btn.primary:hover:not(:disabled) {
  background: #40a9ff;
}
.file-list {
  flex: 1;
  overflow: auto;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  min-height: 0;
}
.file-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid #f5f5f5;
}
.file-item:hover {
  background: #f5f5f5;
}
.file-item.selected {
  background: #e6f7ff;
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
.status-badge.modified { background: #faad14; }
.status-badge.added { background: #52c41a; }
.status-badge.deleted { background: #ff4d4f; }
.status-badge.unversioned { background: #999; }
.status-badge.missing { background: #ff7a45; }
.status-badge.conflicted { background: #f5222d; }
.file-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
  font-size: 12px;
}
.empty-list {
  color: #999;
  text-align: center;
  padding: 24px 0;
}
.commit-section {
  margin-top: 12px;
}
.commit-input {
  width: 100%;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  resize: vertical;
  font-family: inherit;
  box-sizing: border-box;
}
.commit-input:focus {
  border-color: #1890ff;
  outline: none;
}
.commit-stats {
  margin-top: 6px;
  font-size: 12px;
  display: flex;
  gap: 8px;
}
.stat-add {
  color: #52c41a;
  font-weight: 500;
}
.stat-del {
  color: #ff4d4f;
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
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.commit-btn {
  background: #1890ff !important;
  color: #fff !important;
  border-color: #1890ff !important;
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
  background: #fff2f0;
  border: 1px solid #ffccc7;
  border-radius: 4px;
  color: #ff4d4f;
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
  background: #e6ffed;
  color: #22863a;
}
.diff-del {
  background: #ffeef0;
  color: #cb2431;
}
.diff-hunk {
  background: #f0f0ff;
  color: #666;
}
.diff-placeholder {
  color: #999;
  text-align: center;
  margin-top: 40px;
  font-size: 13px;
}
</style>
