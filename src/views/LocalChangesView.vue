<template>
  <div class="local-changes-view">
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
        <div class="view-controls">
          <button
            v-if="viewMode === 'tree'"
            class="view-btn"
            :title="t('localChanges.expandAll')"
            @click="expandAll"
          >
            <ChevronDown :size="14" />
          </button>
          <button
            v-if="viewMode === 'tree'"
            class="view-btn"
            :title="t('localChanges.collapseAll')"
            @click="collapseAll"
          >
            <ChevronRight :size="14" />
          </button>
          <button
            class="view-btn"
            :class="{ active: viewMode === 'flat' }"
            :title="t('localChanges.flatView')"
            @click="viewMode = 'flat'"
          >
            <List :size="14" />
          </button>
          <button
            class="view-btn"
            :class="{ active: viewMode === 'tree' }"
            :title="t('localChanges.treeView')"
            @click="viewMode = 'tree'"
          >
            <FolderTree :size="14" />
          </button>
        </div>
      </div>
      <!-- 扁平列表视图 -->
      <template v-if="viewMode === 'flat'">
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
      </template>
      <!-- 树形视图 -->
      <template v-else>
        <div
          v-for="node in flatTreeNodes"
          :key="node.path"
          class="file-item tree-item"
          :class="{
            selected: selectedFile === node.path,
            checked: selectedPaths.has(node.path),
            'tree-dir': node.isDirectory,
          }"
          :style="{ paddingLeft: `${12 + node.level * 16}px` }"
          @click="handleTreeClick(node, $event)"
          @contextmenu.prevent="openContextMenuForTree(node, $event)"
        >
          <span v-if="node.hasChildren" class="tree-toggle" @click.stop="toggleFolder(node.path)">
            <ChevronRight v-if="!node.isExpanded" :size="14" />
            <ChevronDown v-else :size="14" />
          </span>
          <span v-else class="tree-toggle-placeholder" />
          <input
            v-if="!node.isDirectory"
            type="checkbox"
            :checked="selectedPaths.has(node.path)"
            @click.stop="toggleFile(node.path)"
            :disabled="props.loading"
            class="checkbox"
          />
          <span v-if="node.isDirectory" class="folder-icon">
            <FolderOpen :size="14" />
          </span>
          <span v-if="node.status" class="status-badge" :class="node.status">{{ statusLabel(node.status) }}</span>
          <span class="file-path" :class="{ dir: node.isDirectory }">{{ node.name }}</span>
        </div>
      </template>
      <div v-if="!props.loading && props.localChanges.length === 0" class="empty-list">
        <FileIcon :size="24" />
        <span>{{ t('common.noLocalChanges') }}</span>
      </div>
    </div>
    <div class="drag-bar" @mousedown="onDragStart" @touchstart="onDragStart">
      <div class="drag-handle"></div>
    </div>
    <div class="commit-section" :style="{ height: commitSectionHeight + 'px' }" @click="showHistory = false">
      <textarea
        v-model="commitMessage"
        :placeholder="t('localChanges.commitMessage')"
        class="textarea textarea--no-resize commit-textarea"
      ></textarea>
      <div class="commit-actions">
        <div class="history-wrapper">
          <button @click.stop="showHistory = !showHistory" class="action-btn icon-btn" :title="t('localChanges.recentCommits')">
            <History :size="16" />
          </button>
          <button @click="generateAiMessage" :disabled="aiLoading || selectedPaths.size === 0" class="action-btn ai-btn" :title="t('localChanges.aiGenerate')">
            <Sparkles :size="16" />
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
        <button @click="cancelCommit" class="action-btn icon-btn" :title="t('common.cancel')">
          <X :size="16" />
        </button>
        <button
          @click="showShelveDialog = true"
          :disabled="selectedPaths.size === 0"
          class="action-btn shelve-btn"
          :title="t('shelveView.saveCurrentChanges')"
        >
          <Package :size="16" />
        </button>
        <button @click="submitCommit" :disabled="!canCommit" class="commit-btn" :title="t('common.submit')">
          <Send :size="16" />
        </button>
      </div>
    </div>
    <!-- 储藏命名对话框 -->
    <div v-if="showShelveDialog" class="dialog-overlay" @click.self="showShelveDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <h3 class="dialog-title">{{ t('common.saveShelve') }}</h3>
          <button class="btn btn-icon btn-ghost" @click="showShelveDialog = false">
            <X :size="16" />
          </button>
        </div>
        <div class="dialog-body">
          <p class="dialog-hint">{{ t('shelveView.shelveFileCount', { count: selectedPaths.size }) }}</p>
          <input
            v-model="shelveName"
            :placeholder="t('common.shelveName')"
            class="input"
            @keyup.enter="saveShelve"
            ref="shelveNameInput"
          />
        </div>
        <div class="dialog-footer">
          <button @click="showShelveDialog = false" class="btn btn-secondary">{{ t('common.cancel') }}</button>
          <button @click="saveShelve" :disabled="!shelveName.trim() || shelveLoading" class="btn btn-primary">
            {{ t('common.save') }}
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
    <FileLogModal
      :visible="showFileLog"
      :file-path="fileLogPath"
      :repo-path="props.repoPath"
      @close="showFileLog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, shallowRef, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Sparkles, X, Send, RotateCcw, Plus, Trash2, ExternalLink, FolderOpen, Copy, CheckSquare, Square, History, File as FileIcon, GitMerge, ScrollText, List, FolderTree, ChevronRight, ChevronDown, Package } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import { useConfigStore } from '../stores/configStore'
