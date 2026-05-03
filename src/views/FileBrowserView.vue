<template>
  <div class="file-browser-view">
    <div class="browser-header">
      <button v-if="currentPath" @click="goBack" class="back-btn icon-btn" :title="t('common.back')">
        <ArrowLeft :size="16" />
      </button>
      <select v-model="selectedRevision" class="revision-select" @change="onRevisionChange">
        <option value="HEAD">{{ t('fileBrowser.head') }}</option>
      </select>
      <label class="checkbox-label">
        <input type="checkbox" v-model="showHidden" @change="refresh" />
        {{ t('fileBrowser.showHidden') }}
      </label>
      <button @click="refresh" class="refresh-btn icon-btn" :disabled="props.loading" :title="t('common.refresh')">
        <RefreshCw :size="16" />
      </button>
    </div>
    <div class="browser-content">
      <div class="tree-panel">
        <div
          v-for="entry in displayedEntries"
          :key="entry.name"
          class="tree-item"
          :class="{ selected: selectedFile === entry.name }"
          @click="onEntryClick(entry)"
          @contextmenu.prevent="openContextMenu($event, entry)"
        >
          <span class="entry-icon">{{ entry.kind === 'dir' ? '📁' : '📄' }}</span>
          <span class="entry-name">{{ entry.name }}</span>
          <span v-if="entry.size !== undefined" class="entry-size">{{ formatSize(entry.size) }}</span>
        </div>
        <div v-if="displayedEntries.length === 0" class="empty-tree">{{ t('common.emptyDir') }}</div>
      </div>
      <div class="content-panel">
        <div v-if="selectedFile" class="content-header">
          <span class="content-filename">{{ selectedFile }}</span>
          <div class="content-actions">
            <button @click="$emit('viewHistory', fullPath)" class="action-btn">{{ t('common.history') }}</button>
            <button @click="$emit('aiReview', fullPath)" class="action-btn ai">{{ t('common.aiReview') }}</button>
          </div>
        </div>
        <pre v-if="fileContent" class="file-content">{{ fileContent }}</pre>
        <div v-else class="content-placeholder">{{ t('common.clickToViewContent') }}</div>
      </div>
    </div>
    <ContextMenu
      :visible="ctxMenu.visible"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      :items="ctxMenuItems"
      @close="ctxMenu.visible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ArrowLeft, RefreshCw, ExternalLink, FolderOpen, Copy, History, Terminal } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import type { DirEntry } from '../types/svn'
import type { MenuItem } from '../components/ContextMenu.vue'
import ContextMenu from '../components/ContextMenu.vue'
import { t } from '../locales'

const props = defineProps<{
  repoPath: string
  fileTree: DirEntry[]
  loading: boolean
}>()

const emit = defineEmits<{
  viewHistory: [path: string]
  aiReview: [path: string]
  refreshFileBrowser: [path?: string]
}>()

const fileContent = ref('')
const currentPath = ref('')
const selectedFile = ref('')
const selectedRevision = ref('HEAD')
const showHidden = ref(false)

const fullPath = computed(() => {
  if (!selectedFile.value) return ''
  return currentPath.value
    ? `${props.repoPath}/${currentPath.value}/${selectedFile.value}`
    : `${props.repoPath}/${selectedFile.value}`
})

const displayedEntries = computed(() => {
  if (showHidden.value) return props.fileTree
  return props.fileTree.filter((e) => !e.name.startsWith('.'))
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
    emit('refreshFileBrowser', `${props.repoPath}/${dirPath}`)
  } else {
    selectedFile.value = entry.name
    const filePath = currentPath.value
      ? `${currentPath.value}/${entry.name}`
      : entry.name
    try {
      const params: Record<string, unknown> = {
        path: `${props.repoPath}/${filePath}`,
      }
      if (selectedRevision.value !== 'HEAD') {
        params.revision = selectedRevision.value
      }
      fileContent.value = await invoke<string>('svn_cat', params)
    } catch (e) {
      fileContent.value = t('common.error') + ': ' + e
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
    emit('refreshFileBrowser', `${props.repoPath}/${currentPath.value}`)
  } else {
    emit('refreshFileBrowser')
  }
}

function onRevisionChange() {
  fileContent.value = ''
  selectedFile.value = ''
  if (currentPath.value) {
    emit('refreshFileBrowser', `${props.repoPath}/${currentPath.value}`)
  } else {
    emit('refreshFileBrowser')
  }
}

function refresh() {
  fileContent.value = ''
  selectedFile.value = ''
  currentPath.value = ''
  emit('refreshFileBrowser')
}

const ctxMenu = ref({ visible: false, x: 0, y: 0, entry: null as DirEntry | null })

function openContextMenu(e: MouseEvent, entry: DirEntry) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, entry }
}

