<template>
  <div class="local-changes-view">
    <div class="left-panel" :style="{ width: leftPanelWidth + 'px' }">
      <div class="file-list" :class="{ loading: props.loading }">
        <div v-if="props.loading" class="loading-overlay">
          <div class="spinner" />
        </div>
        <div class="file-list-header">
          <label class="select-all">
            <input type="checkbox" :checked="allSelected" @change="toggleAll" class="checkbox" />
            <span>{{ t('common.selectAll') }}</span>
          </label>
          <span class="selected-count">{{ t('localChanges.selectedOf', { selected: selectedPaths.size, total: props.localChanges.length }) }}</span>
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
            class="checkbox"
          />
          <span class="status-badge" :class="file.status">{{ statusLabel(file.status) }}</span>
          <span class="file-path" :class="{ dir: file.isDirectory }">{{ displayPath(file.path) }}</span>
        </div>
        <div v-if="!props.loading && props.localChanges.length === 0" class="empty-list">
          <FileIcon :size="24" />
          <span>{{ t('common.noLocalChanges') }}</span>
        </div>
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
        <div v-else class="diff-placeholder">
          <GitCompare :size="32" />
          <span>{{ t('common.clickToViewDiff') }}</span>
        </div>
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
            <button @click.stop="showHistory = !showHistory" class="action-btn icon-btn" :title="t('localChanges.recentCommits')">
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
          <button @click="generateAiMessage" :disabled="aiLoading || selectedPaths.size === 0" class="action-btn ai-btn" :title="t('localChanges.aiGenerate')">
            <Sparkles :size="16" />
          </button>
          <button @click="cancelCommit" class="action-btn icon-btn" :title="t('common.cancel')">
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
import { Sparkles, X, Send, RotateCcw, Plus, Trash2, ExternalLink, FolderOpen, Copy, CheckSquare, Square, History, File as FileIcon, GitCompare, GitMerge } from 'lucide-vue-next'
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

const leftPanelWidth = ref(350)
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
  leftPanelWidth.value = Math.max(180, Math.min(500, startWidth.value + delta))
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
  const isConflicted = file.status === 'conflicted'

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
      label: t('contextMenu.acceptTheirs'),
      icon: GitMerge,
      disabled: !isConflicted,
      action: async () => {
        try {
          await invoke('svn_resolve', { path: props.repoPath, paths, accept: 'theirs' })
          emit('refreshLocalChanges')
          toast.success(t('contextMenu.acceptTheirs'))
        } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.acceptMine'),
      icon: GitMerge,
      disabled: !isConflicted,
      action: async () => {
        try {
          await invoke('svn_resolve', { path: props.repoPath, paths, accept: 'mine' })
          emit('refreshLocalChanges')
          toast.success(t('contextMenu.acceptMine'))
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
  gap: 0;
}

.left-panel {
  flex: none;
  display: flex;
  flex-direction: column;
  min-width: 180px;
  max-width: 500px;
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
  transition: background var(--transition-fast);
}

.drag-bar:hover {
  background: var(--color-bg-hover);
}

.drag-handle {
  width: 2px;
  height: 40px;
  background: var(--color-border-light);
  border-radius: var(--radius-full);
  transition: background var(--transition-fast);
}

.drag-bar:hover .drag-handle,
.drag-bar:active .drag-handle {
  background: var(--color-border);
}

.right-panel {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--color-bg-primary);
  display: flex;
  flex-direction: column;
}

.file-list-header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-base);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
  position: sticky;
  top: 0;
  z-index: 1;
}