import { useDiffStore } from '../stores/diffStore'
import type { FileStatus, FileStatusType, DiffTarget } from '../types/svn'
import type { MenuItem } from '../components/ContextMenu.vue'
import ContextMenu from '../components/ContextMenu.vue'
import FileLogModal from '../components/FileLogModal.vue'
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
const aiLoading = ref(false)
const toast = useToastStore()
const diffStore = useDiffStore()
const showFileLog = ref(false)
const fileLogPath = ref('')
const configStore = useConfigStore()

// 储藏对话框
const showShelveDialog = ref(false)
const shelveName = ref('')
const shelveLoading = ref(false)
const shelveNameInput = ref<HTMLInputElement | null>(null)

watch(showShelveDialog, (v) => {
  if (v) nextTick(() => shelveNameInput.value?.focus())
})

// 树形视图相关
type ViewMode = 'flat' | 'tree'
const viewMode = ref<ViewMode>('flat')
const expandedFolders = shallowRef(new Set<string>())

const commitSectionHeight = ref(loadCommitHeight())
const isDragging = ref(false)
const startY = ref(0)
const startHeight = ref(0)

interface TreeNode {
  name: string
  path: string
  isDirectory: boolean
  status?: FileStatusType
  level: number
  children?: TreeNode[]
}

// 构建树形结构
function buildTree(files: FileStatus[]): TreeNode[] {
  const result: TreeNode[] = []

  for (const file of files) {
    const relativePath = displayPath(file.path)
    const parts = relativePath.split(/[\/\\]/)
    let currentPath = ''
    let parentChildren = result

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      const isLast = i === parts.length - 1
      currentPath = currentPath ? `${currentPath}/${part}` : part

      if (isLast && !file.isDirectory) {
        // 文件节点
        parentChildren.push({
          name: part,
          path: file.path,
          isDirectory: false,
          status: file.status,
          level: i,
        })
      } else {
        // 文件夹节点
        let existing = parentChildren.find(n => n.name === part && n.isDirectory)
        if (!existing) {
          existing = {
            name: part,
            path: currentPath,
            isDirectory: true,
            level: i,
            children: [],
          }
          parentChildren.push(existing)
        }
        if (existing.children) {
          parentChildren = existing.children
        }
      }
    }
  }

  return result
}

