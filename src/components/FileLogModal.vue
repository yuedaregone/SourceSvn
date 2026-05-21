<template>
  <div v-if="visible" class="dialog-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
    <div class="modal" :style="{ width: width + 'px', height: height + 'px' }">
      <div class="modal-header">
        <div class="modal-header-left">
          <div class="modal-icon">
            <ScrollText :size="16" />
          </div>
          <span class="modal-title">{{ t('fileLog.title') }}</span>
          <span class="modal-filepath">{{ filePath }}</span>
        </div>
        <button class="btn btn-icon btn-ghost" @click="close">
          <X :size="16" />
        </button>
      </div>

      <div class="modal-body">
        <div v-if="loading" class="loading-state">
          <div class="spinner" />
          <span>{{ t('common.loading') }}</span>
        </div>
        <div v-else-if="entries.length === 0" class="empty-state">
          <History :size="32" />
          <span>{{ t('fileLog.noLogs') }}</span>
        </div>
        <div v-else class="log-table-wrapper">
          <table class="log-table">
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
                v-for="entry in entries"
                :key="entry.revision"
                @click="onEntryClick(entry)"
                :class="{ selected: selectedRevision === entry.revision }"
              >
                <td class="col-revision">
                  <span class="revision-badge">{{ entry.revision }}</span>
                </td>
                <td class="col-author">{{ entry.author }}</td>
                <td class="col-date">{{ formatDate(entry.date) }}</td>
                <td class="col-message">{{ entry.message }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div v-if="selectedEntry" class="detail-panel">
        <div class="detail-header">
          <span class="detail-title">{{ t('logView.detailTitle', { revision: selectedEntry.revision }) }}</span>
          <button class="btn btn-icon btn-ghost btn-sm" @click="selectedRevision = null">
            <X :size="14" />
          </button>
        </div>
        <p class="detail-message">{{ selectedEntry.message }}</p>
        <div v-if="selectedEntry.changedPaths && selectedEntry.changedPaths.length > 0" class="detail-changes">
          <h5>{{ t('logView.changedPaths') }}</h5>
          <div v-for="cp in selectedEntry.changedPaths" :key="cp.path" class="changed-path">
            <span class="action-badge" :class="actionClass(cp.action)">{{ cp.action }}</span>
            <span class="path-text">{{ cp.path }}</span>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-primary" @click="close">
          <X :size="14" />
          <span>{{ t('common.close') }}</span>
        </button>
      </div>

      <div class="resize-handle" @mousedown="onResizeStart"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { X, History, ScrollText } from 'lucide-vue-next'
import type { LogEntry } from '../types/svn'
import { t } from '../locales'

const props = defineProps<{
  visible: boolean
  filePath: string
  repoPath: string
  useRepoPath?: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const overlayMousedown = ref(false)
function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  close()
}

const entries = ref<LogEntry[]>([])
const loading = ref(false)
const selectedRevision = ref<number | null>(null)
const width = ref(720)
const height = ref(640)

const selectedEntry = computed(() => {
  if (!selectedRevision.value) return null
  return entries.value.find(e => e.revision === selectedRevision.value) ?? null
})

watch(() => props.visible, async (val) => {
  if (val && props.filePath) {
    await loadLog()
  } else {
    entries.value = []
    selectedRevision.value = null
  }
})

async function loadLog() {
  loading.value = true
  entries.value = []
  selectedRevision.value = null

  const logPath = props.useRepoPath ? props.repoPath : props.filePath

  try {
    const result = await invoke<{ entries: LogEntry[]; wcRevision: number; root: string }>('svn_log_server', {
      path: logPath,
      limit: 100,
    })
    entries.value = result.entries
  } catch (e1) {
    console.warn('svn_log_server failed, trying svn_log:', e1)
    try {
      entries.value = await invoke<LogEntry[]>('svn_log', {
        path: logPath,
        limit: 100,
      })
    } catch (e2) {
      console.error('Failed to load log:', e2)
    }
  } finally {
    loading.value = false
  }
}

async function onEntryClick(entry: LogEntry) {
  if (selectedRevision.value === entry.revision) {
    selectedRevision.value = null
    return
  }
  selectedRevision.value = entry.revision
  if (!entry.changedPaths || entry.changedPaths.length === 0) {
    const changedPath = props.useRepoPath ? props.repoPath : props.filePath
    try {
      entry.changedPaths = await invoke('svn_log_changed_paths', {
        path: changedPath,
        revision: entry.revision,
      })
    } catch {
      entry.changedPaths = []
    }
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

function close() {
  emit('close')
}

let isResizing = false
let startX = 0
let startY = 0
let startWidth = 0
let startHeight = 0

function onResizeStart(e: MouseEvent) {
  isResizing = true
  startX = e.clientX
  startY = e.clientY
  startWidth = width.value
  startHeight = height.value
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', onResizeEnd)
}

function onResizeMove(e: MouseEvent) {
  if (!isResizing) return
  const dx = e.clientX - startX
  const dy = e.clientY - startY
  width.value = Math.max(600, Math.min(window.innerWidth - 40, startWidth + dx))
  height.value = Math.max(400, Math.min(window.innerHeight - 40, startHeight + dy))
}

function onResizeEnd() {
  isResizing = false
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
}

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
})
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.modal {
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  animation: scaleIn 0.2s ease;
}

@keyframes scaleIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--color-border);
}

