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
            <td class="col-revision">
              <span class="revision-badge">{{ entry.revision }}</span>
            </td>
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
      <button :disabled="currentPage <= 1" @click="currentPage--" class="page-btn">
        <ChevronLeft :size="14" />
        {{ t('common.prevPage') }}
      </button>
      <span class="page-info">{{ t('common.pageInfo', { current: currentPage, total: totalPages }) }}</span>
      <button :disabled="currentPage >= totalPages" @click="currentPage++" class="page-btn">
        {{ t('common.nextPage') }}
        <ChevronRight :size="14" />
      </button>
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
        <div class="detail-left" :style="{ width: splitLeftWidth + 'px' }">
          <h5>{{ t('logView.changedPaths') }}</h5>
          <div v-if="fetchChangedPathsLoading" class="changed-paths-loading">
            <div class="spinner" />
            <span>{{ t('common.loading') }}</span>
          </div>
          <div
            v-for="cp in displayChangedPaths"
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
        <div class="h-drag-divider" @mousedown="onHDragStart" @touchstart="onHDragStart">
          <div class="h-drag-handle"></div>
        </div>
        <div class="detail-right">
          <div v-if="fileDiffLoading" class="diff-loading">
            <div class="spinner" />
            <span>{{ t('common.loading') }}</span>
          </div>
          <div v-else-if="isBinaryFile" class="diff-binary">
            <FileIcon :size="24" />
            <span>{{ t('common.binaryFile') }}</span>
          </div>
          <CodeDiffViewer v-else-if="oldContent !== undefined && newContent !== undefined" :old-string="oldContent" :new-string="newContent" :filename="selectedFilePath || undefined" :empty-hint="t('common.viewDiff')" />
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
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { X, Copy, RotateCcw, ExternalLink, FolderOpen, Eye, Download, ChevronLeft, ChevronRight, File as FileIcon } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import type { LogEntry, ChangedPath } from '../types/svn'
import type { MenuItem } from '../components/ContextMenu.vue'
import ContextMenu from '../components/ContextMenu.vue'
import CodeDiffViewer from '../components/CodeDiffViewer.vue'
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
const oldContent = ref<string | undefined>(undefined)
const newContent = ref<string | undefined>(undefined)
const fileDiffLoading = ref(false)
const isBinaryFile = ref(false)
const detailPanelHeight = ref(300)
const isDragging = ref(false)
const startY = ref(0)
const startHeight = ref(0)
const splitLeftWidth = ref(400)
const isHDragging = ref(false)
const hStartX = ref(0)
const hStartWidth = ref(0)
const fetchedChangedPaths = ref<ChangedPath[] | null>(null)
const fetchChangedPathsLoading = ref(false)

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

async function toggleDetail(revision: number) {
  if (expandedRevision.value === revision) {
    closeDetail()
  } else {
    expandedRevision.value = revision
    selectedFilePath.value = null
    oldContent.value = undefined
    newContent.value = undefined
    isBinaryFile.value = false
    fetchedChangedPaths.value = null

    // 非本地版本需要额外获取变更文件列表
    const entry = props.logEntries.find((e) => e.revision === revision)
    if (entry && (!entry.changedPaths || entry.changedPaths.length === 0) && !isLocal(entry)) {
      fetchChangedPathsLoading.value = true
      try {
        const paths = await invoke<ChangedPath[]>('svn_log_changed_paths', {
          path: props.repoPath,
          revision: revision,
        })
        fetchedChangedPaths.value = paths
      } catch {
        fetchedChangedPaths.value = []
      } finally {
        fetchChangedPathsLoading.value = false
      }
    }
  }
}

