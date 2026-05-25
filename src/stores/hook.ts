import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface HookHandlerConfig {
  name: string
  script_path: string
  enabled: boolean
}

export interface HooksConfig {
  enabled: boolean
  handlers: HookHandlerConfig[]
}

export const useHookStore = defineStore('hook', () => {
  const config = ref<HooksConfig>({
    enabled: true,
    handlers: []
  })

  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadConfig() {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke('hook_load_config')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveConfig() {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_save_config', { config: config.value })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addHandler(handler: HookHandlerConfig) {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_add_handler', { handler })
      await loadConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function removeHandler(name: string) {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_remove_handler', { name })
      await loadConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function updateHandler(name: string, handler: HookHandlerConfig) {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_update_handler', { name, handler })
      await loadConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return {
    config,
    loading,
    error,
    loadConfig,
    saveConfig,
    addHandler,
    removeHandler,
    updateHandler
  }
})
