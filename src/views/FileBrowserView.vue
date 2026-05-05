<template>
  <div class="file-browser-view">
    <div class="browser-content">
      <div class="tree-panel">
        <div v-if="props.loading" class="tree-loading">
          <RefreshCw :size="16" class="spin" />
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
        <div v-if="!props.loading && treeItems.length === 0" class="empty-tree">{{ t('common.emptyDir') }}</div>
      </div>
      <div class="content-panel">
        <div v-if="selectedFilePath" class="content-header">
          <span class="content-filename">{{ selectedFilePath }}</span>
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
import { RefreshCw, ExternalLink, FolderOpen, Copy, History, Terminal, ChevronRight, Folder as FolderIcon, File as FileIcon, Loader2 } from 'lucide-vue-next'
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

interface TreeItem {
  entry: DirEntry
  depth: number
  relativePath: string
}

const fileContent = ref('')
const selectedFilePath = ref('')

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
    icon: History,
    action: () => { emit('viewHistory', fullP) },
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
.tree-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 0;
  color: var(--text-muted);
}
.tree-item {
  display: flex;
  align-items: center;
  padding: 4px 10px;
  gap: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  white-space: nowrap;
}
.tree-item:hover {
  background: var(--bg-hover);
}
.tree-item.selected {
  background: var(--bg-active);
}
.tree-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  transition: transform 0.15s ease;
  color: var(--text-muted);
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
  color: var(--text-secondary);
}
.entry-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}
.entry-size {
  font-size: 11px;
  color: var(--text-muted);
  flex-shrink: 0;
}
.dir-loading {
  display: inline-flex;
  align-items: center;
  color: var(--text-muted);
  flex-shrink: 0;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.spin {
  animation: spin 1s linear infinite;
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
