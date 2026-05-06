<template>
  <div class="local-changes-view">
    <div class="left-panel" :style="{ width: leftPanelWidth + 'px' }">
      <div class="file-list-header">
        <label class="select-all">
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          <span>{{ t('common.selectAll') }}</span>
        </label>
        <span class="selected-count">{{ t('localChanges.selectedCount', { count: selectedPaths.size }) }}</span>
      </div>
      <div class="file-list" :class="{ loading: props.loading }">
        <div v-if="props.loading" class="loading-overlay">
          <RefreshCw :size="24" class="spin" />
        </div>
        <div
          v-for="file in props.localChanges"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedFile === file.path }"
          @click="selectFile(file)"
          @contextmenu.prevent="openContextMenu($event, file)"
        >
          <input
            type="checkbox"
            :checked="selectedPaths.has(file.path)"
            @click.stop="toggleFile(file.path)"
            :disabled="props.loading"
          />
          <span class="status-badge" :class="file.status">{{ file.status[0].toUpperCase() }}</span>
          <span class="file-path" :class="{ dir: file.isDirectory }">{{ displayPath(file.path) }}</span>
        </div>
        <div v-if="!props.loading && props.localChanges.length === 0" class="empty-list">{{ t('common.noLocalChanges') }}</div>
      </div>
      <div class="commit-section">
        <textarea
          v-model="commitMessage"
          :placeholder="t('localChanges.commitMessage')"
          rows="3"
          class="commit-input"
        ></textarea>
        <div class="commit-actions">
          <button @click="generateAiMessage" :disabled="aiLoading || selectedPaths.size === 0" class="ai-btn" :title="t('localChanges.aiGenerate')">
            <Sparkles :size="16" />
          </button>
          <button @click="cancelCommit" class="cancel-btn icon-btn" :title="t('common.cancel')">
            <X :size="16" />
          </button>
          <button @click="submitCommit" :disabled="!canCommit" class="commit-btn" :title="t('common.submit')">
            <Send :size="16" />
          </button>
        </div>
      </div>
    </div>
    <div class="drag-bar" @mousedown="onDragStart" @touchstart="onDragStart">
      <div class="drag-handle"></div>
    </div>
    <div class="right-panel">
      <InlineDiff v-if="diffContent" :diff-text="diffContent" :empty-hint="t('common.clickToViewDiff')" />
      <div v-else class="diff-placeholder">{{ t('common.clickToViewDiff') }}</div>
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
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Sparkles, RefreshCw, X, Send, RotateCcw, Plus, Trash2, ExternalLink, FolderOpen, Copy, CheckSquare, Square } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import { useConfigStore } from '../stores/configStore'
import type { FileStatus, DiffTarget } from '../types/svn'
import type { MenuItem } from '../components/ContextMenu.vue'
import ContextMenu from '../components/ContextMenu.vue'
import InlineDiff from '../components/InlineDiff.vue'
import { t } from '../locales'

const props = defineProps<{
  repoPath: string
  localChanges: FileStatus[]
  loading: boolean
}>()

const emit = defineEmits<{
  refresh: []
  refreshLocalChanges: []
}>()

const selectedPaths = ref(new Set<string>())
const selectedFile = ref('')
const commitMessage = ref('')
const diffContent = ref('')
const aiLoading = ref(false)
const toast = useToastStore()
const configStore = useConfigStore()

const leftPanelWidth = ref(320)
const isDragging = ref(false)
const startX = ref(0)
const startWidth = ref(0)

function onDragStart(e: MouseEvent | TouchEvent) {
  isDragging.value = true
  startX.value = 'touches' in e ? e.touches[0].clientX : e.clientX
  startWidth.value = leftPanelWidth.value
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
  document.addEventListener('touchmove', onDragMove)
  document.addEventListener('touchend', onDragEnd)
}

function onDragMove(e: MouseEvent | TouchEvent) {
  if (!isDragging.value) return
  const currentX = 'touches' in e ? e.touches[0].clientX : e.clientX
  const delta = currentX - startX.value
  leftPanelWidth.value = Math.max(200, Math.min(800, startWidth.value + delta))
}

function onDragEnd() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
  document.removeEventListener('touchmove', onDragMove)
  document.removeEventListener('touchend', onDragEnd)
}

const ctxMenu = ref({ visible: false, x: 0, y: 0, file: null as FileStatus | null })

