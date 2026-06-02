import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface Toast {
  id: number
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
  expanded?: boolean
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([])
  let nextId = 1

  function addToast(message: string, type: Toast['type'] = 'info', duration?: number) {
    const id = nextId++
    toasts.value.push({ id, message, type })

    // error 类型不自动关闭，其他类型默认 3000ms 后关闭
    const effectiveDuration = duration !== undefined ? duration : type === 'error' ? 0 : 3000

    if (effectiveDuration > 0) {
      setTimeout(() => {
        removeToast(id)
      }, effectiveDuration)
    }
  }

  function success(message: string, duration?: number) {
    addToast(message, 'success', duration)
  }

  function error(message: string, duration?: number) {
    addToast(message, 'error', duration)
  }

  function warning(message: string, duration?: number) {
    addToast(message, 'warning', duration)
  }

  function info(message: string, duration?: number) {
    addToast(message, 'info', duration)
  }

  function removeToast(id: number) {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index !== -1) {
      toasts.value.splice(index, 1)
    }
  }

  function clearAll() {
    toasts.value = []
  }

  function toggleExpand(id: number) {
    const t = toasts.value.find(t => t.id === id)
    if (t) t.expanded = !t.expanded
  }

  return {
    toasts,
    addToast,
    success,
    error,
    warning,
    info,
    removeToast,
    clearAll,
    toggleExpand,
  }
})