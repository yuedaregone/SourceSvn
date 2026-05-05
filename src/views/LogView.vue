<template>
  <div class="log-view">
    <div class="filter-bar">
      <select v-model="authorFilter" class="filter-select">
        <option value="">{{ t('logView.allAuthors') }}</option>
        <option v-for="a in authors" :key="a" :value="a">{{ a }}</option>
      </select>
      <input v-model="dateFrom" type="date" class="filter-date" :placeholder="t('common.startDate')" />
      <span class="date-separator">~</span>
      <input v-model="dateTo" type="date" class="filter-date" :placeholder="t('common.endDate')" />
      <input v-model="searchText" :placeholder="t('common.searchMessage')" class="search-input" />
    </div>
    <div class="log-table">
      <table>
        <thead>
          <tr>
            <th class="col-revision">{{ t('logView.revision') }}</th>
            <th class="col-author">{{ t('logView.author') }}</th>
            <th class="col-date">{{ t('logView.date') }}</th>
            <th class="col-message">{{ t('logView.message') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in pagedEntries"
            :key="entry.revision"
            @click="toggleDetail(entry.revision)"
            @contextmenu.prevent="openContextMenu($event, entry)"
            :class="{ expanded: expandedRevision === entry.revision, 'non-local': !isLocal(entry) }"
          >
            <td class="col-revision">{{ entry.revision }}</td>
            <td class="col-author">{{ entry.author }}</td>
            <td class="col-date">{{ formatDate(entry.date) }}</td>
            <td class="col-message">{{ entry.message }}</td>
          </tr>
          <tr v-if="filteredEntries.length === 0">
            <td colspan="4" class="empty-row">{{ t('logView.noLogs') }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="pagination">
      <button :disabled="currentPage <= 1" @click="currentPage--">{{ t('common.prevPage') }}</button>
      <span class="page-info">{{ t('common.pageInfo', { current: currentPage, total: totalPages }) }}</span>
      <button :disabled="currentPage >= totalPages" @click="currentPage++">{{ t('common.nextPage') }}</button>
    </div>
    <div v-if="expandedRevision" class="detail-panel" :style="{ height: detailPanelHeight + 'px' }">
      <div class="drag-bar" @mousedown="onDragStart" @touchstart="onDragStart">
        <div class="drag-handle"></div>
      </div>
      <div class="detail-header">
        <h4>{{ t('logView.detailTitle', { revision: expandedRevision }) }}</h4>
        <button class="close-btn" @click="closeDetail" :title="t('logView.closeDetail')">
          <X :size="16" />
        </button>
      </div>
      <p class="detail-message">{{ expandedEntry?.message }}</p>
      <div class="detail-split">
        <div class="detail-left">
          <h5>{{ t('logView.changedPaths') }}</h5>
          <div
            v-for="cp in expandedEntry?.changedPaths"
            :key="cp.path"
            class="changed-path"
            :class="{ active: selectedFilePath === cp.path }"
            @click="selectFile(cp.path)"
            @contextmenu.prevent="openFileContextMenu($event, cp)"
          >
            <span class="action" :class="actionClass(cp.action)">{{ cp.action }}</span>
            <span class="path-text">{{ displayChangedPath(cp.path) }}</span>
          </div>
        </div>
        <div class="detail-right">
          <div v-if="fileDiffLoading" class="diff-loading">{{ t('common.loading') }}</div>
          <div v-else-if="isBinaryFile" class="diff-binary">{{ t('common.binaryFile') }}</div>
          <InlineDiff v-else-if="fileDiffText" :diff-text="fileDiffText" :empty-hint="t('common.viewDiff')" />
          <div v-else class="diff-placeholder">{{ t('common.viewDiff') }}</div>
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
    <ContextMenu
      :visible="fileCtxMenu.visible"
      :x="fileCtxMenu.x"
      :y="fileCtxMenu.y"
      :items="fileCtxMenuItems"
      @close="fileCtxMenu.visible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { X, Copy, RotateCcw, ExternalLink, FolderOpen, Eye, Download } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import type { LogEntry, ChangedPath } from '../types/svn'
import type { MenuItem } from '../components/ContextMenu.vue'
import ContextMenu from '../components/ContextMenu.vue'
import InlineDiff from '../components/InlineDiff.vue'
import { t } from '../locales'

const props = defineProps<{
  repoPath: string
  logEntries: LogEntry[]
  wcRevision: number
  root: string
  loading: boolean
}>()

const rootUrlPath = computed(() => {
  if (!props.root) return ''
  try {
    return new URL(props.root).pathname.replace(/\/$/, '')
  } catch {
    return ''
  }
})

function displayChangedPath(svnPath: string): string {
  const prefix = rootUrlPath.value
  if (!prefix) return svnPath.replace(/^\//, '')
  if (svnPath.startsWith(prefix + '/')) return svnPath.slice(prefix.length + 1)
  if (svnPath === prefix) return ''
  return svnPath.replace(/^\//, '')
}

const emit = defineEmits<{
  refreshLog: []
}>()

const searchText = ref('')
const authorFilter = ref('')
const dateFrom = ref('')
const dateTo = ref('')
const expandedRevision = ref<number | null>(null)
const currentPage = ref(1)
const pageSize = 50
const selectedFilePath = ref<string | null>(null)
const fileDiffText = ref('')
const fileDiffLoading = ref(false)
const isBinaryFile = ref(false)
const detailPanelHeight = ref(300)
const isDragging = ref(false)
const startY = ref(0)
const startHeight = ref(0)

const authors = computed(() => {
  const set = new Set(props.logEntries.map((e) => e.author))
  return Array.from(set).sort()
})

const filteredEntries = computed(() => {
  let entries = props.logEntries
  if (authorFilter.value) {
    entries = entries.filter((e) => e.author === authorFilter.value)
  }
  if (dateFrom.value) {
    const from = new Date(dateFrom.value).getTime()
    entries = entries.filter((e) => new Date(e.date).getTime() >= from)
  }
  if (dateTo.value) {
    const to = new Date(dateTo.value)
    to.setHours(23, 59, 59, 999)
    const toTime = to.getTime()
    entries = entries.filter((e) => new Date(e.date).getTime() <= toTime)
  }
  if (searchText.value) {
    const text = searchText.value.toLowerCase()
    entries = entries.filter(
      (e) =>
        e.message.toLowerCase().includes(text) ||
        e.author.toLowerCase().includes(text),
    )
  }
  return entries
})

const totalPages = computed(() => Math.max(1, Math.ceil(filteredEntries.value.length / pageSize)))

const pagedEntries = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return filteredEntries.value.slice(start, start + pageSize)
})

const expandedEntry = computed(() => {
  if (!expandedRevision.value) return null
  return props.logEntries.find((e) => e.revision === expandedRevision.value)
})

watch([authorFilter, dateFrom, dateTo, searchText], () => {
  currentPage.value = 1
})

function toggleDetail(revision: number) {
  if (expandedRevision.value === revision) {
    closeDetail()
  } else {
    expandedRevision.value = revision
    selectedFilePath.value = null
    fileDiffText.value = ''
    isBinaryFile.value = false
  }
}

function closeDetail() {
  expandedRevision.value = null
  selectedFilePath.value = null
  fileDiffText.value = ''
  isBinaryFile.value = false
}

function onDragStart(e: MouseEvent | TouchEvent) {
  isDragging.value = true
  startY.value = 'touches' in e ? e.touches[0].clientY : e.clientY
  startHeight.value = detailPanelHeight.value
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
  document.addEventListener('touchmove', onDragMove)
  document.addEventListener('touchend', onDragEnd)
}

function onDragMove(e: MouseEvent | TouchEvent) {
  if (!isDragging.value) return
  const currentY = 'touches' in e ? e.touches[0].clientY : e.clientY
  const delta = startY.value - currentY
  const newHeight = startHeight.value + delta
  detailPanelHeight.value = Math.max(100, Math.min(600, newHeight))
}

function onDragEnd() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
  document.removeEventListener('touchmove', onDragMove)
  document.removeEventListener('touchend', onDragEnd)
}

async function selectFile(filePath: string) {
  if (selectedFilePath.value === filePath) return
  selectedFilePath.value = filePath
  fileDiffText.value = ''
  isBinaryFile.value = false
  fileDiffLoading.value = true
  try {
    const result = await invoke<string>('svn_diff', {
      path: props.repoPath,
      target: { type: 'fileAtRevision', data: { path: filePath, revision: String(expandedRevision.value), baseRevision: String(expandedRevision.value! - 1) } },
    })
    if (result.includes('Binary files')) {
      isBinaryFile.value = true
    } else {
      fileDiffText.value = result
    }
  } catch (error) {
    const err = error as Error
    fileDiffText.value = t('common.error') + ': ' + (err.message || String(error))
  } finally {
    fileDiffLoading.value = false
  }
}

function formatDate(dateStr: string) {
  try {
    const d = new Date(dateStr)
    const m = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    const h = String(d.getHours()).padStart(2, '0')
    const min = String(d.getMinutes()).padStart(2, '0')
    return `${m}-${day} ${h}:${min}`
  } catch {
    return dateStr
  }
}

function actionClass(action: string) {
  if (action === 'A') return 'added'
  if (action === 'D') return 'deleted'
  if (action === 'R') return 'replaced'
  return 'modified'
}

function isLocal(entry: LogEntry): boolean {
  return entry.revision <= props.wcRevision
}

function refresh() {
  emit('refreshLog')
}

const ctxMenu = ref({ visible: false, x: 0, y: 0, entry: null as LogEntry | null })

function openContextMenu(e: MouseEvent, entry: LogEntry) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, entry }
}

