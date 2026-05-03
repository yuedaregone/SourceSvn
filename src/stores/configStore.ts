import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types/config'

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig | null>(null)

  const theme = computed(() => config.value?.appearance.theme ?? 'light')
  const fontSize = computed(() => config.value?.appearance.uiFontSize ?? 14)
  const codeFont = computed(() => config.value?.appearance.codeFontFamily ?? 'monospace')

  async function loadConfig() {
    try {
      config.value = await invoke<AppConfig>('get_config')
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  async function saveConfig() {
    if (!config.value) return
    try {
      await invoke('set_config', { conf: config.value })
    } catch (e) {
      console.error('Failed to save config:', e)
    }
  }

  function updateTheme(theme: string) {
    if (!config.value) return
    config.value.appearance.theme = theme
    saveConfig()
  }

  return {
    config,
    theme,
    fontSize,
    codeFont,
    loadConfig,
    saveConfig,
    updateTheme,
  }
})
