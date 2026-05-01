<template>
  <div class="log-view">
    <div class="filter-bar">
      <select v-model="authorFilter" class="filter-select">
        <option value="">所有作者</option>
        <option v-for="a in authors" :key="a" :value="a">{{ a }}</option>
      </select>
      <input v-model="dateFrom" type="date" class="filter-date" placeholder="开始日期" />
      <span class="date-separator">~</span>
      <input v-model="dateTo" type="date" class="filter-date" placeholder="结束日期" />
      <input v-model="searchText" placeholder="搜索提交信息..." class="search-input" />
      <button @click="refresh" class="refresh-btn" :disabled="store.loading">↻</button>
    </div>
    <div class="log-table">
      <table>
        <thead>
          <tr>
            <th class="col-revision">版本</th>
            <th class="col-author">作者</th>
            <th class="col-date">日期</th>
            <th class="col-message">提交信息</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in pagedEntries"
            :key="entry.revision"
            @click="toggleDetail(entry.revision)"
            :class="{ expanded: expandedRevision === entry.revision }"
          >
            <td class="col-revision">{{ entry.revision }}</td>
            <td class="col-author">{{ entry.author }}</td>
            <td class="col-date">{{ formatDate(entry.date) }}</td>
            <td class="col-message">{{ entry.message }}</td>
          </tr>
          <tr v-if="filteredEntries.length === 0">
            <td colspan="4" class="empty-row">暂无日志记录</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="pagination">
      <button :disabled="currentPage <= 1" @click="currentPage--">&lt;上一页</button>
      <span class="page-info">第 {{ currentPage }}/{{ totalPages }} 页</span>
      <button :disabled="currentPage >= totalPages" @click="currentPage++">下一页&gt;</button>
    </div>
    <div v-if="expandedRevision" class="detail-panel">
      <h4>版本 {{ expandedRevision }} 详细信息</h4>
      <p class="detail-message">{{ expandedEntry?.message }}</p>
      <div v-if="expandedEntry?.changedPaths?.length" class="changed-paths">
        <h5>变更文件:</h5>
        <div v-for="cp in expandedEntry.changedPaths" :key="cp.path" class="changed-path">
          <span class="action" :class="actionClass(cp.action)">{{ cp.action }}</span>
          <span class="path-text">{{ cp.path }}</span>
        </div>
      </div>
      <div class="detail-actions">
        <button @click="$emit('viewDiff', expandedRevision)" class="action-btn">查看差异</button>
        <button @click="$emit('aiReview', expandedRevision)" class="action-btn ai">AI 审查</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import type { LogEntry } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    logEntries: LogEntry[]
    loading: boolean
    refreshLog: () => Promise<void>
  }
}>()

defineEmits<{
  viewDiff: [revision: number]
  aiReview: [revision: number]
}>()

const searchText = ref('')
const authorFilter = ref('')
const dateFrom = ref('')
const dateTo = ref('')
const expandedRevision = ref<number | null>(null)
const currentPage = ref(1)
const pageSize = 50

const authors = computed(() => {
  const set = new Set(props.store.logEntries.map((e) => e.author))
  return Array.from(set).sort()
})

const filteredEntries = computed(() => {
  let entries = props.store.logEntries
  if (authorFilter.value) {
    entries = entries.filter((e) => e.author === authorFilter.value)
  }
  if (dateFrom.value) {
    const from = new Date(dateFrom.value)
    entries = entries.filter((e) => new Date(e.date) >= from)
  }
  if (dateTo.value) {
    const to = new Date(dateTo.value)
    to.setHours(23, 59, 59, 999)
    entries = entries.filter((e) => new Date(e.date) <= to)
  }
  if (searchText.value) {
    const text = searchText.value.toLowerCase()
    entries = entries.filter(
      (e) =>
        e.message.toLowerCase().includes(text) ||
        e.author.toLowerCase().includes(text),
    )
  }
  return entries
})

const totalPages = computed(() => Math.max(1, Math.ceil(filteredEntries.value.length / pageSize)))

const pagedEntries = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return filteredEntries.value.slice(start, start + pageSize)
})

const expandedEntry = computed(() => {
  if (!expandedRevision.value) return null
  return props.store.logEntries.find((e) => e.revision === expandedRevision.value)
})

watch([authorFilter, dateFrom, dateTo, searchText], () => {
  currentPage.value = 1
})

function toggleDetail(revision: number) {
  expandedRevision.value = expandedRevision.value === revision ? null : revision
}

function formatDate(dateStr: string) {
  try {
    const d = new Date(dateStr)
    const m = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    const h = String(d.getHours()).padStart(2, '0')
    const min = String(d.getMinutes()).padStart(2, '0')
    return `${m}-${day} ${h}:${min}`
  } catch {
    return dateStr
  }
}

function actionClass(action: string) {
  if (action === 'A') return 'added'
  if (action === 'D') return 'deleted'
  if (action === 'R') return 'replaced'
  return 'modified'
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
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}
.filter-select,
.filter-date {
  padding: 5px 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  background: #fff;
}
.filter-select {
  min-width: 100px;
}
.filter-date {
  width: 140px;
}
.date-separator {
  color: #999;
  font-size: 13px;
}
.search-input {
  flex: 1;
  min-width: 150px;
  padding: 5px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
}
.refresh-btn {
  padding: 5px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}
.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.log-table {
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
  z-index: 1;
}
.col-revision {
  width: 80px;
}
.col-author {
  width: 100px;
}
.col-date {
  width: 120px;
}
.col-message {
  min-width: 200px;
}
tr:hover {
  background: #f5f5f5;
  cursor: pointer;
}
tr.expanded {
  background: #e6f7ff;
}
.empty-row {
  text-align: center;
  color: #999;
  padding: 24px 0;
}
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid #f0f0f0;
  font-size: 13px;
}
.pagination button {
  padding: 4px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.pagination button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.page-info {
  color: #666;
}
.detail-panel {
  border-top: 1px solid #e8e8e8;
  padding: 12px;
  background: #fafafa;
  border-radius: 0 0 4px 4px;
}
.detail-panel h4 {
  margin: 0 0 8px;
  font-size: 14px;
}
.detail-message {
  color: #333;
  margin: 0 0 8px;
}
.changed-paths {
  margin-top: 8px;
}
.changed-paths h5 {
  margin: 0 0 4px;
  font-size: 13px;
  color: #666;
}
.changed-path {
  font-size: 12px;
  padding: 2px 0;
  font-family: monospace;
  display: flex;
  align-items: center;
  gap: 6px;
}
.action {
  display: inline-block;
  width: 18px;
  text-align: center;
  font-weight: bold;
  font-size: 11px;
  border-radius: 2px;
  color: #fff;
  padding: 1px 0;
}
.action.modified {
  background: #faad14;
}
.action.added {
  background: #52c41a;
}
.action.deleted {
  background: #ff4d4f;
}
.action.replaced {
  background: #722ed1;
}
.path-text {
  color: #333;
}
.detail-actions {
  margin-top: 12px;
  display: flex;
  gap: 8px;
}
.action-btn {
  padding: 4px 14px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.action-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
}
.action-btn.ai {
  border-color: #722ed1;
  color: #722ed1;
}
.action-btn.ai:hover {
  background: #f9f0ff;
}
</style>
