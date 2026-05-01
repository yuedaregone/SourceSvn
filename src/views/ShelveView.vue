<template>
  <div class="shelve-view">
    <div class="shelve-header">
      <button @click="showSaveDialog = true" class="primary-btn">保存当前修改</button>
      <button @click="refresh" class="action-btn" :disabled="store.loading">刷新</button>
      <div class="header-right">
        <button
          @click="bulkApply"
          :disabled="selectedNames.size === 0"
          class="action-btn"
        >
          应用 ({{ selectedNames.size }})
        </button>
        <button
          @click="bulkDelete"
          :disabled="selectedNames.size === 0"
          class="action-btn danger"
        >
          删除 ({{ selectedNames.size }})
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
            <th class="col-name">名称</th>
            <th class="col-date">日期</th>
            <th class="col-actions">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="shelve in store.shelves"
            :key="shelve.name"
            :class="{ selected: selectedNames.has(shelve.name) }"
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
              <button @click="applyShelve(shelve.name)" class="table-btn">应用</button>
              <button @click="deleteShelve(shelve.name)" class="table-btn danger">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="store.shelves.length === 0" class="empty">暂无 Shelve</div>
    </div>
    <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
    <div v-if="showSaveDialog" class="dialog-overlay" @click.self="showSaveDialog = false">
      <div class="dialog">
        <h3>保存 Shelve</h3>
        <input
          v-model="shelveName"
          placeholder="名称"
          class="dialog-input"
          @keyup.enter="saveShelve"
          ref="nameInput"
        />
        <div class="dialog-actions">
          <button @click="showSaveDialog = false" class="cancel-btn">取消</button>
          <button @click="saveShelve" :disabled="!shelveName.trim()" class="primary-btn">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  store: {
    repoPath: string
    shelves: { name: string; date: string }[]
    loading: boolean
    refreshShelves: () => Promise<void>
  }
}>()

const showSaveDialog = ref(false)
const shelveName = ref('')
const errorMessage = ref('')
const selectedNames = ref(new Set<string>())
const nameInput = ref<HTMLInputElement | null>(null)

const allSelected = computed(
  () =>
    props.store.shelves.length > 0 &&
    props.store.shelves.every((s) => selectedNames.value.has(s.name)),
)

watch(showSaveDialog, (v) => {
  if (v) nextTick(() => nameInput.value?.focus())
})

function toggleAll() {
  if (allSelected.value) {
    selectedNames.value = new Set()
  } else {
    selectedNames.value = new Set(props.store.shelves.map((s) => s.name))
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
      path: props.store.repoPath,
      name: shelveName.value.trim(),
    })
    showSaveDialog.value = false
    shelveName.value = ''
    await props.store.refreshShelves()
  } catch (e) {
    errorMessage.value = `保存失败: ${e}`
  }
}

async function applyShelve(name: string) {
  errorMessage.value = ''
  try {
    await invoke('shelve_apply', { path: props.store.repoPath, name })
    await props.store.refreshShelves()
  } catch (e) {
    errorMessage.value = `应用失败: ${e}`
  }
}

async function deleteShelve(name: string) {
  if (!confirm(`确定要删除 '${name}' 吗？`)) return
  errorMessage.value = ''
  try {
    await invoke('shelve_delete', { path: props.store.repoPath, name })
    selectedNames.value.delete(name)
    await props.store.refreshShelves()
  } catch (e) {
    errorMessage.value = `删除失败: ${e}`
  }
}

async function bulkApply() {
  errorMessage.value = ''
  for (const name of selectedNames.value) {
    try {
      await invoke('shelve_apply', { path: props.store.repoPath, name })
    } catch (e) {
      errorMessage.value = `应用 '${name}' 失败: ${e}`
      break
    }
  }
  selectedNames.value = new Set()
  await props.store.refreshShelves()
}

async function bulkDelete() {
  if (!confirm(`确定要删除选中的 ${selectedNames.value.size} 个 Shelve 吗？`)) return
  errorMessage.value = ''
  for (const name of selectedNames.value) {
    try {
      await invoke('shelve_delete', { path: props.store.repoPath, name })
    } catch (e) {
      errorMessage.value = `删除 '${name}' 失败: ${e}`
      break
    }
  }
  selectedNames.value = new Set()
  await props.store.refreshShelves()
}

function refresh() {
  props.store.refreshShelves()
}

onMounted(() => {
  props.store.refreshShelves()
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
  border: 1px solid #1890ff;
  background: #1890ff;
  color: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.primary-btn:hover:not(:disabled) {
  background: #40a9ff;
}
.primary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn {
  padding: 6px 14px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.action-btn:hover:not(:disabled) {
  border-color: #1890ff;
  color: #1890ff;
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.action-btn.danger {
  color: #ff4d4f;
  border-color: #ff4d4f;
}
.action-btn.danger:hover:not(:disabled) {
  background: #fff2f0;
}
.shelve-list {
  flex: 1;
  overflow: auto;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
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
  border-bottom: 1px solid #f0f0f0;
}
th {
  background: #fafafa;
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
  background: #f5f5f5;
}
tr.selected {
  background: #e6f7ff;
}
.table-btn {
  padding: 3px 10px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  margin-right: 4px;
}
.table-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
}
.table-btn.danger {
  color: #ff4d4f;
  border-color: #ff4d4f;
}
.table-btn.danger:hover {
  background: #fff2f0;
}
.empty {
  color: #999;
  text-align: center;
  padding: 24px 0;
}
.error-message {
  margin-top: 8px;
  padding: 6px 8px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  border-radius: 4px;
  color: #ff4d4f;
  font-size: 12px;
}
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.dialog {
  background: #fff;
  padding: 20px;
  border-radius: 8px;
  min-width: 320px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.15);
}
.dialog h3 {
  margin: 0 0 12px;
  font-size: 15px;
}
.dialog-input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}
.dialog-input:focus {
  border-color: #1890ff;
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
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
</style>
