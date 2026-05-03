<template>
  <div class="shelve-view">
    <div class="shelve-header">
      <button @click="showSaveDialog = true" class="primary-btn" :title="t('shelveView.saveCurrentChanges')">
        <Save :size="16" />
      </button>
      <button @click="refresh" class="action-btn icon-btn" :disabled="props.loading" :title="t('common.refresh')">
        <RefreshCw :size="16" />
      </button>
      <div class="header-right">
        <button
          @click="bulkApply"
          :disabled="selectedNames.size === 0"
          class="action-btn"
          :title="t('shelveView.applySelected')"
        >
          <Check :size="16" />
        </button>
        <button
          @click="bulkDelete"
          :disabled="selectedNames.size === 0"
          class="action-btn danger"
          :title="t('shelveView.deleteSelected')"
        >
          <Trash2 :size="16" />
        </button>
      </div>
    </div>
    <div class="shelve-list">
      <table>
        <thead>
          <tr>
            <th class="col-check">
              <input type="checkbox" :checked="allSelected" @change="toggleAll" />
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
              />
            </td>
            <td class="col-name">{{ shelve.name }}</td>
            <td class="col-date">{{ formatDate(shelve.date) }}</td>
            <td class="col-actions">
              <button @click="applyShelve(shelve.name)" class="table-btn icon-btn" :title="t('common.apply')">
                <ArrowRight :size="14" />
              </button>
              <button @click="deleteShelve(shelve.name)" class="table-btn danger icon-btn" :title="t('common.delete')">
                <X :size="14" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="props.shelves.length === 0" class="empty">{{ t('common.noShelves') }}</div>
    </div>
    <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
    <div v-if="showSaveDialog" class="dialog-overlay" @click.self="showSaveDialog = false">
      <div class="dialog">
        <h3>{{ t('common.saveShelve') }}</h3>
        <input
          v-model="shelveName"
          :placeholder="t('common.shelveName')"
          class="dialog-input"
          @keyup.enter="saveShelve"
          ref="nameInput"
        />
        <div class="dialog-actions">
          <button @click="showSaveDialog = false" class="cancel-btn">{{ t('common.cancel') }}</button>
          <button @click="saveShelve" :disabled="!shelveName.trim()" class="primary-btn">{{ t('common.save') }}</button>
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
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Save, RefreshCw, Check, Trash2, ArrowRight, X, Pencil } from 'lucide-vue-next'
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
}>()

const showSaveDialog = ref(false)
const shelveName = ref('')
const errorMessage = ref('')
const selectedNames = ref(new Set<string>())
const nameInput = ref<HTMLInputElement | null>(null)

const allSelected = computed(
  () =>
    props.shelves.length > 0 &&
    props.shelves.every((s) => selectedNames.value.has(s.name)),
)

watch(showSaveDialog, (v) => {
  if (v) nextTick(() => nameInput.value?.focus())
})

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

async function saveShelve() {
  if (!shelveName.value.trim()) return
  errorMessage.value = ''
  try {
    await invoke('shelve_save', {
      path: props.repoPath,
      name: shelveName.value.trim(),
    })
    showSaveDialog.value = false
    shelveName.value = ''
    emit('refreshShelves')
  } catch (e) {
    errorMessage.value = t('common.error') + ': ' + e
  }
}

async function applyShelve(name: string) {
  errorMessage.value = ''
  try {
    await invoke('shelve_apply', { path: props.repoPath, name })
    emit('refreshShelves')
  } catch (e) {
    errorMessage.value = t('common.error') + ': ' + e
  }
}

async function deleteShelve(name: string) {
  if (!confirm(t('common.deleteConfirm', { name }))) return
  errorMessage.value = ''
  try {
    await invoke('shelve_delete', { path: props.repoPath, name })
    selectedNames.value.delete(name)
    emit('refreshShelves')
  } catch (e) {
    errorMessage.value = t('common.error') + ': ' + e
  }
}

async function bulkApply() {
  errorMessage.value = ''
  for (const name of selectedNames.value) {
    try {
      await invoke('shelve_apply', { path: props.repoPath, name })
    } catch (e) {
      errorMessage.value = t('common.error') + ': ' + e
      break
    }
  }
  selectedNames.value = new Set()
  emit('refreshShelves')
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
        const newName = prompt(t('contextMenu.revisionInput'), shelve.name)
        if (newName && newName.trim() !== shelve.name) {
          try {
            // Rename is save + delete original
            await invoke('shelve_save', { path: props.repoPath, name: newName.trim() })
            await invoke('shelve_delete', { path: props.repoPath, name: shelve.name })
            emit('refreshShelves')
          } catch (e) { toast.error(String(e)) }
        }
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
  gap: 8px;
  margin-bottom: 12px;
}
.header-right {
  margin-left: auto;
  display: flex;
  gap: 8px;
}
.primary-btn {
  padding: 6px 14px;
  border: 1px solid var(--accent-color);
  background: var(--accent-color);
  color: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.primary-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}
.primary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn {
  padding: 6px 14px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
}
.action-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn.danger {
  color: var(--danger-color);
  border-color: var(--danger-color);
}
.action-btn.danger:hover:not(:disabled) {
  background: var(--bg-hover);
}
.action-btn.icon-btn {
  padding: 6px;
  width: 28px;
  height: 28px;
}
.shelve-list {
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
  color: var(--text-primary);
}
th {
  background: var(--bg-secondary);
  font-weight: 600;
  position: sticky;
  top: 0;
}
.col-check {
  width: 40px;
  text-align: center;
}
.col-name {
  min-width: 160px;
}
.col-date {
  width: 180px;
}
.col-actions {
  width: 120px;
}
tr:hover {
  background: var(--bg-hover);
}
tr.selected {
  background: var(--bg-active);
}
.table-btn {
  padding: 3px 10px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  margin-right: 4px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
}
.table-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.table-btn.danger {
  color: var(--danger-color);
  border-color: var(--danger-color);
}
.table-btn.danger:hover {
  background: var(--bg-hover);
}
.table-btn.icon-btn {
  padding: 3px;
  width: 24px;
  height: 24px;
}
.empty {
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}
.error-message {
  margin-top: 8px;
  padding: 6px 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--danger-color);
  border-radius: 4px;
  color: var(--danger-color);
  font-size: 12px;
}
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.dialog {
  background: var(--bg-primary);
  padding: 20px;
  border-radius: 8px;
  min-width: 320px;
  box-shadow: var(--shadow);
}
.dialog h3 {
  margin: 0 0 12px;
  font-size: 15px;
  color: var(--text-primary);
}
.dialog-input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--border-input);
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
  background: var(--bg-primary);
  color: var(--text-primary);
}
.dialog-input:focus {
  border-color: var(--accent-color);
  outline: none;
}
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}
.cancel-btn {
  padding: 6px 14px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
}
</style>
