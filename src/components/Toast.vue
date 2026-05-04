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
  top: 20px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 6px;
  min-width: 280px;
  max-width: 420px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  animation: slideIn 0.3s ease-out;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.toast.success {
  border-color: color-mix(in srgb, var(--success-color) 25%, var(--border-color));
}

.toast.error {
  border-color: color-mix(in srgb, var(--danger-color) 25%, var(--border-color));
  color: var(--danger-color);
}

.toast.warning {
  border-color: color-mix(in srgb, var(--warning-color) 25%, var(--border-color));
}

.toast.info {
  border-color: color-mix(in srgb, var(--accent-color) 25%, var(--border-color));
}

.toast-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.toast.success .toast-icon { color: var(--text-secondary); }
.toast.error .toast-icon   { color: var(--danger-color); }
.toast.warning .toast-icon { color: var(--text-secondary); }
.toast.info .toast-icon    { color: var(--text-secondary); }

.toast-body {
  flex: 1;
  min-width: 0;
}

.toast-message {
  font-size: 13px;
  line-height: 1.5;
  display: block;
  word-break: break-all;
}

.toast-detail {
  font-size: 12px;
  line-height: 1.4;
  margin-top: 8px;
  display: block;
  white-space: pre-wrap;
  max-height: 200px;
  overflow: auto;
  border-radius: 4px;
  padding: 8px 10px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.toast.error .toast-detail {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
}

.toast.warning .toast-detail {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
}

.toast.info .toast-detail {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
}

.toast-close {
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  opacity: 0.45;
  transition: opacity 0.2s;
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
  gap: 4px;
  margin-top: 4px;
  padding: 4px 10px;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
  transition: color 0.2s, border-color 0.2s;
}

.clear-all-btn:hover {
  color: var(--danger-color);
  border-color: var(--danger-color);
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

@keyframes slideIn {
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
