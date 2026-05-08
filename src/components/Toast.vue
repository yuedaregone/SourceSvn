<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="toast"
        :class="toast.type"
      >
        <component :is="getIcon(toast.type)" :size="18" class="toast-icon" />
        <div class="toast-body">
          <span class="toast-message">{{ toast.message }}</span>
          <span v-if="toast.expanded" class="toast-detail">{{ toast.message }}</span>
        </div>
        <button class="toast-close" @click="toastStore.removeToast(toast.id)">
          <X :size="14" />
        </button>
      </div>
    </TransitionGroup>
    <button v-if="toastStore.toasts.length > 1" class="clear-all-btn" @click="toastStore.clearAll()">
      <Trash2 :size="13" />
      <span>{{ t('toast.clearAll') }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { X, CheckCircle, AlertCircle, Info, AlertTriangle, Trash2 } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import { t } from '../locales'

const toastStore = useToastStore()

const ICON_MAP: Record<string, typeof Info> = {
  success: CheckCircle,
  error: AlertCircle,
  warning: AlertTriangle,
  info: Info,
}

function getIcon(type: string) {
  return ICON_MAP[type] || Info
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: var(--space-5);
  right: var(--space-5);
  z-index: var(--z-toast);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-lg);
  min-width: 280px;
  max-width: 420px;
  box-shadow: var(--shadow-lg);
  animation: slideInRight 0.3s ease;
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
}

.toast.success {
  border-color: var(--color-success);
}

.toast.error {
  border-color: var(--color-danger);
}

.toast.warning {
  border-color: var(--color-warning);
}

.toast.info {
  border-color: var(--color-accent);
}

.toast-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.toast.success .toast-icon {
  color: var(--color-success);
}

.toast.error .toast-icon {
  color: var(--color-danger);
}

.toast.warning .toast-icon {
  color: var(--color-warning);
}

.toast.info .toast-icon {
  color: var(--color-accent);
}

.toast-body {
  flex: 1;
  min-width: 0;
}

.toast-message {
  font-size: var(--text-base);
  line-height: 1.5;
  display: block;
  word-break: break-all;
}

.toast-detail {
  font-size: var(--text-sm);
  line-height: 1.4;
  margin-top: var(--space-2);
  display: block;
  white-space: pre-wrap;
  max-height: 200px;
  overflow: auto;
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-3);
  font-family: var(--font-mono);
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
}

.toast-close {
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
  padding: var(--space-1);
  border-radius: var(--radius-sm);
  opacity: 0.5;
  transition: opacity var(--transition-fast);
  margin-top: -2px;
  color: inherit;
}

.toast-close:hover {
  opacity: 1;
}

.clear-all-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-1);
  margin-top: var(--space-1);
  padding: var(--space-1) var(--space-3);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.clear-all-btn:hover {
  color: var(--color-danger);
  border-color: var(--color-danger);
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateX(100%);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>
