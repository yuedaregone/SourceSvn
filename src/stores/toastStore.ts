import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface Toast {
  id: number
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([])
  let nextId = 1

  function addToast(message: string, type: Toast['type'] = 'info', duration = 3000) {
    const id = nextId++
    toasts.value.push({ id, message, type })
    
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id)
      }, duration)
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

  return {
    toasts,
    addToast,
    success,
    error,
    warning,
    info,
    removeToast,
    clearAll,
  }
})