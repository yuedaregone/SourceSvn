<template>
  <div v-if="visible" class="dialog-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <div class="modal-header-left">
          <div class="modal-icon">
            <Download :size="16" />
          </div>
          <span class="modal-title">{{ t('common.pullResult') }}</span>
          <span class="modal-rev" v-if="result">r{{ result.revision }}</span>
        </div>
        <button class="close-btn" @click="$emit('close')">&times;</button>
      </div>

      <div class="stats-bar" v-if="result">
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
        <table class="file-table">
          <thead>
            <tr>
              <th style="width: 44px"></th>
              <th style="width: 44px">{{ t('common.status') }}</th>
              <th>{{ t('common.filePath') }}</th>
              <th style="width: 100px">{{ t('common.modifier') }}</th>
              <th style="width: 72px; text-align: center">{{ t('common.actions') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="file in sortedFiles"
              :key="file.path"
              :class="{ 'row-conflict': file.status === 'C' }"
            >
              <td />
              <td>
                <span class="status-badge" :class="statusClass(file.status)">
                  {{ file.status }}
                </span>
              </td>
              <td class="file-path" :title="file.path">{{ file.path }}</td>
              <td class="file-author">{{ file.author }}</td>
              <td style="text-align: center">
                <button
                  v-if="file.status === 'C'"
                  class="diff-btn resolve-btn"
                  @click="handleResolve(file)"
                >
                  {{ t('common.resolve') }}
                </button>
                <button
                  v-else
                  class="diff-btn"
                  @click="handleViewDiff(file)"
                >
                  {{ t('common.view') }}
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="modal-footer">
        <button class="btn btn-primary" @click="$emit('close')">{{ t('common.close') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Download } from 'lucide-vue-next'
import type { UpdateResult, UpdateFileItem } from '../types/svn'
import { useToastStore } from '../stores/toastStore'
import { t } from '../locales'

const props = defineProps<{
  visible: boolean
  result: UpdateResult | null
}>()

const STATUS_ORDER: Record<string, number> = { C: 0, M: 1, A: 2, U: 2 }

const sortedFiles = computed(() => {
  if (!props.result) return []
  return [...props.result.files].sort(
    (a, b) => (STATUS_ORDER[a.status] ?? 9) - (STATUS_ORDER[b.status] ?? 9),
  )
})

const conflictCount = computed(
  () => props.result?.files.filter((f) => f.status === 'C').length ?? 0,
)
const mergedCount = computed(
  () => props.result?.files.filter((f) => f.status === 'M').length ?? 0,
)
const updatedCount = computed(
  () => props.result?.files.filter((f) => f.status === 'A' || f.status === 'U').length ?? 0,
)

function statusClass(status: string) {
  if (status === 'C') return 'conflict'
  if (status === 'M') return 'merged'
  return 'added'
}

function handleResolve(_file: UpdateFileItem) {
  // TODO: open third-party merge tool
  useToastStore().info(t('common.resolve') + ': ' + _file.path)
}

function handleViewDiff(_file: UpdateFileItem) {
  // TODO: open DiffViewer (deferred to avoid nested modal complexity)
  useToastStore().info(t('common.view') + ': ' + _file.path)
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--overlay-bg, rgba(0, 0, 0, 0.5));
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}

.modal {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 12px;
  width: 560px;
  max-height: 460px;
  box-shadow: var(--shadow, 0 8px 32px rgba(0, 0, 0, 0.4));
  display: flex;
  flex-direction: column;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.96); }
  to { opacity: 1; transform: scale(1); }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px 12px;
  border-bottom: 1px solid var(--border);
}

.modal-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--accent, #7c6ff7);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.modal-title {
  font-size: 15px;
  font-weight: 600;
}

.modal-rev {
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--bg-tertiary, #33334d);
  padding: 2px 8px;
  border-radius: 4px;
}

.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-muted, #707090);
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  transition: all 0.15s;
}
.close-btn:hover {
  background: var(--bg-tertiary, #33334d);
  color: var(--text-primary);
}

.stats-bar {
  display: flex;
  gap: 16px;
  padding: 10px 20px;
  background: var(--bg-secondary, #2a2a3d);
  border-bottom: 1px solid var(--border);
  font-size: 13px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.stat-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.stat-dot.conflict { background: var(--red, #f87171); }
.stat-dot.merged { background: var(--yellow, #fbbf24); }
.stat-dot.added { background: var(--green, #4ade80); }

.stat-label { color: var(--text-secondary); }
.stat-value { font-weight: 600; }
.conflict-value { color: var(--red, #f87171); }
.merged-value { color: var(--yellow, #fbbf24); }
.added-value { color: var(--green, #4ade80); }

.file-table-wrapper {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.file-table-wrapper::-webkit-scrollbar { width: 6px; }
.file-table-wrapper::-webkit-scrollbar-track { background: transparent; }
.file-table-wrapper::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}

.file-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.file-table th {
  position: sticky;
  top: 0;
  background: var(--bg-secondary, #2a2a3d);
  text-align: left;
  padding: 8px 12px;
  font-weight: 500;
  color: var(--text-muted, #707090);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border);
  z-index: 1;
}

.file-table td {
  padding: 9px 12px;
  border-bottom: 1px solid rgba(61, 61, 92, 0.4);
  vertical-align: middle;
}

.file-table tr:hover { background: var(--bg-secondary, #2a2a3d); }

.file-table tr.row-conflict {
  background: rgba(248, 113, 113, 0.08);
}
.file-table tr.row-conflict:hover {
  background: rgba(248, 113, 113, 0.14);
}
.file-table tr.row-conflict td:first-child {
  box-shadow: inset 3px 0 0 var(--red, #f87171);
}

.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 5px;
  font-size: 11px;
  font-weight: 700;
}
.status-badge.conflict {
  background: rgba(248, 113, 113, 0.15);
  color: var(--red, #f87171);
}
.status-badge.merged {
  background: rgba(251, 191, 36, 0.15);
  color: var(--yellow, #fbbf24);
}
.status-badge.added {
  background: rgba(74, 222, 128, 0.15);
  color: var(--green, #4ade80);
}

.file-path {
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', monospace;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
}

.file-author {
  color: var(--text-secondary);
  white-space: nowrap;
}

.diff-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 10px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-secondary);
  border-radius: 5px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}
.diff-btn:hover {
  background: var(--accent, #7c6ff7);
  border-color: var(--accent, #7c6ff7);
  color: white;
}

.resolve-btn {
  border-color: rgba(248, 113, 113, 0.4);
  color: var(--red, #f87171);
}
.resolve-btn:hover {
  background: var(--red, #f87171);
  border-color: var(--red, #f87171);
  color: white;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
  gap: 10px;
}

.btn {
  padding: 7px 18px;
  border-radius: 7px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border);
  transition: all 0.15s;
}

.btn-primary {
  background: var(--accent, #7c6ff7);
  border-color: var(--accent, #7c6ff7);
  color: white;
}
.btn-primary:hover {
  background: var(--accent-hover, #9b90f9);
}
</style>
