<template>
  <div class="file-browser-view">
    <div class="browser-header">
      <button v-if="currentPath" @click="goBack" class="back-btn">返回上级</button>
      <select v-model="selectedRevision" class="revision-select" @change="onRevisionChange">
        <option value="HEAD">HEAD</option>
      </select>
      <label class="checkbox-label">
        <input type="checkbox" v-model="showHidden" @change="refresh" />
        显示隐藏文件
      </label>
      <button @click="refresh" class="refresh-btn" :disabled="store.loading">刷新</button>
    </div>
    <div class="browser-content">
      <div class="tree-panel">
        <div
          v-for="entry in displayedEntries"
          :key="entry.name"
          class="tree-item"
          :class="{ selected: selectedFile === entry.name }"
          @click="onEntryClick(entry)"
        >
          <span class="entry-icon">{{ entry.kind === 'dir' ? '📁' : '📄' }}</span>
          <span class="entry-name">{{ entry.name }}</span>
          <span v-if="entry.size !== undefined" class="entry-size">{{ formatSize(entry.size) }}</span>
        </div>
        <div v-if="displayedEntries.length === 0" class="empty-tree">空目录</div>
      </div>
      <div class="content-panel">
        <div v-if="selectedFile" class="content-header">
          <span class="content-filename">{{ selectedFile }}</span>
          <div class="content-actions">
            <button @click="$emit('viewHistory', fullPath)" class="action-btn">历史</button>
            <button @click="$emit('aiReview', fullPath)" class="action-btn ai">AI 审查</button>
          </div>
        </div>
        <pre v-if="fileContent" class="file-content">{{ fileContent }}</pre>
        <div v-else class="content-placeholder">点击文件查看内容</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DirEntry } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    fileTree: DirEntry[]
    loading: boolean
    refreshFileBrowser: (path?: string) => Promise<void>
  }
}>()

defineEmits<{
  viewHistory: [path: string]
  aiReview: [path: string]
}>()

const fileContent = ref('')
const currentPath = ref('')
const selectedFile = ref('')
const selectedRevision = ref('HEAD')
const showHidden = ref(false)

const fullPath = computed(() => {
  if (!selectedFile.value) return ''
  return currentPath.value
    ? `${props.store.repoPath}/${currentPath.value}/${selectedFile.value}`
    : `${props.store.repoPath}/${selectedFile.value}`
})

const displayedEntries = computed(() => {
  if (showHidden.value) return props.store.fileTree
  return props.store.fileTree.filter((e) => !e.name.startsWith('.'))
})

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

async function onEntryClick(entry: DirEntry) {
  if (entry.kind === 'dir') {
    const dirPath = currentPath.value
      ? `${currentPath.value}/${entry.name}`
      : entry.name
    currentPath.value = dirPath
    selectedFile.value = ''
    fileContent.value = ''
    await props.store.refreshFileBrowser(`${props.store.repoPath}/${dirPath}`)
  } else {
    selectedFile.value = entry.name
    const filePath = currentPath.value
      ? `${currentPath.value}/${entry.name}`
      : entry.name
    try {
      const params: Record<string, unknown> = {
        path: `${props.store.repoPath}/${filePath}`,
      }
      if (selectedRevision.value !== 'HEAD') {
        params.revision = selectedRevision.value
      }
      fileContent.value = await invoke<string>('svn_cat', params)
    } catch (e) {
      fileContent.value = `读取失败: ${e}`
    }
  }
}

function goBack() {
  fileContent.value = ''
  selectedFile.value = ''
  const parts = currentPath.value.split('/')
  parts.pop()
  currentPath.value = parts.join('/')
  if (currentPath.value) {
    props.store.refreshFileBrowser(`${props.store.repoPath}/${currentPath.value}`)
  } else {
    props.store.refreshFileBrowser()
  }
}

function onRevisionChange() {
  fileContent.value = ''
  selectedFile.value = ''
  if (currentPath.value) {
    props.store.refreshFileBrowser(`${props.store.repoPath}/${currentPath.value}`)
  } else {
    props.store.refreshFileBrowser()
  }
}

function refresh() {
  fileContent.value = ''
  selectedFile.value = ''
  currentPath.value = ''
  props.store.refreshFileBrowser()
}

onMounted(() => {
  props.store.refreshFileBrowser()
})
</script>

<style scoped>
.file-browser-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.browser-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}
.revision-select {
  padding: 5px 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  background: #fff;
  min-width: 100px;
}
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  cursor: pointer;
}
.refresh-btn {
  margin-left: auto;
  padding: 5px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.browser-content {
  display: flex;
  flex: 1;
  gap: 12px;
  min-height: 0;
}
.tree-panel {
  width: 280px;
  min-width: 200px;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  overflow: auto;
}
.tree-item {
  display: flex;
  align-items: center;
  padding: 6px 10px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid #f5f5f5;
}
.tree-item:hover {
  background: #f5f5f5;
}
.tree-item.selected {
  background: #e6f7ff;
}
.entry-icon {
  font-size: 14px;
  flex-shrink: 0;
}
.entry-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.entry-size {
  font-size: 11px;
  color: #999;
  flex-shrink: 0;
}
.empty-tree {
  color: #999;
  text-align: center;
  padding: 24px 0;
}
.content-panel {
  flex: 1;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  overflow: auto;
  display: flex;
  flex-direction: column;
}
.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid #e8e8e8;
  background: #fafafa;
}
.content-filename {
  font-size: 13px;
  font-weight: 500;
  font-family: monospace;
}
.content-actions {
  display: flex;
  gap: 6px;
}
.action-btn {
  padding: 3px 10px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.action-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
}
.action-btn.ai {
  border-color: #722ed1;
  color: #722ed1;
}
.action-btn.ai:hover {
  background: #f9f0ff;
}
.file-content {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
  padding: 12px;
  margin: 0;
  line-height: 1.5;
}
.content-placeholder {
  color: #999;
  text-align: center;
  margin-top: 40px;
}
</style>
