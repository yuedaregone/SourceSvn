import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types/config'

export const useConfigStore = defineStore('config', {
  state: () => ({
    config: null as AppConfig | null,
  }),
  getters: {
    theme: (state) => state.config?.appearance.theme ?? 'light',
    fontSize: (state) => state.config?.appearance.uiFontSize ?? 14,
    codeFont: (state) => state.config?.appearance.codeFontFamily ?? 'monospace',
  },
  actions: {
    async loadConfig() {
      try {
        this.config = await invoke<AppConfig>('get_config')
      } catch (e) {
        console.error('Failed to load config:', e)
      }
    },
    async saveConfig() {
      if (!this.config) return
      try {
        await invoke('set_config', { conf: this.config })
      } catch (e) {
        console.error('Failed to save config:', e)
      }
    },
    updateTheme(theme: string) {
      if (!this.config) return
      this.config.appearance.theme = theme
      this.saveConfig()
    },
  },
})