function closeDetail() {
  expandedRevision.value = null
  selectedFilePath.value = null
  oldContent.value = undefined
  newContent.value = undefined
  isBinaryFile.value = false
  fetchedChangedPaths.value = null
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

// 水平拖动：调整左右面板宽度
function onHDragStart(e: MouseEvent | TouchEvent) {
  isHDragging.value = true
  hStartX.value = 'touches' in e ? e.touches[0].clientX : e.clientX
  hStartWidth.value = splitLeftWidth.value
  document.addEventListener('mousemove', onHDragMove)
  document.addEventListener('mouseup', onHDragEnd)
  document.addEventListener('touchmove', onHDragMove)
  document.addEventListener('touchend', onHDragEnd)
}

function onHDragMove(e: MouseEvent | TouchEvent) {
  if (!isHDragging.value) return
  const currentX = 'touches' in e ? e.touches[0].clientX : e.clientX
  const delta = currentX - hStartX.value
  splitLeftWidth.value = Math.max(150, Math.min(600, hStartWidth.value + delta))
}

function onHDragEnd() {
  isHDragging.value = false
  document.removeEventListener('mousemove', onHDragMove)
  document.removeEventListener('mouseup', onHDragEnd)
  document.removeEventListener('touchmove', onHDragMove)
  document.removeEventListener('touchend', onHDragEnd)
}

// 合并变更路径：优先使用已获取的数据，其次使用条目自带数据
const displayChangedPaths = computed(() => {
  if (fetchedChangedPaths.value !== null) return fetchedChangedPaths.value
  return expandedEntry.value?.changedPaths ?? []
})

async function selectFile(filePath: string) {
  if (selectedFilePath.value === filePath) return
  if (expandedRevision.value == null) return
  selectedFilePath.value = filePath
  oldContent.value = undefined
  newContent.value = undefined
  isBinaryFile.value = false
  fileDiffLoading.value = true
  try {
    const rev = String(expandedRevision.value)
    const baseRev = String(expandedRevision.value - 1)
    // 先用 svn diff 检测是否为二进制文件
    const diff = await invoke<string>('svn_diff', {
      path: props.repoPath,
      target: { type: 'fileAtRevision', data: { path: filePath, revision: rev, baseRevision: baseRev } },
    })
    if (diff.includes('Binary files')) {
      isBinaryFile.value = true
    } else {
      // 文本文件：获取两个版本的内容
      const [old, cur] = await Promise.all([
        invoke<string>('svn_cat_at_revision', { repoPath: props.repoPath, filePath, revision: baseRev }),
        invoke<string>('svn_cat_at_revision', { repoPath: props.repoPath, filePath, revision: rev }),
      ])
      oldContent.value = old
      newContent.value = cur
    }
  } catch (error) {
    const err = error as Error
    oldContent.value = undefined
    newContent.value = undefined
    // 显示错误信息
    oldContent.value = ''
    newContent.value = t('common.error') + ': ' + (err.message || String(error))
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

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
  document.removeEventListener('touchmove', onDragMove)
  document.removeEventListener('touchend', onDragEnd)
  document.removeEventListener('mousemove', onHDragMove)
  document.removeEventListener('mouseup', onHDragEnd)
  document.removeEventListener('touchmove', onHDragMove)
  document.removeEventListener('touchend', onHDragEnd)
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
  gap: var(--space-2);
  margin-bottom: var(--space-3);
  flex-wrap: wrap;
}

.filter-select,
.filter-date {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-input);
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  transition: all var(--transition-fast);
}

.filter-select:focus,
.filter-date:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px var(--color-accent-muted);
}

.filter-select {
  min-width: 100px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%238b95a5' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right var(--space-3) center;
  padding-right: var(--space-8);
}

.filter-date {
  width: 140px;
}

.date-separator {
  color: var(--color-text-muted);
  font-size: var(--text-base);
}

.search-input {
  flex: 1;
  min-width: 150px;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-input);
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  transition: all var(--transition-fast);
  order: 1;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px var(--color-accent-muted);
}

.log-table {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-bg-primary);
}

table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  font-size: var(--text-base);
}

th,
td {
  padding: var(--space-3) var(--space-4);
  text-align: left;
  border-bottom: 1px solid var(--color-border-light);
  white-space: nowrap;
}

th {
  background: var(--color-bg-secondary);
  font-weight: 600;
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  position: sticky;
  top: 0;
  z-index: 1;
  user-select: none;
}

