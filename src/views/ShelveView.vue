<template>
  <div class="shelve-view">
    <div class="shelve-header">
      <button @click="refresh" class="btn btn-secondary" :disabled="props.loading" :title="t('common.refresh')">
        <RefreshCw :size="16" />
      </button>
      <div class="header-right">
        <button
          @click="bulkApply"
          :disabled="selectedNames.size === 0"
          class="btn btn-secondary"
          :title="t('shelveView.applySelected')"
        >
          <Check :size="16" />
          <span>{{ t('shelveView.apply') }}</span>
        </button>
        <button
          @click="bulkDelete"
          :disabled="selectedNames.size === 0"
          class="btn btn-danger"
          :title="t('shelveView.deleteSelected')"
        >
          <Trash2 :size="16" />
          <span>{{ t('shelveView.delete') }}</span>
        </button>
      </div>
    </div>
    <div class="shelve-list">
      <table>
        <thead>
          <tr>
            <th class="col-check">
              <input type="checkbox" :checked="allSelected" @change="toggleAll" class="checkbox" />
            </th>
            <th class="col-name">{{ t('common.name') }}</th>
            <th class="col-date">{{ t('common.date') }}</th>
            <th class="col-actions">{{ t('common.actions') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="shelve in props.shelves"
            :key="shelve.name"
            :class="{ selected: selectedNames.has(shelve.name) }"
            @contextmenu.prevent="openContextMenu($event, shelve)"
          >
            <td class="col-check">
              <input
                type="checkbox"
                :checked="selectedNames.has(shelve.name)"
                @change="toggleSelect(shelve.name)"
                class="checkbox"
              />
            </td>
            <td class="col-name">
              <span class="shelve-name">{{ shelve.name }}</span>
            </td>
            <td class="col-date">
              <span class="date-text">{{ formatDate(shelve.date) }}</span>
            </td>
            <td class="col-actions">
              <button @click="applyShelve(shelve.name)" class="btn btn-icon btn-secondary" :title="t('common.apply')">
                <ArrowRight :size="14" />
              </button>
              <button @click="deleteShelve(shelve.name)" class="btn btn-icon btn-danger" :title="t('common.delete')">
                <X :size="14" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="props.shelves.length === 0" class="empty">
        <Package :size="24" />
        <span>{{ t('common.noShelves') }}</span>
      </div>
    </div>
    <div v-if="errorMessage" class="error-message">
      <AlertCircle :size="14" />
      <span>{{ errorMessage }}</span>
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
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { RefreshCw, Check, Trash2, ArrowRight, X, Pencil, Package, AlertCircle } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import type { ShelveInfo } from '../types/svn'
import type { MenuItem } from '../components/ContextMenu.vue'
import ContextMenu from '../components/ContextMenu.vue'
import { t } from '../locales'

const props = defineProps<{
  repoPath: string
  shelves: { name: string; date: string }[]
  loading: boolean
}>()

const emit = defineEmits<{
  refreshShelves: []
  refreshLocalChanges: []
}>()

const errorMessage = ref('')
const selectedNames = ref(new Set<string>())

const allSelected = computed(
  () =>
    props.shelves.length > 0 &&
    props.shelves.every((s) => selectedNames.value.has(s.name)),
)

function toggleAll() {
  if (allSelected.value) {
    selectedNames.value = new Set()
  } else {
    selectedNames.value = new Set(props.shelves.map((s) => s.name))
  }
}

function toggleSelect(name: string) {
  const next = new Set(selectedNames.value)
  if (next.has(name)) next.delete(name)
  else next.add(name)
  selectedNames.value = next
}

function formatDate(dateStr: string) {
  try {
    const d = new Date(dateStr)
    const y = d.getFullYear()
    const m = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    const h = String(d.getHours()).padStart(2, '0')
    const min = String(d.getMinutes()).padStart(2, '0')
    return `${y}-${m}-${day} ${h}:${min}`
  } catch {
    return dateStr
  }
}

// Apply（pop 语义：应用 patch 并删除储藏）
async function applyShelve(name: string) {
  errorMessage.value = ''
  try {
    await invoke('shelve_apply', { path: props.repoPath, name, deleteAfterApply: true })
    const next = new Set(selectedNames.value)
    next.delete(name)
    selectedNames.value = next
    emit('refreshShelves')
    emit('refreshLocalChanges')
  } catch (e) {
    errorMessage.value = t('common.error') + ': ' + e
  }
}

async function deleteShelve(name: string) {
  if (!confirm(t('common.deleteConfirm', { name }))) return
  errorMessage.value = ''
  try {
    await invoke('shelve_delete', { path: props.repoPath, name })
    const next = new Set(selectedNames.value)
    next.delete(name)
    selectedNames.value = next
    emit('refreshShelves')
  } catch (e) {
    errorMessage.value = t('common.error') + ': ' + e
  }
}

async function bulkApply() {
  errorMessage.value = ''
  const names = Array.from(selectedNames.value)
  for (const name of names) {
    try {
      await invoke('shelve_apply', { path: props.repoPath, name, deleteAfterApply: true })
    } catch (e) {
      errorMessage.value = t('common.error') + ': ' + e
      break
    }
  }
  selectedNames.value = new Set()
  emit('refreshShelves')
  emit('refreshLocalChanges')
}

async function bulkDelete() {
  if (!confirm(t('common.bulkDeleteConfirm', { count: selectedNames.value.size }))) return
  errorMessage.value = ''
  for (const name of selectedNames.value) {
    try {
      await invoke('shelve_delete', { path: props.repoPath, name })
    } catch (e) {
      errorMessage.value = t('common.error') + ': ' + e
      break
    }
  }
  selectedNames.value = new Set()
  emit('refreshShelves')
}

function refresh() {
  emit('refreshShelves')
}

const ctxMenu = ref({ visible: false, x: 0, y: 0, shelve: null as ShelveInfo | null })

function openContextMenu(e: MouseEvent, shelve: ShelveInfo) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, shelve }
}

