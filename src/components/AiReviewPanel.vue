<template>
  <div v-if="visible" class="ai-panel">
    <div class="panel-header">
      <span>AI 代码审查</span>
      <button @click="$emit('close')">×</button>
    </div>
    <div class="panel-content">
      <div v-if="loading" class="loading">AI 正在审查变更...</div>
      <div v-else class="review-text">{{ content }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean
  content: string
  loading: boolean
}>()

defineEmits<{
  close: []
}>()
</script>

<style scoped>
.ai-panel {
  width: 350px;
  border-left: 1px solid #e8e8e8;
  display: flex;
  flex-direction: column;
  background: #fafafa;
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-bottom: 1px solid #e8e8e8;
  font-weight: 600;
}
.panel-header button {
  border: none;
  background: transparent;
  font-size: 18px;
  cursor: pointer;
}
.panel-content {
  flex: 1;
  overflow: auto;
  padding: 12px;
  font-size: 13px;
  line-height: 1.6;
}
.loading {
  color: #1890ff;
}
</style>
