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
      <button @click="refresh" class="refresh-btn" :disabled="props.loading" :title="t('common.refresh')">
        <RefreshCw :size="16" />
      </button>
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
          >
            <span class="action" :class="actionClass(cp.action)">{{ cp.action }}</span>
            <span class="path-text">{{ cp.path }}</span>
          </div>
        </div>
        <div class="detail-right">
          <div v-if="fileDiffLoading" class="diff-loading">{{ t('common.loading') }}</div>
          <div v-else-if="isBinaryFile" class="diff-binary">{{ t('common.binaryFile') }}</div>
          <pre v-else-if="fileDiffText" class="diff-content"><template v-for="(line, i) in fileDiffLines" :key="i"><span :class="diffLineClass(line)">{{ line }}</span>
</template></pre>
          <div v-else class="diff-placeholder">{{ t('common.viewDiff') }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { RefreshCw, X } from 'lucide-vue-next'
import type { LogEntry } from '../types/svn'
import { t } from '../locales'

const props = defineProps<{
  repoPath: string
  logEntries: LogEntry[]
  wcRevision: number
  loading: boolean
}>()

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

const fileDiffLines = computed(() => {
  if (!fileDiffText.value) return []
  return fileDiffText.value.split('\n')
})

function diffLineClass(line: string) {
  if (line.startsWith('+') && !line.startsWith('+++')) return 'diff-add'
  if (line.startsWith('-') && !line.startsWith('---')) return 'diff-del'
  if (line.startsWith('@@')) return 'diff-hunk'
  return ''
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
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}
.filter-select,
.filter-date {
  padding: 5px 8px;
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
  padding: 5px 12px;
  border: 1px solid var(--border-input);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
}
.refresh-btn {
  padding: 5px 12px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-primary);
}
.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.log-table {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
}
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
th,
td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid var(--border-light);
}
th {
  background: var(--bg-secondary);
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 1;
  color: var(--text-primary);
}
.col-revision {
  width: 80px;
}
.col-author {
  width: 100px;
}
.col-date {
  width: 120px;
}
.col-message {
  min-width: 200px;
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
  padding: 24px 0;
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
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid var(--border-light);
  font-size: 13px;
  background: var(--bg-primary);
}
.pagination button {
  padding: 4px 12px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
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
  border-radius: 0 0 4px 4px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
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
  padding: 0 12px;
  margin: 8px 0;
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
}
.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.detail-message {
  color: var(--text-primary);
  margin: 0 12px 8px;
}
.changed-paths {
  margin-top: 8px;
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
  margin-top: 12px;
  display: flex;
  gap: 8px;
}
.detail-split {
  display: flex;
  gap: 12px;
  margin: 0 12px 12px;
  flex: 1;
  min-height: 100px;
  overflow: hidden;
}
.detail-left {
  width: 280px;
  flex-shrink: 0;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
}
.detail-left h5 {
  margin: 0;
  padding: 6px 8px;
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
  border-radius: 4px;
  background: var(--bg-primary);
}
.diff-loading,
.diff-binary,
.diff-placeholder {
  color: var(--text-muted);
  text-align: center;
  padding: 40px 16px;
  font-size: 13px;
}
.diff-content {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  white-space: pre;
  padding: 8px 12px;
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
</style>
