<template>
  <div class="shelve-view">
    <div class="shelve-header">
      <button @click="showSaveDialog = true" class="save-btn">保存当前修改</button>
      <button @click="refresh" class="refresh-btn">刷新</button>
    </div>
    <div class="shelve-list">
      <table>
        <thead>
          <tr>
            <th>名称</th>
            <th>日期</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="shelve in store.shelves" :key="shelve.name">
            <td>{{ shelve.name }}</td>
            <td>{{ formatDate(shelve.date) }}</td>
            <td>
              <button @click="applyShelve(shelve.name)" class="action-btn">应用</button>
              <button @click="deleteShelve(shelve.name)" class="action-btn delete">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="store.shelves.length === 0" class="empty">暂无 Shelve</div>
    </div>
    <div v-if="showSaveDialog" class="dialog-overlay">
      <div class="dialog">
        <h3>保存 Shelve</h3>
        <input v-model="shelveName" placeholder="名称" class="dialog-input" />
        <div class="dialog-actions">
          <button @click="showSaveDialog = false" class="cancel-btn">取消</button>
          <button @click="saveShelve" :disabled="!shelveName.trim()" class="save-btn">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  store: {
    repoPath: string
    shelves: { name: string; date: string }[]
    refreshShelves: () => Promise<void>
  }
}>()

const showSaveDialog = ref(false)
const shelveName = ref('')

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN')
  } catch {
    return dateStr
  }
}

async function saveShelve() {
  if (!shelveName.value.trim()) return
  try {
    await invoke('shelve_save', {
      path: props.store.repoPath,
      name: shelveName.value.trim(),
    })
    showSaveDialog.value = false
    shelveName.value = ''
    await props.store.refreshShelves()
  } catch (e) {
    console.error('Save shelve failed:', e)
  }
}

async function applyShelve(name: string) {
  try {
    await invoke('shelve_apply', {
      path: props.store.repoPath,
      name,
    })
    await props.store.refreshShelves()
  } catch (e) {
    console.error('Apply shelve failed:', e)
  }
}

async function deleteShelve(name: string) {
  if (!confirm(`确定要删除 '${name}' 吗？`)) return
  try {
    await invoke('shelve_delete', {
      path: props.store.repoPath,
      name,
    })
    await props.store.refreshShelves()
  } catch (e) {
    console.error('Delete shelve failed:', e)
  }
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
  gap: 8px;
  margin-bottom: 12px;
}
.shelve-header button {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.save-btn {
  background: #1890ff !important;
  color: #fff !important;
  border-color: #1890ff !important;
}
.shelve-list {
  flex: 1;
  overflow: auto;
}
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
th, td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid #f0f0f0;
}
th {
  background: #fafafa;
  font-weight: 600;
}
.action-btn {
  padding: 4px 8px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  margin-right: 4px;
}
.action-btn.delete {
  color: #ff4d4f;
  border-color: #ff4d4f;
}
.empty {
  color: #999;
  text-align: center;
  margin-top: 40px;
}
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.dialog {
  background: #fff;
  padding: 20px;
  border-radius: 8px;
  min-width: 300px;
}
.dialog h3 {
  margin: 0 0 12px;
}
.dialog-input {
  width: 100%;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}
.cancel-btn {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
}
</style>
