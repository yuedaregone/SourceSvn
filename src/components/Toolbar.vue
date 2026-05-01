<template>
  <div class="toolbar">
    <button @click="$emit('pull')" :disabled="loading" class="icon-btn" title="拉取更新">
      <Download :size="16" />
    </button>
    <button @click="$emit('refresh')" :disabled="loading" class="icon-btn" title="刷新">
      <RefreshCw :size="16" />
    </button>
    <span v-if="loading" class="loading-indicator">处理中...</span>
  </div>
</template>

<script setup lang="ts">
import { Download, RefreshCw } from 'lucide-vue-next'

defineProps<{
  loading: boolean
}>()

defineEmits<{
  pull: []
  refresh: []
}>()
</script>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  height: 32px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 8px;
  gap: 4px;
}
.toolbar button {
  padding: 4px 12px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.toolbar button:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.toolbar .icon-btn {
  padding: 4px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.toolbar button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.loading-indicator {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-muted);
}
</style>