const ctxMenuItems = computed<MenuItem[]>(() => {
  const shelve = ctxMenu.value.shelve
  if (!shelve) return []
  const toast = useToastStore()

  return [
    {
      label: t('contextMenu.applyShelve'),
      icon: ArrowRight,
      action: () => { applyShelve(shelve.name) },
    },
    { divider: true },
    {
      label: t('contextMenu.rename'),
      icon: Pencil,
      action: async () => {
        const newName = prompt(t('contextMenu.renameInput'), shelve.name)
        if (!newName || newName.trim() === shelve.name) return
        try {
          // 直接重命名 patch 文件，不重新生成 diff
          await invoke('shelve_rename', { path: props.repoPath, oldName: shelve.name, newName: newName.trim() })
          emit('refreshShelves')
        } catch (e) { toast.error(String(e)) }
      },
    },
    {
      label: t('common.delete'),
      icon: Trash2,
      action: () => { deleteShelve(shelve.name) },
    },
  ]
})

onMounted(() => {
  emit('refreshShelves')
})
</script>

<style scoped>
.shelve-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.shelve-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.header-right {
  margin-left: auto;
  display: flex;
  gap: var(--space-2);
}

.shelve-list {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-bg-primary);
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-base);
}

th,
td {
  padding: var(--space-3) var(--space-4);
  text-align: left;
  border-bottom: 1px solid var(--color-border-light);
  color: var(--color-text-primary);
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
}

.col-check {
  width: 40px;
  text-align: center;
}

.col-name {
  min-width: 160px;
}

.shelve-name {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.col-date {
  width: 180px;
}

.date-text {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.col-actions {
  width: 100px;
}

tr {
  transition: background var(--transition-fast);
}

tr:hover {
  background: var(--color-bg-hover);
}

tr.selected {
  background: var(--color-bg-active);
}

.empty {
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--space-8) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

.error-message {
  margin-top: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--color-danger-muted);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-md);
  color: var(--color-danger);
  font-size: var(--text-sm);
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
</style>
