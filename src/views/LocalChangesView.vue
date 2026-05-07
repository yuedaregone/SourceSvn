<template>
  <div class="local-changes-view">
    <div class="left-panel" :style="{ width: leftPanelWidth + 'px' }">
      <div class="file-list-header">
        <label class="select-all">
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          <span>{{ t('common.selectAll') }}</span>
        </label>
        <span class="selected-count">{{ t('localChanges.selectedOf', { selected: selectedPaths.size, total: props.localChanges.length }) }}</span>
      </div>
      <div class="file-list" :class="{ loading: props.loading }">
        <div v-if="props.loading" class="loading-overlay">
          <RefreshCw :size="24" class="spin" />
        </div>
        <div
          v-for="(file, index) in props.localChanges"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedFile === file.path, checked: selectedPaths.has(file.path) }"
          @click="selectFile(file, index, $event)"
          @contextmenu.prevent="openContextMenu($event, file)"
        >
          <input
            type="checkbox"
            :checked="selectedPaths.has(file.path)"
            @click.stop="toggleFile(file.path)"
            :disabled="props.loading"
          />
          <span class="status-badge" :class="file.status">{{ statusLabel(file.status) }}</span>
          <span class="file-path" :class="{ dir: file.isDirectory }">{{ displayPath(file.path) }}</span>
        </div>
        <div v-if="!props.loading && props.localChanges.length === 0" class="empty-list">{{ t('common.noLocalChanges') }}</div>
      </div>
    </div>
    <div class="drag-bar" @mousedown="onDragStart" @touchstart="onDragStart">
      <div class="drag-handle"></div>
    </div>
    <div class="right-panel">
      <div class="diff-area">
        <!-- 二进制文件大小对比 -->
        <div v-if="binarySizeDiff" class="binary-diff">
          <div class="binary-diff-title">{{ t('localChanges.binarySizeDiff') }}</div>
          <div class="binary-diff-info">
            <span class="binary-size">{{ formatSize(binarySizeDiff.baseSize) }}</span>
            <span class="binary-arrow">→</span>
            <span class="binary-size">{{ formatSize(binarySizeDiff.currentSize) }}</span>
            <span class="binary-delta" :class="{ positive: delta > 0, negative: delta < 0 }">
              ({{ delta > 0 ? '+' : '' }}{{ formatSize(delta) }})
            </span>
          </div>
        </div>
        <!-- 代码 diff -->
        <InlineDiff v-else-if="diffContent" :diff-text="diffContent" :empty-hint="t('common.clickToViewDiff')" />
        <!-- 空状态 -->
        <div v-else class="diff-placeholder">{{ t('common.clickToViewDiff') }}</div>
      </div>
      <div class="commit-section" @click="showHistory = false">
        <textarea
          v-model="commitMessage"
          :placeholder="t('localChanges.commitMessage')"
          rows="2"
          class="commit-input"
        ></textarea>
        <div class="commit-actions">
          <div class="history-wrapper">
            <button @click.stop="showHistory = !showHistory" class="history-btn icon-btn" :title="t('localChanges.recentCommits')">
              <History :size="16" />
            </button>
            <div v-if="showHistory" class="history-dropdown" @click.stop>
              <div
                v-for="(msg, i) in recentMessages"
                :key="i"
                class="history-item"
                @click="selectMessage(msg)"
              >{{ msg }}</div>
              <div v-if="recentMessages.length === 0" class="history-empty">{{ t('localChanges.noRecentCommits') }}</div>
            </div>
          </div>
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
import { Sparkles, RefreshCw, X, Send, RotateCcw, Plus, Trash2, ExternalLink, FolderOpen, Copy, CheckSquare, Square, History } from 'lucide-vue-next'
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
  commitHistory?: string[]
}>()

const emit = defineEmits<{
  refresh: []
  refreshLocalChanges: []
  addCommitMessage: [msg: string]
}>()

const selectedPaths = ref(new Set<string>())
const selectedFile = ref('')
const lastClickIndex = ref(-1)
const commitMessage = ref('')
const showHistory = ref(false)
const diffContent = ref('')
const aiLoading = ref(false)
const binarySizeDiff = ref<{ baseSize: number; currentSize: number } | null>(null)
const toast = useToastStore()
const configStore = useConfigStore()

const leftPanelWidth = ref(320)
const isDragging = ref(false)
const startX = ref(0)
const startWidth = ref(0)

const recentMessages = computed(() =>
  (props.commitHistory ?? []).slice(0, 5),
)

function selectMessage(msg: string) {
  commitMessage.value = msg
  showHistory.value = false
}

const allSelected = computed(
  () =>
    props.localChanges.length > 0 &&
    props.localChanges.every((f) => selectedPaths.value.has(f.path)),
)

function toggleAll() {
  if (allSelected.value) {
    selectedPaths.value = new Set()
  } else {
    selectedPaths.value = new Set(props.localChanges.map((f) => f.path))
  }
}