.col-revision {
  width: 80px;
  min-width: 80px;
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

tr {
  transition: background var(--transition-fast);
}

tr:hover {
  background: var(--color-bg-hover);
  cursor: pointer;
}

tr.expanded {
  background: var(--color-bg-active);
}

.empty-row {
  text-align: center;
  color: var(--color-text-muted);
  padding: var(--space-8) 0;
}

tr.non-local {
  opacity: 0.5;
}

tr.non-local:hover {
  opacity: 0.8;
}

.revision-badge {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--color-accent);
  background: var(--color-accent-muted);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-3) 0;
  border-top: 1px solid var(--color-border-light);
  font-size: var(--text-base);
  background: var(--color-bg-primary);
}

.page-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-input);
  background: var(--color-bg-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-sm);
  color: var(--color-text-primary);
  transition: all var(--transition-fast);
}

.page-btn:hover:not(:disabled) {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.page-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.page-info {
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
}

.detail-panel {
  border-top: 1px solid var(--color-border);
  padding: 0;
  background: var(--color-bg-secondary);
  border-radius: 0 0 var(--radius-lg) var(--radius-lg);
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
  transition: background var(--transition-fast);
}

.drag-bar:hover {
  background: var(--color-bg-hover);
}

.drag-handle {
  width: 40px;
  height: 2px;
  background: var(--color-border-light);
  border-radius: var(--radius-full);
  transition: background var(--transition-fast);
}

.drag-bar:hover .drag-handle,
.drag-bar:active .drag-handle {
  background: var(--color-border);
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-4);
  margin: var(--space-3) 0;
}

.detail-header h4 {
  margin: 0;
  font-size: var(--text-md);
  color: var(--color-text-primary);
  font-weight: 600;
}

.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
}

.close-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.detail-message {
  color: var(--color-text-primary);
  margin: 0 var(--space-4) var(--space-3);
  font-size: var(--text-base);
  line-height: 1.6;
}

.changed-path {
  font-size: var(--text-sm);
  padding: var(--space-1) 0;
  font-family: var(--font-mono);
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  width: 24px;
  height: 20px;
  text-align: center;
  font-weight: bold;
  font-size: var(--text-xs);
  border-radius: var(--radius-sm);
  color: #fff;
  flex-shrink: 0;
}

.action.modified {
  background: var(--color-warning);
}

.action.added {
  background: var(--color-success);
}

.action.deleted {
  background: var(--color-danger);
}

.action.replaced {
  background: var(--color-purple);
}

.path-text {
  color: var(--color-text-primary);
}

.detail-split {
  display: flex;
  gap: 0;
  margin: 0 var(--space-4) var(--space-4);
  flex: 1;
  min-height: 150px;
  overflow: hidden;
}

.detail-left {
  flex-shrink: 0;
  overflow-x: hidden;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg-primary);
}

.detail-left h5 {
  margin: 0;
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border-light);
  position: sticky;
  top: 0;
  z-index: 1;
}

.detail-left .changed-path {
  cursor: pointer;
  padding: var(--space-1) var(--space-3);
  transition: background var(--transition-fast);
}

.detail-left .changed-path:hover {
  background: var(--color-bg-hover);
}

.detail-left .changed-path.active {
  background: var(--color-bg-active);
}

.h-drag-divider {
  width: 6px;
  cursor: ew-resize;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background var(--transition-fast);
}

.h-drag-divider:hover {
  background: var(--color-bg-hover);
}

.h-drag-handle {
  width: 2px;
  height: 40px;
  background: var(--color-border-light);
  border-radius: var(--radius-full);
  transition: background var(--transition-fast);
}

.h-drag-divider:hover .h-drag-handle,
.h-drag-divider:active .h-drag-handle {
  background: var(--color-border);
}

.changed-paths-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-6) var(--space-3);
  color: var(--color-text-muted);
  font-size: var(--text-sm);
}

.detail-right {
  flex: 1;
  min-width: 0;
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg-primary);
}

.diff-loading,
.diff-binary,
.diff-placeholder {
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--space-10) var(--space-4);
  font-size: var(--text-base);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

@media (max-width: 768px) {
  .filter-bar {
    gap: var(--space-1);
  }

  .search-input {
    min-width: 100%;
    order: 1;
  }

  .detail-split {
    flex-direction: column;
  }

  .detail-left {
    width: 100% !important;
    max-height: 150px;
  }

  .h-drag-divider {
    display: none;
  }
}
</style>