// 扁平化树（只包含可见节点）
interface FlatNode {
  name: string
  path: string
  isDirectory: boolean
  status?: FileStatusType
  level: number
  hasChildren: boolean
  isExpanded?: boolean
}

const treeData = computed(() => buildTree(props.localChanges))

const flatTreeNodes = computed<FlatNode[]>(() => {
  const result: FlatNode[] = []

  function traverse(nodes: TreeNode[]) {
    for (const node of nodes) {
      const hasChildren = node.isDirectory && node.children && node.children.length > 0
      const isExpanded = expandedFolders.value.has(node.path)

      result.push({
        name: node.name,
        path: node.path,
        isDirectory: node.isDirectory,
        status: node.status,
        level: node.level,
        hasChildren: !!hasChildren,
        isExpanded: hasChildren ? isExpanded : undefined,
      })

      if (hasChildren && isExpanded && node.children) {
        traverse(node.children)
      }
    }
  }

  traverse(treeData.value)
  return result
})

function toggleFolder(path: string) {
  const next = new Set(expandedFolders.value)
  if (next.has(path)) {
    next.delete(path)
  } else {
    next.add(path)
  }
  expandedFolders.value = next
}

function expandAll() {
  const paths = new Set<string>()
  function collect(nodes: TreeNode[]) {
    for (const node of nodes) {
      if (node.isDirectory && node.children && node.children.length > 0) {
        paths.add(node.path)
        collect(node.children)
      }
    }
  }
  collect(treeData.value)
  expandedFolders.value = paths
}

function collapseAll() {
  expandedFolders.value = new Set()
}

function loadCommitHeight(): number {
  const saved = localStorage.getItem('localChanges.commitHeight')
  if (saved) {
    const h = parseInt(saved, 10)
    if (!isNaN(h) && h >= 80 && h <= 300) return h
  }
  return 120
}

function saveCommitHeight(height: number) {
  localStorage.setItem('localChanges.commitHeight', String(height))
}

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

function onDragStart(e: MouseEvent | TouchEvent) {
  isDragging.value = true
  startY.value = 'touches' in e ? e.touches[0].clientY : e.clientY
  startHeight.value = commitSectionHeight.value
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
  document.addEventListener('touchmove', onDragMove)
  document.addEventListener('touchend', onDragEnd)
}

function onDragMove(e: MouseEvent | TouchEvent) {
  if (!isDragging.value) return
  const currentY = 'touches' in e ? e.touches[0].clientY : e.clientY
  const delta = startY.value - currentY
  commitSectionHeight.value = Math.max(80, Math.min(300, startHeight.value + delta))
}

function onDragEnd() {
  isDragging.value = false
  saveCommitHeight(commitSectionHeight.value)
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
  locked: 'L',
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
    if (file.isDirectory) {
      toast.info(t('localChanges.directoryNoDiff'))
      return
    }
    try {
      if (file.status === 'unversioned' || file.status === 'added') {
        // 新文件：只有新内容
        const newContent = await invoke<string>('read_local_file', {
          repoPath: props.repoPath,
          filePath: file.path,
        })
        diffStore.openWithContent(file.path, undefined, newContent)
      } else if (file.status === 'deleted' || file.status === 'missing') {
        // 删除的文件：只有旧内容
        const oldContent = await invoke<string>('svn_cat_in_dir', {
          repoPath: props.repoPath,
          filePath: file.path,
          revision: 'BASE',
        })
        diffStore.openWithContent(file.path, oldContent, undefined)
      } else {
        // 修改的文件：获取旧版本和新版本完整内容
        const [oldContent, newContent] = await Promise.all([
          invoke<string>('svn_cat_in_dir', {
            repoPath: props.repoPath,
            filePath: file.path,
            revision: 'BASE',
          }),
          invoke<string>('read_local_file', { repoPath: props.repoPath, filePath: file.path }),
        ])
        diffStore.openWithContent(file.path, oldContent, newContent)
      }
    } catch (e) {
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
}

async function saveShelve() {
  if (!shelveName.value.trim()) return
  shelveLoading.value = true
  try {
    await invoke('shelve_save', {
      path: props.repoPath,
      name: shelveName.value.trim(),
      files: Array.from(selectedPaths.value),
    })
    showShelveDialog.value = false
    shelveName.value = ''
    toast.success(t('shelveView.saveSuccess'))
    emit('refreshLocalChanges')
  } catch (e) {
    toast.error(String(e))
  } finally {
    shelveLoading.value = false
  }
}

function openContextMenu(e: MouseEvent, file: FileStatus) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, file }
  selectedFile.value = file.path
}