const ctxMenuItems = computed<MenuItem[]>(() => {
  const entry = ctxMenu.value.entry
  if (!entry) return []
  const toast = useToastStore()

  return [
    {
      label: t('contextMenu.showChanges'),
      icon: Eye,
      action: () => { toggleDetail(entry.revision) },
    },
    {
      label: t('contextMenu.copyRevision'),
      icon: Copy,
      action: () => {
        navigator.clipboard.writeText(String(entry.revision))
        toast.success(t('contextMenu.copySuccess'))
      },
    },
    { divider: true },
    {
      label: t('contextMenu.updateToRevision'),
      icon: Download,
      action: async () => {
        const rev = prompt(t('contextMenu.revisionInput'))
        if (rev) {
          try {
            await invoke('svn_update_to_revision', { path: props.repoPath, revision: parseInt(rev) })
            toast.success(t('contextMenu.updateToRevision'))
          } catch (e) { toast.error(String(e)) }
        }
      },
    },
    {
      label: t('contextMenu.revertToRevision'),
      icon: RotateCcw,
      action: async () => {
        if (confirm(`Revert to revision ${entry.revision}?`)) {
          try {
            await invoke('svn_update_to_revision', { path: props.repoPath, revision: entry.revision })
            toast.success(t('contextMenu.revertToRevision'))
            refresh()
          } catch (e) { toast.error(String(e)) }
        }
      },
    },
  ]
})

