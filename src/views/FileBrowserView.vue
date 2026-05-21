<template>
  <div class="file-browser-view">
    <div class="browser-content">
      <div class="tree-panel">
        <div v-if="props.loading" class="tree-loading">
          <div class="spinner" />
        </div>
        <template v-else>
          <div
            v-for="item in treeItems"
            :key="item.relativePath"
            class="tree-item"
            :class="{ selected: selectedFilePath === item.relativePath }"
            :style="{ paddingLeft: item.depth * 16 + 10 + 'px' }"
            @click="onEntryClick(item.entry, item.relativePath)"
            @contextmenu.prevent="openContextMenu($event, item.entry, item.relativePath)"
          >
            <span
              v-if="item.entry.kind === 'dir'"
              class="tree-arrow"
              :class="{ expanded: expandedKeys[item.relativePath] }"
            >
              <ChevronRight :size="12" />
            </span>
            <span v-else class="tree-arrow-placeholder"></span>

            <span class="entry-icon">
              <FolderIcon v-if="item.entry.kind === 'dir'" :size="14" />
              <FileIcon v-else :size="14" />
            </span>

            <span class="entry-name">{{ item.entry.name }}</span>

            <span v-if="item.entry.kind === 'file' && item.entry.size !== undefined" class="entry-size">
              {{ formatSize(item.entry.size) }}
            </span>

            <span v-if="item.entry.kind === 'dir' && dirLoading[item.relativePath]" class="dir-loading">
              <Loader2 :size="12" class="spin" />
            </span>
          </div>
        </template>
        <div v-if="!props.loading && treeItems.length === 0" class="empty-tree">
          <FolderIcon :size="24" />
          <span>{{ t('common.emptyDir') }}</span>
        </div>
      </div>
      <div class="content-panel">
        <div v-if="selectedFilePath" class="content-header">
          <span class="content-filename">{{ selectedFilePath }}</span>
          <div class="content-actions">
            <button @click="$emit('viewHistory', fullPath)" class="action-btn">
              <History :size="14" />
              <span>{{ t('common.history') }}</span>
            </button>
            <button @click="$emit('aiReview', fullPath)" class="action-btn ai-btn">
              <Sparkles :size="14" />
              <span>{{ t('common.aiReview') }}</span>
            </button>
          </div>
        </div>
        <pre v-if="fileContent" class="file-content">{{ fileContent }}</pre>
        <div v-else class="content-placeholder">
          <FileText :size="32" />
          <span>{{ t('common.clickToViewContent') }}</span>
        </div>
      </div>
    </div>
    <ContextMenu
      :visible="ctxMenu.visible"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      :items="ctxMenuItems"
      @close="ctxMenu.visible = false"
    />
    <FileLogModal
      :visible="showFileLog"
      :file-path="fileLogPath"
      :repo-path="props.repoPath"
      @close="showFileLog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ExternalLink, FolderOpen, Copy, History, Terminal, ChevronRight, Folder as FolderIcon, File as FileIcon, Loader2, Sparkles, FileText, ScrollText } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import type { DirEntry } from '../types/svn'
import type { MenuItem } from '../components/ContextMenu.vue'
import ContextMenu from '../components/ContextMenu.vue'
import FileLogModal from '../components/FileLogModal.vue'
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

interface TreeItem {
  entry: DirEntry
  depth: number
  relativePath: string
}

const fileContent = ref('')
const selectedFilePath = ref('')
const showFileLog = ref(false)
const fileLogPath = ref('')

const expandedDirs = ref<Record<string, DirEntry[]>>({})
const expandedKeys = ref<Record<string, boolean>>({})
const dirLoading = ref<Record<string, boolean>>({})

const fullPath = computed(() => {
  if (!selectedFilePath.value) return ''
  return `${props.repoPath}/${selectedFilePath.value}`
})

function appendChildren(
  result: TreeItem[],
  entries: DirEntry[],
  depth: number,
  parentPath: string,
) {
  for (const entry of entries) {
    const relPath = `${parentPath}/${entry.name}`
    result.push({ entry, depth, relativePath: relPath })
    if (entry.kind === 'dir' && expandedKeys.value[relPath]) {
      const children = expandedDirs.value[relPath] ?? []
      appendChildren(result, children, depth + 1, relPath)
    }
  }
}

const treeItems = computed<TreeItem[]>(() => {
  const result: TreeItem[] = []
  for (const entry of props.fileTree) {
    result.push({ entry, depth: 0, relativePath: entry.name })
    if (entry.kind === 'dir' && expandedKeys.value[entry.name]) {
      const children = expandedDirs.value[entry.name] ?? []
      appendChildren(result, children, 1, entry.name)
    }
  }
  return result
})

async function toggleDir(relativePath: string) {
  if (expandedKeys.value[relativePath]) {
    const { [relativePath]: _, ...rest } = expandedKeys.value
    expandedKeys.value = rest
    return
  }

  expandedKeys.value = { ...expandedKeys.value, [relativePath]: true }

  if (expandedDirs.value[relativePath]) return

  dirLoading.value = { ...dirLoading.value, [relativePath]: true }
  try {
    const path = `${props.repoPath}/${relativePath}`
    const children = await invoke<DirEntry[]>('svn_list', {
      path,
      recursive: false,
    })
    expandedDirs.value = { ...expandedDirs.value, [relativePath]: children }
  } catch (e) {
    useToastStore().error(String(e))
    const { [relativePath]: _, ...rest } = expandedKeys.value
    expandedKeys.value = rest
  } finally {
    dirLoading.value = { ...dirLoading.value, [relativePath]: false }
  }
}