.modal-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-width: 0;
  flex: 1;
}

.modal-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: var(--color-accent-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent);
  flex-shrink: 0;
}

.modal-title {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--color-text-primary);
  flex-shrink: 0;
}

.modal-filepath {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  height: 32px;
  padding: 0 var(--space-3);
  border: 1px solid var(--color-border-input);
  background: var(--color-bg-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-sm);
  color: var(--color-text-primary);
  transition: all var(--transition-fast);
}

.btn:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.btn-primary {
  background: var(--color-accent);
  color: var(--color-text-inverse);
  border-color: var(--color-accent);
}

.btn-primary:hover {
  background: var(--color-accent-hover);
  border-color: var(--color-accent-hover);
}

.btn-icon {
  padding: 0;
  width: 32px;
}

.btn-ghost {
  border: none;
  background: transparent;
}

.btn-ghost:hover {
  background: var(--color-bg-hover);
}

.btn-sm {
  height: 28px;
  width: 28px;
}

.modal-body {
  flex: 1;
  overflow: auto;
  min-height: 0;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-10);
  color: var(--color-text-muted);
}

.log-table-wrapper {
  overflow: auto;
}

.log-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-base);
}

.log-table th {
  position: sticky;
  top: 0;
  background: var(--color-bg-secondary);
  text-align: left;
  padding: var(--space-2) var(--space-3);
  font-weight: 600;
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--color-border);
  z-index: 1;
  user-select: none;
}

.log-table td {
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--color-border-light);
  vertical-align: middle;
}

.log-table tr {
  cursor: pointer;
  transition: background var(--transition-fast);
}

.log-table tr:hover {
  background: var(--color-bg-hover);
}

.log-table tr.selected {
  background: var(--color-bg-active);
}

.col-revision {
  width: 80px;
  min-width: 80px;
}

.col-author {
  width: 100px;
  min-width: 80px;
}

.col-date {
  width: 120px;
  min-width: 100px;
}

.col-message {
  min-width: 200px;
  white-space: normal;
  word-break: break-word;
}

.revision-badge {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--color-accent);
  background: var(--color-accent-muted);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}

.detail-panel {
  border-top: 1px solid var(--color-border);
  padding: var(--space-3) var(--space-5);
  background: var(--color-bg-secondary);
  max-height: 200px;
  overflow: auto;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}

.detail-title {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-text-primary);
}

.detail-message {
  margin: 0 0 var(--space-2);
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.detail-changes h5 {
  margin: 0 0 var(--space-1);
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.changed-path {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) 0;
  font-size: var(--text-sm);
  font-family: var(--font-mono);
}

.action-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  width: 22px;
  height: 22px;
  text-align: center;
  font-weight: 700;
  font-size: var(--text-xs);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.action-badge.modified {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}

.action-badge.added {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.action-badge.deleted {
  background: var(--color-danger-muted);
  color: var(--color-danger);
}

.action-badge.replaced {
  background: var(--color-purple-muted);
  color: var(--color-purple);
}

.path-text {
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--color-border);
  gap: var(--space-2);
}

.resize-handle {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 16px;
  height: 16px;
  cursor: nwse-resize;
}

.resize-handle::after {
  content: '';
  position: absolute;
  bottom: 4px;
  right: 4px;
  width: 8px;
  height: 8px;
  border-right: 2px solid var(--color-border);
  border-bottom: 2px solid var(--color-border);
}
</style>
