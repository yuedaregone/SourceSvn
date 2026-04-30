<template>
  <div class="file-browser-view">
    <div class="browser-header">
      <button v-if="currentPath" @click="goBack" class="back-btn">返回上级</button>
      <span v-if="currentPath" class="current-path">{{ currentPath }}</span>
      <button @click="refresh" class="refresh-btn">刷新</button>
    </div>
    <div class="browser-content">
      <div class="tree-panel">
        <div
          v-for="entry in store.fileTree"
          :key="entry.name"
          class="tree-item"
          @click="onEntryClick(entry)"
        >
          <span class="entry-icon">{{ entry.kind === 'dir' ? '📁' : '📄' }}</span>
          <span class="entry-name">{{ entry.name }}</span>
        </div>
      </div>
      <div class="content-panel">
        <pre v-if="fileContent" class="file-content">{{ fileContent }}</pre>
        <div v-else class="content-placeholder">点击文件查看内容</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DirEntry } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    fileTree: DirEntry[]
    refreshFileBrowser: (path?: string) => Promise<void>
  }
}>()

const fileContent = ref('')
const currentPath = ref('')

async function onEntryClick(entry: DirEntry) {
  if (entry.kind === 'dir') {
    const dirPath = currentPath.value
      ? `${currentPath.value}/${entry.name}`
      : entry.name
    currentPath.value = dirPath
    await props.store.refreshFileBrowser(`${props.store.repoPath}/${dirPath}`)
  } else {
    const filePath = currentPath.value
      ? `${currentPath.value}/${entry.name}`
      : entry.name
    try {
      fileContent.value = await invoke<string>('svn_cat', {
        path: `${props.store.repoPath}/${filePath}`,
      })
    } catch (e) {
      fileContent.value = `读取失败: ${e}`
    }
  }
}

function goBack() {
  fileContent.value = ''
  const parts = currentPath.value.split('/')
  parts.pop()
  currentPath.value = parts.join('/')
  if (currentPath.value) {
    props.store.refreshFileBrowser(`${props.store.repoPath}/${currentPath.value}`)
  } else {
    props.store.refreshFileBrowser()
  }
}

function refresh() {
  fileContent.value = ''
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
  gap: 8px;
  margin-bottom: 8px;
}
.back-btn {
  padding: 4px 8px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.current-path {
  flex: 1;
  font-size: 12px;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.browser-content {
  display: flex;
  flex: 1;
  gap: 12px;
}
.tree-panel {
  width: 250px;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  overflow: auto;
}
.tree-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
}
.tree-item:hover {
  background: #f5f5f5;
}
.content-panel {
  flex: 1;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  overflow: auto;
  padding: 12px;
}
.file-content {
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
}
.content-placeholder {
  color: #999;
  text-align: center;
  margin-top: 40px;
}
</style>
