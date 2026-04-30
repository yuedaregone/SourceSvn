<template>
  <div class="log-view">
    <div class="filter-bar">
      <input v-model="searchText" placeholder="搜索提交信息..." class="search-input" />
      <button @click="refresh" class="refresh-btn">↻</button>
    </div>
    <div class="log-table">
      <table>
        <thead>
          <tr>
            <th>版本</th>
            <th>作者</th>
            <th>日期</th>
            <th>提交信息</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in filteredEntries"
            :key="entry.revision"
            @click="toggleDetail(entry.revision)"
            :class="{ expanded: expandedRevision === entry.revision }"
          >
            <td>{{ entry.revision }}</td>
            <td>{{ entry.author }}</td>
            <td>{{ formatDate(entry.date) }}</td>
            <td>{{ entry.message }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-if="expandedRevision" class="detail-panel">
      <h4>版本 {{ expandedRevision }} 详细信息</h4>
      <p>{{ expandedEntry?.message }}</p>
      <div v-if="expandedEntry?.changedPaths" class="changed-paths">
        <h5>变更文件:</h5>
        <div v-for="cp in expandedEntry.changedPaths" :key="cp.path" class="changed-path">
          <span class="action">{{ cp.action }}</span>
          <span>{{ cp.path }}</span>
        </div>
      </div>
      <div class="detail-actions">
        <button @click="$emit('viewDiff', expandedRevision)">查看差异</button>
        <button @click="$emit('aiReview', expandedRevision)">AI 审查</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { LogEntry } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    logEntries: LogEntry[]
    refreshLog: () => Promise<void>
  }
}>()

defineEmits<{
  viewDiff: [revision: number]
  aiReview: [revision: number]
}>()

const searchText = ref('')
const expandedRevision = ref<number | null>(null)

const filteredEntries = computed(() => {
  if (!searchText.value) return props.store.logEntries
  const text = searchText.value.toLowerCase()
  return props.store.logEntries.filter(
    (e) =>
      e.message.toLowerCase().includes(text) ||
      e.author.toLowerCase().includes(text),
  )
})

const expandedEntry = computed(() => {
  if (!expandedRevision.value) return null
  return props.store.logEntries.find((e) => e.revision === expandedRevision.value)
})

function toggleDetail(revision: number) {
  expandedRevision.value = expandedRevision.value === revision ? null : revision
}

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN')
  } catch {
    return dateStr
  }
}

function refresh() {
  props.store.refreshLog()
}

onMounted(() => {
  if (props.store.logEntries.length === 0) {
    refresh()
  }
})
</script>

<style scoped>
.log-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.filter-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}
.search-input {
  flex: 1;
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
}
.refresh-btn {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
}
.log-table {
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
  position: sticky;
  top: 0;
}
tr:hover {
  background: #f5f5f5;
  cursor: pointer;
}
tr.expanded {
  background: #e6f7ff;
}
.detail-panel {
  border-top: 1px solid #e8e8e8;
  padding: 12px;
  background: #fafafa;
}
.changed-paths {
  margin-top: 8px;
}
.changed-path {
  font-size: 12px;
  padding: 2px 0;
  font-family: monospace;
}
.action {
  display: inline-block;
  width: 20px;
  font-weight: bold;
  color: #1890ff;
}
.detail-actions {
  margin-top: 12px;
  display: flex;
  gap: 8px;
}
.detail-actions button {
  padding: 4px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
</style>
