<template>
  <div v-if="visible" class="dialog-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
    <div class="modal">
      <div class="modal-header">
        <div class="modal-header-left">
          <div class="modal-icon">
            <Download :size="16" />
          </div>
          <span class="modal-title">{{ t('common.pullResult') }}</span>
          <span class="modal-rev" v-if="result && result.revision">r{{ result.revision }}</span>
        </div>
        <button class="btn btn-icon btn-ghost" :disabled="pulling" @click="$emit('close')">
          <X :size="16" />
        </button>
      </div>

      <div class="stats-bar" v-if="result && result.files.length > 0">
        <div class="stat-item">
          <span class="stat-dot conflict" />
          <span class="stat-label">{{ t('common.conflict') }}</span>
          <span class="stat-value conflict-value">{{ conflictCount }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-dot merged" />
          <span class="stat-label">{{ t('common.merged') }}</span>
          <span class="stat-value merged-value">{{ mergedCount }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-dot added" />
          <span class="stat-label">{{ t('common.updated') }}</span>
          <span class="stat-value added-value">{{ updatedCount }}</span>
        </div>
      </div>

      <div class="file-table-wrapper" v-if="result">
        <table v-if="result.files.length > 0" class="file-table">
          <thead>
            <tr>
              <th style="width: 36px"></th>
              <th style="width: 52px">{{ t('common.status') }}</th>
              <th>{{ t('common.filePath') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="file in sortedFiles"
              :key="file.path"
              :class="{ 'row-conflict': file.status === 'C', 'clickable': file.status !== 'C' }"
              @click="viewDiff(file)"
              @contextmenu.prevent="openContextMenu($event, file)"
            >
              <td />
              <td>
                <span class="status-badge" :class="statusClass(file.status)">
                  {{ file.status }}
                </span>
              </td>
              <td class="file-path" :title="file.path">{{ file.path }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="pulling" class="running-bar">
        <div class="spinner spinner-sm" />
        <span>{{ t('common.pulling') }}</span>
      </div>
      <div class="modal-footer">
        <button class="btn btn-primary" :disabled="pulling" @click="$emit('close')">
          <X :size="14" />
          <span>{{ t('common.close') }}</span>
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
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Download, X, ExternalLink, FolderOpen, Copy, GitMerge } from 'lucide-vue-next'
import type { UpdateResult, UpdateFileItem } from '../types/svn'
import type { MenuItem } from './ContextMenu.vue'
import ContextMenu from './ContextMenu.vue'
import { useToastStore } from '../stores/toastStore'
import { useDiffStore } from '../stores/diffStore'
import { t } from '../locales'

const emit = defineEmits<{ close: []; refresh: [] }>()
const overlayMousedown = ref(false)
function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  if (!props.pulling) emit('close')
}

const props = defineProps<{
  visible: boolean
  result: UpdateResult | null
  pulling: boolean
  repoPath: string
}>()

const toast = useToastStore()
const diffStore = useDiffStore()

// Local mutable copy of files for status updates after resolve
const localFiles = ref<UpdateFileItem[]>([])
watch(() => props.result?.files, (files) => {
  localFiles.value = files ? [...files] : []
}, { immediate: true })

const ctxMenu = ref({ visible: false, x: 0, y: 0, file: null as UpdateFileItem | null })

function openContextMenu(e: MouseEvent, file: UpdateFileItem) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, file }
}

function fullPath(relPath: string): string {
  return `${props.repoPath}\\${relPath.replace(/^\//, '').replace(/\//g, '\\')}`
}

async function viewDiff(file: UpdateFileItem) {
  if (file.status === 'C') return
  try {
    // 获取旧版本（拉取前）和新版本（拉取后，即当前工作副本）完整内容
    const oldRev = props.result?.oldRevision
    const newContent = await invoke<string>('read_local_file', {
      repoPath: props.repoPath,
      filePath: file.path,
    })
    // 新增文件在旧版本中不存在，旧内容为空
    let oldContent = ''
    if (file.status !== 'A' && oldRev) {
      try {
        oldContent = await invoke<string>('svn_cat_in_dir', {
          repoPath: props.repoPath,
          filePath: file.path,
          revision: String(oldRev),
        })
      } catch {
        oldContent = ''
      }
    }
    diffStore.openWithContent(file.path, oldContent, newContent)
  } catch (e) {
    toast.error(String(e))
  }
}

const ctxMenuItems = computed<MenuItem[]>(() => {
  const file = ctxMenu.value.file
  if (!file) return []
  const absPath = fullPath(file.path)
  const isConflict = file.status === 'C'

  return [
    {
      label: t('contextMenu.openWithEditor'),
      icon: ExternalLink,
      action: async () => {
        try { await invoke('open_file_with_default_app', { path: absPath }) } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.showInExplorer'),
      icon: FolderOpen,
      action: async () => {
        try { await invoke('open_in_system', { path: absPath }) } catch (e) { toast.error(String(e)) }
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
    { divider: true },
    {
      label: t('contextMenu.acceptTheirs'),
      icon: GitMerge,
      disabled: !isConflict,
      action: async () => {
        try {
          await invoke('svn_resolve', { path: props.repoPath, paths: [file.path], accept: 'theirs' })
          const f = localFiles.value.find(f => f.path === file.path)
          if (f) f.status = 'M'
          toast.success(t('contextMenu.acceptTheirs'))
          emit('refresh')
        } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('contextMenu.acceptMine'),
      icon: GitMerge,
      disabled: !isConflict,
      action: async () => {
        try {
          await invoke('svn_resolve', { path: props.repoPath, paths: [file.path], accept: 'mine' })
          const f = localFiles.value.find(f => f.path === file.path)
          if (f) f.status = 'M'
          toast.success(t('contextMenu.acceptMine'))
          emit('refresh')
        } catch (e) { toast.error(String(e)) }
      },
    },
  ]
})

const STATUS_ORDER: Record<string, number> = { C: 0, M: 1, A: 2, U: 2 }

const sortedFiles = computed(() => {
  if (!props.result) return []
  return [...localFiles.value].sort((a, b) => {
    const sa = STATUS_ORDER[a.status] ?? 9
    const sb = STATUS_ORDER[b.status] ?? 9
    if (sa !== sb) return sa - sb
    // 同状态内：后接收的排上面（反转原始顺序）
    const ia = localFiles.value.indexOf(a)
    const ib = localFiles.value.indexOf(b)
    return ib - ia
  })
})

const conflictCount = computed(
  () => localFiles.value.filter((f) => f.status === 'C').length ?? 0,
)
const mergedCount = computed(
  () => localFiles.value.filter((f) => f.status === 'M').length ?? 0,
)
const updatedCount = computed(
  () => localFiles.value.filter((f) => f.status === 'A' || f.status === 'U').length ?? 0,
)

function statusClass(status: string) {
  if (status === 'C') return 'conflict'
  if (status === 'M') return 'merged'
  return 'added'
}
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
  width: min(720px, 80vw);
  height: min(640px, 75vh);
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
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
}

.modal-title {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--color-text-primary);
}

.modal-rev {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  background: var(--color-bg-secondary);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
}

.stats-bar {
  display: flex;
  gap: var(--space-4);
  padding: var(--space-3) var(--space-5);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
  font-size: var(--text-base);
}

.stat-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.stat-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.stat-dot.conflict {
  background: var(--color-danger);
}

.stat-dot.merged {
  background: var(--color-warning);
}

.stat-dot.added {
  background: var(--color-success);
}

.stat-label {
  color: var(--color-text-secondary);
}

.stat-value {
  font-weight: 600;
}

.conflict-value {
  color: var(--color-danger);
}

.merged-value {
  color: var(--color-warning);
}

.added-value {
  color: var(--color-success);
}

.file-table-wrapper {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.file-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-base);
}

.file-table th {
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
}

.file-table td {
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--color-border-light);
  vertical-align: middle;
}

.file-table tr {
  transition: background var(--transition-fast);
}

.file-table tr.clickable {
  cursor: pointer;
}

.file-table tr:hover {
  background: var(--color-bg-hover);
}

.file-table tr.row-conflict {
  background: var(--color-danger-muted);
}

.file-table tr.row-conflict:hover {
  background: var(--color-danger-muted);
  opacity: 0.8;
}

.file-table tr.row-conflict td:first-child {
  box-shadow: inset 3px 0 0 var(--color-danger);
}

.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
  font-weight: 700;
}

.status-badge.conflict {
  background: var(--color-danger-muted);
  color: var(--color-danger);
}

.status-badge.merged {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}

.status-badge.added {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.file-path {
  font-family: var(--font-mono);
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
  font-size: var(--text-sm);
}

.running-bar {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-5);
  background: var(--color-bg-secondary);
  border-top: 1px solid var(--color-border);
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--color-border);
  gap: var(--space-2);
}
</style>