async function onEntryClick(entry: DirEntry, relativePath: string) {
  if (entry.kind === 'dir') {
    toggleDir(relativePath)
  } else {
    selectedFilePath.value = relativePath
    try {
      fileContent.value = await invoke<string>('svn_cat', {
        path: `${props.repoPath}/${relativePath}`,
      })
    } catch (e) {
      fileContent.value = t('common.error') + ': ' + e
    }
  }
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const ctxMenu = ref({ visible: false, x: 0, y: 0, entry: null as DirEntry | null, relativePath: '' })

function openContextMenu(e: MouseEvent, entry: DirEntry, relativePath: string) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, entry, relativePath }
}

const ctxMenuItems = computed<MenuItem[]>(() => {
  const entry = ctxMenu.value.entry
  if (!entry) return []
  const isFile = entry.kind === 'file'
  const relPath = ctxMenu.value.relativePath
  const fullP = `${props.repoPath}/${relPath}`
  const toast = useToastStore()

  const items: MenuItem[] = []

  if (isFile) {
    items.push({
      label: t('contextMenu.openWithEditor'),
      icon: ExternalLink,
      action: async () => {
        try { await invoke('open_file_with_default_app', { path: fullP }) } catch (e) { toast.error(String(e)) }
      },
    })
  }
  items.push({
    label: t('contextMenu.showInExplorer'),
    icon: FolderOpen,
    action: async () => {
      try { await invoke('open_in_system', { path: fullP }) } catch (e) { toast.error(String(e)) }
    },
  })
  items.push({ divider: true })
  items.push({
    label: t('contextMenu.showLog'),
    icon: ScrollText,
    action: () => {
      fileLogPath.value = fullP
      showFileLog.value = true
    },
  })
  if (isFile) {
    items.push({
      label: t('contextMenu.showBlame'),
      icon: Terminal,
      action: async () => {
        try {
          const entries = await invoke<{ revision: number; author: string; lineNumber: number }[]>('svn_blame', {
            path: fullP,
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
          await invoke('svn_cleanup', { path: fullP })
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
      navigator.clipboard.writeText(relPath)
      toast.success(t('contextMenu.copySuccess'))
    },
  })
  items.push({
    label: t('contextMenu.copyAbsPath'),
    icon: Copy,
    action: () => {
      navigator.clipboard.writeText(fullP)
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

.browser-content {
  display: flex;
  flex: 1;
  gap: var(--space-3);
  min-height: 0;
}

.tree-panel {
  width: 280px;
  min-width: 200px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  overflow: auto;
  background: var(--color-bg-primary);
}

.tree-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-6) 0;
  color: var(--color-text-muted);
}

.tree-item {
  display: flex;
  align-items: center;
  padding: var(--space-2) var(--space-3);
  gap: var(--space-2);
  cursor: pointer;
  font-size: var(--text-base);
  color: var(--color-text-primary);
  white-space: nowrap;
  transition: background var(--transition-fast);
}

.tree-item:hover {
  background: var(--color-bg-hover);
}

.tree-item.selected {
  background: var(--color-bg-active);
}

.tree-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  transition: transform var(--transition-fast);
  color: var(--color-text-muted);
}

.tree-arrow.expanded {
  transform: rotate(90deg);
}

.tree-arrow-placeholder {
  width: 16px;
  flex-shrink: 0;
}

.entry-icon {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.entry-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: var(--text-sm);
}

.entry-size {
  font-size: var(--text-xs);
  color: var(--color-text-muted);
  flex-shrink: 0;
  font-family: var(--font-mono);
}

.dir-loading {
  display: inline-flex;
  align-items: center;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-tree {
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--space-8) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

.content-panel {
  flex: 1;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  overflow: auto;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-primary);
}

.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
}

.content-filename {
  font-size: var(--text-base);
  font-weight: 500;
  font-family: var(--font-mono);
  color: var(--color-text-primary);
}

.content-actions {
  display: flex;
  gap: var(--space-2);
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-1);
  height: 28px;
  padding: 0 var(--space-3);
  border: 1px solid var(--color-border-input);
  background: var(--color-bg-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-sm);
  color: var(--color-text-primary);
  transition: all var(--transition-fast);
}

.action-btn:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.action-btn.ai-btn {
  color: var(--color-purple);
  border-color: var(--color-purple-muted);
}

.action-btn.ai-btn:hover {
  background: var(--color-purple-muted);
  border-color: var(--color-purple);
}

.file-content {
  font-family: var(--font-code);
  font-size: var(--text-base);
  white-space: pre-wrap;
  word-break: break-all;
  padding: var(--space-4);
  margin: 0;
  line-height: 1.6;
  color: var(--color-text-primary);
}

.content-placeholder {
  color: var(--color-text-muted);
  text-align: center;
  margin-top: var(--space-10);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}
</style>