.select-all {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.selected-count {
  color: var(--color-text-muted);
  font-size: var(--text-sm);
}

.file-list {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  min-height: 0;
  background: var(--color-bg-primary);
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
  background: var(--color-bg-primary);
  opacity: 0.9;
  z-index: 10;
}

.file-item {
  display: flex;
  align-items: center;
  padding: var(--space-2) var(--space-3);
  gap: var(--space-2);
  cursor: pointer;
  font-size: var(--text-base);
  border-bottom: 1px solid var(--color-border-light);
  user-select: none;
  transition: background var(--transition-fast);
}

.file-item:hover {
  background: var(--color-bg-hover);
}

.file-item.selected {
  background: var(--color-bg-active);
}

.file-item.checked:not(.selected) {
  background: var(--color-accent-muted);
}

.file-item input[type="checkbox"],
.select-all input[type="checkbox"] {
  appearance: none;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--color-border-input);
  border-radius: var(--radius-sm);
  background: var(--color-bg-primary);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.file-item input[type="checkbox"]:checked,
.select-all input[type="checkbox"]:checked {
  background: var(--color-accent);
  border-color: var(--color-accent);
}

.file-item input[type="checkbox"]:checked::after,
.select-all input[type="checkbox"]:checked::after {
  content: '';
  width: 5px;
  height: 9px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
  position: absolute;
  left: 4px;
  top: 1px;
}

.file-item input[type="checkbox"]:hover,
.select-all input[type="checkbox"]:hover {
  border-color: var(--color-accent);
}

.file-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--color-text-primary);
}

.empty-list {
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--space-8) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

/* 右侧提交区 */
.commit-section {
  padding: var(--space-3);
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

.commit-input {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-input);
  border-radius: var(--radius-md);
  font-size: var(--text-base);
  resize: none;
  font-family: var(--font-ui);
  box-sizing: border-box;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  transition: all var(--transition-fast);
}

.commit-input:focus {
  border-color: var(--color-accent);
  outline: none;
  box-shadow: 0 0 0 3px var(--color-accent-muted);
}

.commit-actions {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-2);
  justify-content: flex-end;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
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

.action-btn:hover:not(:disabled) {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.icon-btn {
  padding: 0;
  width: 32px;
}

.commit-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 32px;
  padding: 0 var(--space-4);
  background: var(--color-accent);
  color: var(--color-text-inverse);
  border: 1px solid var(--color-accent);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-sm);
  font-weight: 500;
  transition: all var(--transition-fast);
}

.commit-btn:hover:not(:disabled) {
  background: var(--color-accent-hover);
  border-color: var(--color-accent-hover);
  box-shadow: var(--shadow-glow);
}

.commit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ai-btn {
  color: var(--color-purple);
  border-color: var(--color-purple-muted);
}

.ai-btn:hover:not(:disabled) {
  background: var(--color-purple-muted);
  border-color: var(--color-purple);
}

.history-wrapper {
  position: relative;
  margin-right: auto;
}

.history-dropdown {
  position: absolute;
  bottom: calc(100% + var(--space-2));
  left: 0;
  min-width: 260px;
  max-width: 400px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  z-index: var(--z-dropdown);
  overflow: hidden;
  animation: slideUp 0.15s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.history-item {
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-sm);
  color: var(--color-text-primary);
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  border-bottom: 1px solid var(--color-border-light);
  transition: background var(--transition-fast);
}

.history-item:last-child {
  border-bottom: none;
}

.history-item:hover {
  background: var(--color-bg-hover);
}

.history-empty {
  padding: var(--space-3);
  font-size: var(--text-sm);
  color: var(--color-text-muted);
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
  color: var(--color-text-muted);
  text-align: center;
  margin-top: var(--space-10);
  font-size: var(--text-base);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

/* 二进制文件大小对比 */
.binary-diff {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--space-4);
}

.binary-diff-title {
  font-size: var(--text-base);
  color: var(--color-text-secondary);
}

.binary-diff-info {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-xl);
  font-family: var(--font-mono);
}

.binary-size {
  color: var(--color-text-primary);
}

.binary-arrow {
  color: var(--color-text-muted);
}

.binary-delta {
  font-size: var(--text-md);
  font-weight: 600;
}

.binary-delta.positive {
  color: var(--color-danger);
}

.binary-delta.negative {
  color: var(--color-success);
}
</style>