function displayPath(filePath: string): string {
  const normRepo = props.repoPath.replace(/[\/\\]+$/, '').replace(/\//g, '\\')
  const normFile = filePath.replace(/\//g, '\\')
  if (normFile.toLowerCase().startsWith(normRepo.toLowerCase() + '\\')) {
    return normFile.slice(normRepo.length + 1)
  }
  return filePath
}

const allSelected = computed(
  () =>
    props.localChanges.length > 0 &&
    props.localChanges.every((f) => selectedPaths.value.has(f.path)),
)

const canCommit = computed(
  () => selectedPaths.value.size > 0 && commitMessage.value.trim().length > 0,
)

function toggleAll() {
  if (allSelected.value) {
    selectedPaths.value = new Set()
  } else {
    selectedPaths.value = new Set(props.localChanges.map((f) => f.path))
  }
}

function toggleFile(path: string) {
  const next = new Set(selectedPaths.value)
  if (next.has(path)) {
    next.delete(path)
  } else {
    next.add(path)
  }
  selectedPaths.value = next
}

async function selectFile(file: FileStatus) {
  selectedFile.value = file.path
  if (file.isDirectory) {
    diffContent.value = ''
    toast.info(t('localChanges.directoryNoDiff'))
    return
  }
  if (file.status === 'deleted' || file.status === 'missing') {
    diffContent.value = ''
    toast.warning(`${t('common.status')}: ${file.status} — ${file.path}`)
    return
  }
  try {
    if (file.status === 'unversioned') {
      diffContent.value = await invoke<string>('diff_unversioned_file', {
        repoPath: props.repoPath,
        filePath: file.path,
      })
    } else {
      const target: DiffTarget = { type: 'file', data: { path: file.path } }
      diffContent.value = await invoke<string>('svn_diff', {
        path: props.repoPath,
        target,
      })
    }
  } catch (e) {
    diffContent.value = ''
    toast.error(String(e), 0)
  }
}

async function generateAiMessage() {
  if (selectedPaths.value.size === 0) return
  aiLoading.value = true
  try {
    const firstPath = Array.from(selectedPaths.value)[0]
    const target: DiffTarget = { type: 'file', data: { path: firstPath } }
    const diff = await invoke<string>('svn_diff', {
      path: props.repoPath,
      target,
    })
    commitMessage.value = await invoke<string>('generate_commit_message', { diff })
  } catch (e) {
    toast.error(t('common.aiReviewFailed', { msg: String(e) }), 0)
  } finally {
    aiLoading.value = false
  }
}

async function submitCommit() {
  if (!canCommit.value) return
  if (configStore.config?.behavior.confirmBeforeCommit) {
    if (!confirm('确认提交?')) return
  }
  try {
    await invoke('svn_commit', {
      path: props.repoPath,
      message: commitMessage.value,
      files: Array.from(selectedPaths.value),
    })
    commitMessage.value = ''
    selectedPaths.value = new Set()
    selectedFile.value = ''
    diffContent.value = ''
    emit('refreshLocalChanges')
    emit('refresh')
  } catch (e) {
    toast.error(String(e), 0)
  }
}

function cancelCommit() {
  commitMessage.value = ''
  selectedPaths.value = new Set()
  selectedFile.value = ''
  diffContent.value = ''
}

function openContextMenu(e: MouseEvent, file: FileStatus) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, file }
  selectedFile.value = file.path
}

