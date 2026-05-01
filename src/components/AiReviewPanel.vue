<template>
  <Teleport to="body">
    <Transition name="slide">
      <div v-if="visible" class="ai-panel-overlay" @click.self="$emit('close')">
        <div class="ai-panel">
          <div class="panel-header">
            <span class="panel-title">AI 代码审查</span>
            <button class="close-btn" @click="$emit('close')">&times;</button>
          </div>
          <div class="panel-content">
            <div v-if="loading && !content" class="loading-state">
              <div class="spinner"></div>
              <span>AI 正在审查变更...</span>
            </div>
            <div v-else class="review-text">{{ content }}</div>
            <div v-if="loading && content" class="streaming-indicator">正在接收...</div>
          </div>
          <div class="panel-footer" v-if="content && !loading">
            <button @click="copyContent" class="copy-btn">复制结果</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
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

async function copyContent() {
  try {
    const el = document.querySelector('.review-text')
    if (el) {
      await navigator.clipboard.writeText(el.textContent || '')
    }
  } catch (e) {
    console.error('复制失败:', e)
  }
}
</script>

<style scoped>
.ai-panel-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--overlay-bg);
  z-index: 250;
  display: flex;
  justify-content: flex-end;
}
.ai-panel {
  width: 400px;
  max-width: 90vw;
  height: 100%;
  background: var(--bg-primary);
  box-shadow: -4px 0 24px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-color);
}
.panel-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
}
.close-btn {
  border: none;
  background: transparent;
  font-size: 22px;
  cursor: pointer;
  color: var(--text-muted);
  line-height: 1;
}
.close-btn:hover {
  color: var(--text-primary);
}
.panel-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
  font-size: 13px;
  line-height: 1.7;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-primary);
}
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  margin-top: 60px;
  color: var(--text-secondary);
}
.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.streaming-indicator {
  margin-top: 8px;
  font-size: 12px;
  color: var(--accent-color);
}
.panel-footer {
  padding: 10px 16px;
  border-top: 1px solid var(--border-color);
}
.copy-btn {
  padding: 5px 14px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.copy-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.slide-enter-active,
.slide-leave-active {
  transition: opacity 0.25s ease;
}
.slide-enter-active .ai-panel,
.slide-leave-active .ai-panel {
  transition: transform 0.25s ease;
}
.slide-enter-from .ai-panel,
.slide-leave-to .ai-panel {
  transform: translateX(100%);
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
}
.slide-enter-to,
.slide-leave-from {
  opacity: 1;
}
</style>