function getEntryRelativePath(entry: DirEntry) {
  return currentPath.value
    ? `${currentPath.value}/${entry.name}`
    : entry.name
}

function getEntryFullPath(entry: DirEntry) {
  return `${props.repoPath}/${getEntryRelativePath(entry)}`
}

const ctxMenuItems = computed<MenuItem[]>(() => {
  const entry = ctxMenu.value.entry
  if (!entry) return []
  const isFile = entry.kind === 'file'
  const toast = useToastStore()

  const items: MenuItem[] = []

  if (isFile) {
    items.push({
      label: t('contextMenu.openWithEditor'),
      icon: ExternalLink,
      action: async () => {
        try { await invoke('open_in_system', { path: getEntryFullPath(entry) }) } catch (e) { toast.error(String(e)) }
      },
    })
  }
  items.push({
    label: t('contextMenu.showInExplorer'),
    icon: FolderOpen,
    action: async () => {
      try { await open(getEntryFullPath(entry)) } catch (e) { toast.error(String(e)) }
    },
  })
  items.push({ divider: true })
  items.push({
    label: t('contextMenu.showLog'),
    icon: History,
    action: () => { emit('viewHistory', getEntryFullPath(entry)) },
  })
  if (isFile) {
    items.push({
      label: t('contextMenu.showBlame'),
      icon: Terminal,
      action: async () => {
        try {
          const entries = await invoke<{ revision: number; author: string; lineNumber: number }[]>('svn_blame', {
            path: getEntryFullPath(entry),
          })
          const blameText = entries.map(e => `  ${e.revision}  ${e.author.padEnd(12)}  L${e.lineNumber}`).join('\n')
          await navigator.clipboard.writeText(blameText)
          toast.success('Blame result copied to clipboard')
        } catch (e) { toast.error(String(e)) }
      },
    })
  } else {
    items.push({
      label: t('contextMenu.cleanup'),
      action: async () => {
        try {
          await invoke('svn_cleanup', { path: getEntryFullPath(entry) })
          toast.success(t('contextMenu.cleanup'))
        } catch (e) { toast.error(String(e)) }
      },
    })
  }
  items.push({ divider: true })
  items.push({
    label: t('contextMenu.copyPath'),
    icon: Copy,
    action: () => {
      navigator.clipboard.writeText(getEntryRelativePath(entry))
      toast.success(t('contextMenu.copySuccess'))
    },
  })
  items.push({
    label: t('contextMenu.copyAbsPath'),
    icon: Copy,
    action: () => {
      navigator.clipboard.writeText(getEntryFullPath(entry))
      toast.success(t('contextMenu.copySuccess'))
    },
  })

  return items
})

onMounted(() => {
  emit('refreshFileBrowser')
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
  border: 1px solid var(--border-input);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
  min-width: 100px;
}
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  cursor: pointer;
  color: var(--text-primary);
}
.refresh-btn {
  margin-left: auto;
  padding: 5px 12px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
}
.refresh-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.refresh-btn.icon-btn,
.back-btn.icon-btn {
  padding: 5px;
  width: 26px;
  height: 26px;
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
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: auto;
  background: var(--bg-primary);
}
.tree-item {
  display: flex;
  align-items: center;
  padding: 6px 10px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid var(--border-light);
  color: var(--text-primary);
}
.tree-item:hover {
  background: var(--bg-hover);
}
.tree-item.selected {
  background: var(--bg-active);
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
  color: var(--text-muted);
  flex-shrink: 0;
}
.empty-tree {
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}
.content-panel {
  flex: 1;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}
.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}
.content-filename {
  font-size: 13px;
  font-weight: 500;
  font-family: monospace;
  color: var(--text-primary);
}
.content-actions {
  display: flex;
  gap: 6px;
}
.action-btn {
  padding: 3px 10px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
}
.action-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.action-btn.ai {
  border-color: var(--purple-color);
  color: var(--purple-color);
}
.action-btn.ai:hover {
  background: var(--bg-hover);
}
.file-content {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
  padding: 12px;
  margin: 0;
  line-height: 1.5;
  color: var(--text-primary);
}
.content-placeholder {
  color: var(--text-muted);
  text-align: center;
  margin-top: 40px;
}
</style>
