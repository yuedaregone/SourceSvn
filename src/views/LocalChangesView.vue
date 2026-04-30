<template>
  <div class="local-changes-view">
    <div class="left-panel">
      <div class="file-list-header">
        <label>
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          全选
        </label>
        <span class="selected-count">已选 {{ selectedFiles.length }} 个文件</span>
        <button @click="$emit('refresh')" class="refresh-btn">刷新</button>
      </div>
      <div class="file-list">
        <div
          v-for="file in store.localChanges"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedPaths.has(file.path) }"
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
      </div>
      <div class="commit-section">
        <textarea
          v-model="commitMessage"
          placeholder="提交信息..."
          rows="3"
          class="commit-input"
        ></textarea>
        <div class="commit-actions">
          <button @click="generateAiMessage" :disabled="aiLoading" class="ai-btn">
            {{ aiLoading ? '生成中...' : 'AI 生成注释' }}
          </button>
          <button @click="cancelCommit" class="cancel-btn">取消</button>
          <button @click="submitCommit" :disabled="!canCommit" class="commit-btn">提交</button>
        </div>
      </div>
    </div>
    <div class="right-panel">
      <div v-if="diffContent" class="diff-content">
        <pre>{{ diffContent }}</pre>
      </div>
      <div v-else class="diff-placeholder">点击文件查看差异</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FileStatus } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    localChanges: FileStatus[]
    refreshLocalChanges: () => Promise<void>
  }
}>()

defineEmits<{
  refresh: []
}>()

const selectedPaths = ref(new Set<string>())
const commitMessage = ref('')
const diffContent = ref('')
const aiLoading = ref(false)

const selectedFiles = computed(() =>
  props.store.localChanges.filter((f) => selectedPaths.value.has(f.path)),
)

const allSelected = computed(
  () =>
    props.store.localChanges.length > 0 &&
    props.store.localChanges.every((f) => selectedPaths.value.has(f.path)),
)

const canCommit = computed(
  () => selectedPaths.value.size > 0 && commitMessage.value.trim().length > 0,
)

function toggleAll() {
  if (allSelected.value) {
    selectedPaths.value.clear()
  } else {
    props.store.localChanges.forEach((f) => selectedPaths.value.add(f.path))
  }
}

function toggleFile(path: string) {
  if (selectedPaths.value.has(path)) {
    selectedPaths.value.delete(path)
  } else {
    selectedPaths.value.add(path)
  }
}

async function selectFile(file: FileStatus) {
  try {
    diffContent.value = await invoke<string>('svn_diff', {
      path: props.store.repoPath,
      target: { type: 'File', data: { path: file.path } },
    })
  } catch (e) {
    diffContent.value = `获取差异失败: ${e}`
  }
}

async function generateAiMessage() {
  if (selectedPaths.value.size === 0) return
  aiLoading.value = true
  try {
    const diff = await invoke<string>('svn_diff', {
      path: props.store.repoPath,
      target: { type: 'File', data: { path: Array.from(selectedPaths.value)[0] } },
    })
    commitMessage.value = await invoke<string>('generate_commit_message', { diff })
  } catch (e) {
    console.error('AI generation failed:', e)
  } finally {
    aiLoading.value = false
  }
}

async function submitCommit() {
  if (!canCommit.value) return
  try {
    await invoke('svn_commit', {
      path: props.store.repoPath,
      message: commitMessage.value,
      files: Array.from(selectedPaths.value),
    })
    commitMessage.value = ''
    selectedPaths.value.clear()
    await props.store.refreshLocalChanges()
  } catch (e) {
    console.error('Commit failed:', e)
  }
}

function cancelCommit() {
  commitMessage.value = ''
  selectedPaths.value.clear()
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
}
.right-panel {
  flex: 1;
  border-left: 1px solid #e8e8e8;
  padding-left: 12px;
}
.file-list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 13px;
}
.file-list {
  flex: 1;
  overflow: auto;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
}
.file-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid #f0f0f0;
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
}
.status-badge.modified { background: #faad14; }
.status-badge.added { background: #52c41a; }
.status-badge.deleted { background: #ff4d4f; }
.status-badge.unversioned { background: #999; }
.status-badge.missing { background: #ff7a45; }
.status-badge.conflicted { background: #f5222d; }
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
.diff-content pre {
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
}
.diff-placeholder {
  color: #999;
  text-align: center;
  margin-top: 40px;
}
</style>
