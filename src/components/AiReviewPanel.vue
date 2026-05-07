<template>
  <Teleport to="body">
    <Transition name="slide">
      <div v-if="visible" class="ai-panel-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
        <div class="ai-panel">
          <div class="panel-header">
            <div class="panel-header-left">
              <Sparkles :size="16" class="panel-icon" />
              <span class="panel-title">{{ t('aiReviewPanel.title') }}</span>
            </div>
            <button class="btn btn-icon btn-ghost" @click="$emit('close')">
              <X :size="16" />
            </button>
          </div>
          <div class="panel-content">
            <div v-if="loading && !content" class="loading-state">
              <div class="spinner" />
              <span>{{ t('aiReviewPanel.reviewing') }}</span>
            </div>
            <div v-else class="review-text">{{ content }}</div>
            <div v-if="loading && content" class="streaming-indicator">
              <div class="pulse-dot" />
              <span>{{ t('aiReviewPanel.streaming') }}</span>
            </div>
          </div>
          <div class="panel-footer" v-if="content && !loading">
            <button @click="copyContent" class="btn btn-secondary">
              <Copy :size="14" />
              <span>{{ t('aiReviewPanel.copyResult') }}</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useToastStore } from '../stores/toastStore'
import { t } from '../locales'
import { X, Sparkles, Copy } from 'lucide-vue-next'

const props = defineProps<{
  visible: boolean
  content: string
  loading: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const overlayMousedown = ref(false)
function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  emit('close')
}

async function copyContent() {
  try {
    await navigator.clipboard.writeText(props.content)
    useToastStore().success(t('aiReviewPanel.copySuccess'))
  } catch {
    useToastStore().error(t('aiReviewPanel.copyFailed'))
  }
}
</script>

<style scoped>
.ai-panel-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  z-index: var(--z-overlay);
  display: flex;
  justify-content: flex-end;
}

.ai-panel {
  width: 400px;
  max-width: 90vw;
  height: 100%;
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--color-border);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4);
  border-bottom: 1px solid var(--color-border);
}

.panel-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.panel-icon {
  color: var(--color-accent);
}

.panel-title {
  font-weight: 600;
  font-size: var(--text-md);
  color: var(--color-text-primary);
}

.panel-content {
  flex: 1;
  overflow: auto;
  padding: var(--space-4);
  font-size: var(--text-base);
  line-height: 1.7;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--color-text-primary);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-10);
  color: var(--color-text-secondary);
}

.streaming-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-2);
  font-size: var(--text-sm);
  color: var(--color-accent);
}

.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-accent);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(0.8);
  }
}

.panel-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--color-border);
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