function openContextMenuForTree(node: FlatNode, e: MouseEvent) {
  const file = props.localChanges.find(f => f.path === node.path)
  if (file) {
    openContextMenu(e, file)
  } else if (node.isDirectory) {
    // 文件夹的右键菜单
    ctxMenu.value = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      file: { path: node.path, status: 'modified', isDirectory: true },
    }
    selectedFile.value = node.path
  }
}

async function handleTreeClick(node: FlatNode, event: MouseEvent) {
  if (node.isDirectory) {
    toggleFolder(node.path)
    selectedFile.value = node.path
    return
  }

  // 文件点击 - 复用原有逻辑
  const file = props.localChanges.find(f => f.path === node.path)
  if (file) {
    const index = props.localChanges.indexOf(file)
    await selectFile(file, index, event)
  }
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
      label: t('contextMenu.showLog'),
      icon: ScrollText,
      action: () => {
        fileLogPath.value = file.path
        showFileLog.value = true
      },
    },
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
        navigator.clipboard.writeText(displayPath(file.path))
        toast.success(t('contextMenu.copySuccess'))
      },
    },
    {
      label: t('contextMenu.copyAbsPath'),
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
  flex-direction: column;
  height: 100%;
  gap: 0;
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

.drag-bar {
  height: 6px;
  cursor: ns-resize;
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

.commit-section {
  flex: none;
  padding: var(--space-2);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg-primary);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
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

.view-controls {
  display: flex;
  gap: var(--space-1);
  margin-left: auto;
}

.view-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-primary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.view-btn:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.view-btn.active {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: var(--color-text-inverse);
}

/* 树形视图样式 */
.tree-item {
  font-size: var(--text-sm);
}

.tree-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  cursor: pointer;
  color: var(--color-text-muted);
  flex-shrink: 0;
  transition: color var(--transition-fast);
}

.tree-toggle:hover {
  color: var(--color-text-primary);
}

.tree-toggle-placeholder {
  width: 16px;
  flex-shrink: 0;
}

.folder-icon {
  display: inline-flex;
  align-items: center;
  color: var(--color-accent);
  flex-shrink: 0;
}

/* 提交区 */
.commit-textarea {
  flex: 1;
  min-height: 0;
  resize: none;
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
  display: flex;
  gap: var(--space-2);
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

/* 储藏按钮 */
.shelve-btn {
  color: var(--color-warning);
  border-color: var(--color-warning-muted);
}

.shelve-btn:hover:not(:disabled) {
  background: var(--color-warning-muted);
  border-color: var(--color-warning);
}

/* 储藏命名对话框 */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  animation: fadeInOverlay 0.2s ease;
}

@keyframes fadeInOverlay {
  from { opacity: 0; }
  to { opacity: 1; }
}

.dialog {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  width: 400px;
  box-shadow: var(--shadow-xl);
  animation: scaleInDialog 0.2s ease;
}

@keyframes scaleInDialog {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--color-border);
}

.dialog-title {
  margin: 0;
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-text-primary);
}

.dialog-body {
  padding: var(--space-4) var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.dialog-hint {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  margin: 0;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--color-border);
}
</style>