const ctxMenuItems = computed<MenuItem[]>(() => {
  const file = ctxMenu.value.file
  if (!file) return []
  const isUnversioned = file.status === 'unversioned'
  const isModified = file.status === 'modified' || file.status === 'conflicted' || file.status === 'missing'

  return [
    {
      label: t('contextMenu.revert'),
      icon: RotateCcw,
      disabled: !isModified,
      action: async () => {
        if (configStore.config?.behavior.confirmBeforeRevert && !confirm(t('contextMenu.revertConfirm'))) return
        try {
          await invoke('svn_revert', { path: props.repoPath, paths: [file.path] })
          emit('refreshLocalChanges')
          toast.success(t('contextMenu.revert'))
        } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.add'),
      icon: Plus,
      disabled: !isUnversioned,
      action: async () => {
        try {
          await invoke('svn_add', { path: props.repoPath, paths: [file.path] })
          emit('refreshLocalChanges')
          toast.success(t('contextMenu.add'))
        } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.delete'),
      icon: Trash2,
      disabled: isUnversioned,
      action: async () => {
        try {
          await invoke('svn_delete', { path: props.repoPath, paths: [file.path], keepLocal: false })
          emit('refreshLocalChanges')
          toast.success(t('contextMenu.delete'))
        } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.deleteFromDisk'),
      icon: Trash2,
      action: async () => {
        try {
          await invoke('delete_files_from_disk', { path: props.repoPath, paths: [file.path] })
          emit('refreshLocalChanges')
          toast.success(t('contextMenu.deleteFromDisk'))
        } catch (e) { toast.error(String(e)) }
      },
    },
    { divider: true },
    {
      label: t('contextMenu.openWithEditor'),
      icon: ExternalLink,
      action: async () => {
        try { await invoke('open_file_with_default_app', { path: file.path }) } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.showInExplorer'),
      icon: FolderOpen,
      action: async () => {
        try { await invoke('open_in_system', { path: file.path }) } catch (e) { toast.error(String(e)) }
      },
    },
    { divider: true },
    {
      label: t('contextMenu.copyPath'),
      icon: Copy,
      action: () => {
        navigator.clipboard.writeText(file.path)
        toast.success(t('contextMenu.copySuccess'))
      },
    },
    {
      label: t('contextMenu.selectAll'),
      icon: CheckSquare,
      action: () => { selectedPaths.value = new Set(props.localChanges.map(f => f.path)) },
    },
    {
      label: t('contextMenu.deselectAll'),
      icon: Square,
      action: () => { selectedPaths.value = new Set() },
    },
  ]
})
</script>

<style scoped>
.local-changes-view {
  display: flex;
  height: 100%;
  gap: 2px;
}
.left-panel {
  flex: none;
  display: flex;
  flex-direction: column;
  min-width: 200px;
  max-width: 800px;
  overflow: hidden;
}
.drag-bar {
  width: 6px;
  cursor: ew-resize;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.drag-bar:hover {
  background: var(--bg-hover);
}
.drag-handle {
  width: 2px;
  height: 40px;
  background: var(--border-light);
  border-radius: 1px;
}
.drag-bar:hover .drag-handle,
.drag-bar:active .drag-handle {
  background: var(--border-color);
}
.right-panel {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
}
.file-list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 13px;
}
.select-all {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}
.selected-count {
  color: var(--text-secondary);
}
.file-list {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  min-height: 0;
  background: var(--bg-primary);
  position: relative;
}
.file-list.loading {
  pointer-events: none;
}
.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  opacity: 0.9;
}
.loading-overlay .spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.file-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid var(--border-light);
}
.file-item:hover {
  background: var(--bg-hover);
}
.file-item.selected {
  background: var(--bg-active);
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
  flex-shrink: 0;
}
.status-badge.modified { background: var(--warning-color); }
.status-badge.added { background: var(--success-color); }
.status-badge.deleted { background: var(--danger-color); }
.status-badge.unversioned { background: var(--text-muted); }
.status-badge.missing { background: #ff7a45; }
.status-badge.conflicted { background: #f5222d; }
/* 复选框样式 - 完全自定义 */
.file-item input[type="checkbox"],
.select-all input[type="checkbox"] {
  width: 14px;
  height: 14px;
  cursor: pointer;
  appearance: none;
  -webkit-appearance: none;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-input);
  border-radius: 3px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}
.file-item input[type="checkbox"]::before,
.select-all input[type="checkbox"]::before {
  content: '';
  width: 6px;
  height: 6px;
  background-color: transparent;
  border-radius: 2px;
  transition: background-color 0.2s;
}
.file-item input[type="checkbox"]:checked::before,
.select-all input[type="checkbox"]:checked::before {
  background-color: var(--accent-color);
}
.file-item input[type="checkbox"]:hover,
.select-all input[type="checkbox"]:hover {
  border-color: var(--accent-color);
}
[data-theme="dark"] .file-item input[type="checkbox"],
[data-theme="dark"] .select-all input[type="checkbox"] {
  background-color: var(--bg-secondary);
}
.file-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
  font-size: 12px;
  color: var(--text-primary);
}
.empty-list {
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}
.commit-section {
  margin-top: 12px;
}
.commit-input {
  width: 100%;
  padding: 8px;
  border: 1px solid var(--border-input);
  border-radius: 4px;
  font-size: 13px;
  resize: none;
  font-family: inherit;
  box-sizing: border-box;
  background: var(--bg-primary);
  color: var(--text-primary);
}
.commit-input:focus {
  border-color: var(--accent-color);
  outline: none;
}
.commit-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
  justify-content: flex-end;
}
.commit-actions button {
  padding: 6px 16px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
}
.commit-actions button:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.commit-actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.commit-actions .icon-btn {
  padding: 6px;
  width: 28px;
  height: 28px;
}
.commit-btn {
  background: var(--accent-color) !important;
  color: #fff !important;
  border-color: var(--accent-color) !important;
}
.commit-btn:hover:not(:disabled) {
  background: var(--accent-hover) !important;
}
.ai-btn {
  margin-right: auto;
  color: var(--purple-color);
  border-color: var(--purple-color);
}
.ai-btn:hover:not(:disabled) {
  background: rgba(114, 46, 209, 0.1);
}
.diff-placeholder {
  color: var(--text-muted);
  text-align: center;
  margin-top: 40px;
  font-size: 13px;
}
</style>