const fileCtxMenu = ref({ visible: false, x: 0, y: 0, filePath: '', _revision: 0 })

function repoPathToLocal(repoRelativePath: string): string {
  const rel = repoRelativePath.replace(/^\//, '').replace(/\//g, '\\')
  return rel ? `${props.repoPath}\\${rel}` : props.repoPath
}

function openFileContextMenu(e: MouseEvent, cp: ChangedPath) {
  fileCtxMenu.value = { visible: true, x: e.clientX, y: e.clientY, filePath: cp.path, _revision: expandedRevision.value ?? 0 }
}

const fileCtxMenuItems = computed<MenuItem[]>(() => {
  const { filePath } = fileCtxMenu.value
  if (!filePath) return []
  const localPath = repoPathToLocal(filePath)
  const toast = useToastStore()

  return [
    {
      label: t('contextMenu.copyPath'),
      icon: Copy,
      action: () => {
        navigator.clipboard.writeText(displayChangedPath(filePath))
        toast.success(t('contextMenu.copySuccess'))
      },
    },
    {
      label: t('contextMenu.copyAbsPath'),
      icon: Copy,
      action: () => {
        navigator.clipboard.writeText(localPath)
        toast.success(t('contextMenu.copySuccess'))
      },
    },
    {
      label: t('contextMenu.showInExplorer'),
      icon: FolderOpen,
      action: async () => {
        try { await invoke('open_in_system', { path: localPath }) } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.openWithEditor'),
      icon: ExternalLink,
      action: async () => {
        try { await invoke('open_file_with_default_app', { path: localPath }) } catch (e) { toast.error(String(e)) }
      },
    },
  ]
})

onMounted(() => {
  refresh()
})
</script>

<style scoped>
.log-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.filter-bar {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
  flex-wrap: wrap;
}
.filter-select,
.filter-date {
  padding: 5px var(--spacing-sm);
  border: 1px solid var(--border-input);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
}
.filter-select {
  min-width: 100px;
}
.filter-date {
  width: 140px;
}
.date-separator {
  color: var(--text-muted);
  font-size: 13px;
}
.search-input {
  flex: 1;
  min-width: 150px;
  padding: 5px var(--spacing-md);
  border: 1px solid var(--border-input);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
  order: 1;
}
.refresh-btn {
  padding: 5px var(--spacing-md);
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-primary);
  transition: all 0.2s ease;
}
.refresh-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.log-table {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}
table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  font-size: 13px;
}
th,
td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: left;
  border-bottom: 1px solid var(--border-light);
  white-space: nowrap;
}
th {
  background: var(--bg-secondary);
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 1;
  color: var(--text-primary);
  user-select: none;
}
.col-revision {
  width: 70px;
  min-width: 70px;
}
.col-author {
  width: 120px;
  min-width: 100px;
}
.col-date {
  width: 130px;
  min-width: 110px;
}
.col-message {
  flex: 1;
  min-width: 150px;
  white-space: normal;
  word-break: break-word;
}
tr:hover {
  background: var(--bg-hover);
  cursor: pointer;
}
tr.expanded {
  background: var(--bg-active);
}
.empty-row {
  text-align: center;
  color: var(--text-muted);
  padding: var(--spacing-xl) 0;
}
tr.non-local {
  opacity: 0.45;
}
tr.non-local:hover {
  opacity: 0.7;
}
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) 0;
  border-top: 1px solid var(--border-light);
  font-size: 13px;
  background: var(--bg-primary);
  border-radius: 0 0 8px 8px;
}
.pagination button {
  padding: 4px var(--spacing-md);
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
  transition: all 0.2s ease;
}
.pagination button:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.pagination button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.page-info {
  color: var(--text-secondary);
}
.detail-panel {
  border-top: 1px solid var(--border-color);
  padding: 0;
  background: var(--bg-secondary);
  border-radius: 0 0 8px 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 200px;
  max-height: 50vh;
}
.drag-bar {
  height: 6px;
  cursor: ns-resize;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
}
.drag-bar:hover {
  background: var(--bg-hover);
}
.drag-handle {
  width: 40px;
  height: 2px;
  background: var(--border-light);
  border-radius: 1px;
}
.drag-bar:hover .drag-handle,
.drag-bar:active .drag-handle {
  background: var(--border-color);
}
.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-md);
  margin: var(--spacing-sm) 0;
}
.detail-header h4 {
  margin: 0;
  font-size: 14px;
  color: var(--text-primary);
}
.close-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 4px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}
.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.detail-message {
  color: var(--text-primary);
  margin: 0 var(--spacing-md) var(--spacing-sm);
}
.changed-paths {
  margin-top: var(--spacing-sm);
}
.changed-paths h5 {
  margin: 0 0 4px;
  font-size: 13px;
  color: var(--text-secondary);
}
.changed-path {
  font-size: 12px;
  padding: 2px 0;
  font-family: monospace;
  display: flex;
  align-items: center;
  gap: 6px;
}
.action {
  display: inline-block;
  width: 18px;
  text-align: center;
  font-weight: bold;
  font-size: 11px;
  border-radius: 2px;
  color: #fff;
  padding: 1px 0;
}
.action.modified {
  background: var(--warning-color);
}
.action.added {
  background: var(--success-color);
}
.action.deleted {
  background: var(--danger-color);
}
.action.replaced {
  background: var(--purple-color);
}
.path-text {
  color: var(--text-primary);
}
.detail-actions {
  margin-top: var(--spacing-md);
  display: flex;
  gap: var(--spacing-sm);
}
.detail-split {
  display: flex;
  gap: var(--spacing-md);
  margin: 0 var(--spacing-md) var(--spacing-md);
  flex: 1;
  min-height: 150px;
  overflow: hidden;
}
.detail-left {
  width: 300px;
  flex-shrink: 0;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
}
.detail-left h5 {
  margin: 0;
  padding: 6px var(--spacing-sm);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  position: sticky;
  top: 0;
}
.detail-left .changed-path {
  cursor: pointer;
  padding: 2px var(--spacing-sm);
}
.detail-left .changed-path:hover {
  background: var(--bg-hover);
}
.detail-left .changed-path.active {
  background: var(--bg-active);
}
.detail-right {
  flex: 1;
  min-width: 0;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
}
.diff-loading,
.diff-binary,
.diff-placeholder {
  color: var(--text-muted);
  text-align: center;
  padding: 40px var(--spacing-md);
  font-size: 13px;
}
@media (max-width: 768px) {
  .filter-bar {
    gap: var(--spacing-xs);
  }
  .search-input {
    min-width: 100%;
    order: 1;
  }
  .detail-split {
    flex-direction: column;
  }
  .detail-left {
    width: 100%;
    max-height: 150px;
  }
}
</style>
