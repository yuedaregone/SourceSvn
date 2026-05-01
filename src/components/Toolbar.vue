<template>
  <div class="toolbar">
    <button @click="$emit('pull')" :disabled="loading">拉取</button>
    <button @click="$emit('commit')" :disabled="loading">提交</button>
    <button @click="$emit('refresh')" :disabled="loading">刷新</button>
    <span v-if="loading" class="loading-indicator">处理中...</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  loading: boolean
}>()

defineEmits<{
  pull: []
  commit: []
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