const delta = computed(() => {
  if (!binarySizeDiff.value) return 0
  return binarySizeDiff.value.currentSize - binarySizeDiff.value.baseSize
})

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const abs = Math.abs(bytes)
  if (abs < 1024) return bytes + ' B'
  if (abs < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

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

const STATUS_LABELS: Record<string, string> = {
  modified: 'M',
  added: 'A',
  deleted: 'D',
  unversioned: '?',
  missing: '!',
  conflicted: 'C',
}

function statusLabel(status: string): string {
  return STATUS_LABELS[status] ?? status[0].toUpperCase()
}

function displayPath(filePath: string): string {
  const normRepo = props.repoPath.replace(/[\/\\]+$/, '').replace(/\//g, '\\')
  const normFile = filePath.replace(/\//g, '\\')
  if (normFile.toLowerCase().startsWith(normRepo.toLowerCase() + '\\')) {
    return normFile.slice(normRepo.length + 1)
  }
  return filePath
}

const canCommit = computed(
  () => selectedPaths.value.size > 0 && commitMessage.value.trim().length > 0,
)

function toggleFile(path: string) {
  const next = new Set(selectedPaths.value)
  if (next.has(path)) {
    next.delete(path)
  } else {
    next.add(path)
  }
  selectedPaths.value = next
}

async function selectFile(file: FileStatus, index: number, event: MouseEvent) {
  // Shift+Click: 范围选择
  if (event.shiftKey && lastClickIndex.value >= 0) {
    const start = Math.min(lastClickIndex.value, index)
    const end = Math.max(lastClickIndex.value, index)
    const next = new Set(selectedPaths.value)
    for (let i = start; i <= end; i++) {
      next.add(props.localChanges[i].path)
    }
    selectedPaths.value = next
    selectedFile.value = file.path
  }
  // Ctrl+Click: 切换单个选中，不切换 diff
  else if (event.ctrlKey || event.metaKey) {
    toggleFile(file.path)
  }
  // 普通点击: 切换选中 + 查看 diff
  else {
    selectedPaths.value = new Set([file.path])
    selectedFile.value = file.path
  }

  lastClickIndex.value = index

  // 查看 diff（仅 Ctrl+Click 不切换 diff）
  if (!event.ctrlKey && !event.metaKey) {
    binarySizeDiff.value = null
    if (file.isDirectory) {
      diffContent.value = ''
      toast.info(t('localChanges.directoryNoDiff'))
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
        const diff = await invoke<string>('svn_diff', {
          path: props.repoPath,
          target,
        })
        // 检测二进制文件
        if (diff.includes('Binary files') || diff.includes('不能以文本形式显示')) {
          diffContent.value = ''
          try {
            const [baseSize, currentSize] = await invoke<[number, number]>('file_size_diff', {
              repoPath: props.repoPath,
              filePath: file.path,
            })
            binarySizeDiff.value = { baseSize, currentSize }
          } catch {
            binarySizeDiff.value = null
          }
        } else {
          diffContent.value = diff
        }
      }
    } catch (e) {
      diffContent.value = ''
      binarySizeDiff.value = null
      toast.error(String(e), 0)
    }
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
  const msg = commitMessage.value
  try {
    await invoke('svn_commit', {
      path: props.repoPath,
      message: msg,
      files: Array.from(selectedPaths.value),
    })
    emit('addCommitMessage', msg)
    commitMessage.value = ''
    selectedPaths.value = new Set()
    selectedFile.value = ''
    diffContent.value = ''
    binarySizeDiff.value = null
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
  binarySizeDiff.value = null
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

  const paths = selectedPaths.value.has(file.path)
    ? Array.from(selectedPaths.value)
    : [file.path]

  return [
    {
      label: t('contextMenu.revert'),
      icon: RotateCcw,
      disabled: !isModified,
      action: async () => {
        if (configStore.config?.behavior.confirmBeforeRevert && !confirm(t('contextMenu.revertConfirm'))) return
        try {
          await invoke('svn_revert', { path: props.repoPath, paths })
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
          await invoke('svn_add', { path: props.repoPath, paths })
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
          await invoke('svn_delete', { path: props.repoPath, paths, keepLocal: false })
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
          await invoke('delete_files_from_disk', { path: props.repoPath, paths })
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
  font-size: 12px;
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
  padding: 4px 8px;
  gap: 6px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid var(--border-light);
  user-select: none;
}
.file-item:hover {
  background: var(--bg-hover);
}
.file-item.selected {
  background: var(--bg-active);
}
.file-item.checked:not(.selected) {
  background: var(--bg-hover);
}
.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 3px;
  font-size: 10px;
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
/* 右侧提交区 */
.commit-section {
  padding: 8px;
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}
.commit-input {
  width: 100%;
  padding: 6px 8px;
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
  margin-top: 6px;
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
  color: var(--purple-color);
  border-color: var(--purple-color);
}
.ai-btn:hover:not(:disabled) {
  background: rgba(114, 46, 209, 0.1);
}
.history-wrapper {
  position: relative;
  margin-right: auto;
}
.history-btn {
  color: var(--text-secondary);
}
.history-btn:hover {
  color: var(--accent-color);
  border-color: var(--accent-color);
}
.history-dropdown {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 4px;
  min-width: 260px;
  max-width: 400px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: var(--shadow);
  z-index: 10;
  overflow: hidden;
}
.history-item {
  padding: 6px 10px;
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  border-bottom: 1px solid var(--border-light);
}
.history-item:last-child {
  border-bottom: none;
}
.history-item:hover {
  background: var(--bg-hover);
}
.history-empty {
  padding: 10px;
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
}
/* 右侧 diff 区域 */
.diff-area {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.diff-placeholder {
  color: var(--text-muted);
  text-align: center;
  margin-top: 40px;
  font-size: 13px;
}
/* 二进制文件大小对比 */
.binary-diff {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
}
.binary-diff-title {
  font-size: 13px;
  color: var(--text-secondary);
}
.binary-diff-info {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 18px;
  font-family: monospace;
}
.binary-size {
  color: var(--text-primary);
}
.binary-arrow {
  color: var(--text-muted);
}
.binary-delta {
  font-size: 14px;
  font-weight: 600;
}
.binary-delta.positive { color: var(--danger-color); }
.binary-delta.negative { color: var(--success-color); }
</style>
