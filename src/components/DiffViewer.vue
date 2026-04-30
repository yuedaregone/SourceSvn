<template>
  <div v-if="visible" class="diff-overlay" @click.self="$emit('close')">
    <div class="diff-modal">
      <div class="diff-header">
        <span>文件: {{ filePath }}</span>
        <div class="diff-mode-toggle">
          <button :class="{ active: mode === 'unified' }" @click="mode = 'unified'">统一视图</button>
          <button :class="{ active: mode === 'side_by_side' }" @click="mode = 'side_by_side'">并排视图</button>
        </div>
        <button class="close-btn" @click="$emit('close')">×</button>
      </div>
      <div class="diff-content">
        <pre>{{ diffText }}</pre>
      </div>
      <div class="diff-footer">
        <button @click="copyDiff">复制差异</button>
        <button @click="$emit('aiReview', diffText)">AI 审查</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  visible: boolean
  filePath: string
  diffText: string
}>()

defineEmits<{
  close: []
  aiReview: [diff: string]
}>()

const mode = ref<'unified' | 'side_by_side'>('unified')

async function copyDiff() {
  try {
    await navigator.clipboard.writeText(props.diffText)
  } catch (e) {
    console.error('复制失败:', e)
  }
}
</script>

<style scoped>
.diff-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}
.diff-modal {
  background: #fff;
  border-radius: 8px;
  width: 80%;
  height: 80%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.diff-header {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e8e8e8;
  gap: 12px;
}
.diff-mode-toggle {
  display: flex;
  gap: 4px;
}
.diff-mode-toggle button {
  padding: 4px 8px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.diff-mode-toggle button.active {
  background: #1890ff;
  color: #fff;
  border-color: #1890ff;
}
.close-btn {
  margin-left: auto;
  border: none;
  background: transparent;
  font-size: 20px;
  cursor: pointer;
}
.diff-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
}
.diff-content pre {
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
}
.diff-footer {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid #e8e8e8;
}
.diff-footer button {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
</style>
