<template>
  <div class="settings-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
    <div class="settings-modal">
      <div class="settings-header">
        <h3 class="settings-title">{{ t('settings.title') }}</h3>
        <button class="btn btn-icon btn-ghost" @click="$emit('close')">
          <X :size="16" />
        </button>
      </div>
      <div class="settings-body">
        <div class="settings-tabs">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key"
          >
            <component :is="tab.icon" :size="16" />
            <span>{{ tab.label }}</span>
          </button>
        </div>
        <div class="settings-content">
          <!-- General -->
          <div v-if="activeTab === 'general'" class="tab-panel">
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.theme') }}</label>
              <div class="radio-group">
                <label class="radio-option">
                  <input type="radio" v-model="config.appearance.theme" value="light" class="radio" />
                  <span>{{ t('settings.light') }}</span>
                </label>
                <label class="radio-option">
                  <input type="radio" v-model="config.appearance.theme" value="dark" class="radio" />
                  <span>{{ t('settings.dark') }}</span>
                </label>
                <label class="radio-option">
                  <input type="radio" v-model="config.appearance.theme" value="system" class="radio" />
                  <span>{{ t('settings.system') }}</span>
                </label>
              </div>
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.uiFont') }}</label>
              <div class="input-inline">
                <input v-model="config.appearance.uiFontFamily" class="input input-mid" />
                <select v-model.number="config.appearance.uiFontSize" class="select input-small">
                  <option v-for="s in fontSizeOptions" :key="s" :value="s">{{ s }}px</option>
                </select>
              </div>
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.codeFont') }}</label>
              <div class="input-inline">
                <input v-model="config.appearance.codeFontFamily" class="input input-mid" />
                <select v-model.number="config.appearance.codeFontSize" class="select input-small">
                  <option v-for="s in fontSizeOptions" :key="s" :value="s">{{ s }}px</option>
                </select>
              </div>
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.iconSize') }}</label>
              <select v-model.number="config.appearance.iconSize" class="select">
                <option :value="16">16px</option>
                <option :value="20">20px</option>
                <option :value="24">24px</option>
                <option :value="32">32px</option>
              </select>
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.externalEditor') }}</label>
              <div class="input-with-btn">
                <input v-model="config.externalEditor" :placeholder="t('settings.editorPlaceholder')" class="input" />
                <button @click="selectEditorPath" class="btn btn-secondary">
                  <FolderOpen :size="14" />
                  <span>{{ t('settings.browse') }}</span>
                </button>
              </div>
            </div>
            <div class="setting-row">
              <button @click="resetAppearance" class="btn btn-secondary">
                <RotateCcw :size="14" />
                <span>{{ t('settings.resetDefault') }}</span>
              </button>
            </div>
          </div>

          <!-- SVN -->
          <div v-if="activeTab === 'svn'" class="tab-panel">
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.svnPath') }}</label>
              <div class="input-with-btn">
                <input v-model="config.svn.executable" :placeholder="t('settings.svnPathPlaceholder')" class="input" />
                <button @click="selectSvnPath" class="btn btn-secondary">
                  <FolderOpen :size="14" />
                  <span>{{ t('settings.browse') }}</span>
                </button>
                <button @click="detectSvnPath" :disabled="isDetecting" class="btn btn-primary">
                  <Search :size="14" />
                  <span>{{ isDetecting ? t('settings.detecting') : t('settings.autoDetect') }}</span>
                </button>
              </div>
              <span v-if="detectStatus" class="detect-status" :class="{ error: detectStatus.includes('失败') }">
                {{ detectStatus }}
              </span>
            </div>
          </div>

          <!-- AI -->
          <div v-if="activeTab === 'ai'" class="tab-panel">
            <div class="setting-row">
              <label class="setting-label">Provider</label>
              <select v-model="config.ai.provider" class="select">
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
                <option value="custom">{{ t('settings.custom') }}</option>
              </select>
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.apiEndpoint') }}</label>
              <input v-model="config.ai.endpoint" placeholder="https://api.openai.com/v1" class="input" />
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.apiKey') }}</label>
              <input v-model="config.ai.apiKey" type="password" placeholder="sk-..." class="input" />
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.model') }}</label>
              <input v-model="config.ai.model" placeholder="gpt-4o-mini" class="input" />
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.timeout') }}</label>
              <input type="number" v-model.number="config.ai.timeoutSecs" min="5" max="300" class="input" />
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.commitPrompt') }}</label>
              <textarea v-model="config.ai.commitPrompt" rows="4" class="textarea textarea--no-resize" :placeholder="t('settings.promptPlaceholder')"></textarea>
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.reviewPrompt') }}</label>
              <textarea v-model="config.ai.reviewPrompt" rows="4" class="textarea textarea--no-resize" :placeholder="t('settings.promptPlaceholder')"></textarea>
            </div>
          </div>

          <!-- Hook -->
          <div v-if="activeTab === 'hook'" class="tab-panel tab-panel--hook">
            <HookConfig />
          </div>

          <!-- Advanced -->
          <div v-if="activeTab === 'advanced'" class="tab-panel">
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.svnTimeout') }}</label>
              <input type="number" v-model.number="config.advanced.svnTimeoutSecs" min="10" max="600" class="input" />
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.logLevel') }}</label>
              <select v-model="config.advanced.logLevel" class="select">
                <option value="error">Error</option>
                <option value="warn">Warn</option>
                <option value="info">Info</option>
                <option value="debug">Debug</option>
              </select>
            </div>
            <div class="setting-row setting-row-inline">
              <label class="setting-label">{{ t('settings.confirmBeforeCommit') }}</label>
              <input type="checkbox" v-model="config.behavior.confirmBeforeCommit" class="checkbox" />
            </div>
            <div class="setting-row setting-row-inline">
              <label class="setting-label">{{ t('settings.confirmBeforeRevert') }}</label>
              <input type="checkbox" v-model="config.behavior.confirmBeforeRevert" class="checkbox" />
            </div>
            <div class="setting-row">
              <label class="setting-label">{{ t('settings.autoRefresh') }}</label>
              <input type="number" v-model.number="config.behavior.autoRefreshSecs" min="0" max="300" class="input" />
            </div>
          </div>
        </div>
      </div>
      <div class="settings-footer">
        <button @click="$emit('close')" class="btn btn-secondary">{{ t('common.cancel') }}</button>
        <button @click="apply" class="btn btn-secondary">
          <Check :size="14" />
          <span>{{ t('settings.apply') }}</span>
        </button>
        <button @click="saveAndClose" class="btn btn-primary">
          <Save :size="14" />
          <span>{{ t('settings.save') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useConfigStore } from '../stores/configStore'
import { useToastStore } from '../stores/toastStore'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type { AppConfig } from '../types/config'
import { t } from '../locales'
import { X, Settings, GitBranch, Sparkles, Shield, FolderOpen, RotateCcw, Search, Check, Save, Terminal } from 'lucide-vue-next'
import HookConfig from '../components/hook/HookConfig.vue'

const emit = defineEmits<{
  close: []
}>()

const configStore = useConfigStore()
const activeTab = ref('general')
const isDetecting = ref(false)
const detectStatus = ref('')
const overlayMousedown = ref(false)

function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  emit('close')
}

const tabs = computed(() => [
  { key: 'general', label: t('settings.general'), icon: Settings },
  { key: 'svn', label: 'SVN', icon: GitBranch },
  { key: 'ai', label: 'AI', icon: Sparkles },
  { key: 'hook', label: 'Hook', icon: Terminal },
  { key: 'advanced', label: t('settings.advanced'), icon: Shield },
])

const fontSizeOptions = [10, 11, 12, 13, 14, 15, 16, 18, 20]

const ua = navigator.userAgent
const isWin = ua.includes('Windows')
const isMac = ua.includes('Macintosh') || ua.includes('Mac OS')

const defaultAppearance = {
  theme: 'light',
  uiFontFamily: isWin
    ? 'Segoe UI, sans-serif'
    : isMac
      ? '-apple-system, BlinkMacSystemFont, sans-serif'
      : 'Noto Sans, Ubuntu, sans-serif',
  uiFontSize: 14,
  codeFontFamily: isWin
    ? 'Consolas, Courier New, monospace'
    : isMac
      ? 'Menlo, Monaco, monospace'
      : 'DejaVu Sans Mono, Ubuntu Mono, monospace',
  codeFontSize: 13,
  iconSize: 20,
}

const config = reactive<AppConfig>({
  configVersion: 1,
  window: { width: 1200, height: 800, maximized: false },
  appearance: { ...defaultAppearance },
  session: { openTabs: [], activeTabIndex: 0, recentRepos: [], maxRecentRepos: 20 },
  svn: {},
  ai: { provider: 'openai', endpoint: 'https://api.openai.com/v1', apiKey: '', model: 'gpt-4o-mini', timeoutSecs: 30, commitPrompt: '', reviewPrompt: '' },
  diff: { contextLines: 3, ignoreWhitespace: false, viewMode: 'unified' },
  log: { fetchLimit: 100, showChangedPaths: true },
  commit: {},
  behavior: { confirmBeforeCommit: true, confirmBeforeRevert: true },
  advanced: { svnTimeoutSecs: 60, logLevel: 'warn' },
  cleanup: { vacuumPristines: false, vacuumPrunables: false, includeExternals: false, removeUnversionedTrees: false, removeIgnoredTrees: false, dropDavCache: false },
  externalEditor: '',
})

onMounted(() => {
  if (configStore.config) {
    Object.assign(config, JSON.parse(JSON.stringify(configStore.config)))
    // 确保字体配置有默认值
    if (!config.appearance.uiFontFamily) {
      config.appearance.uiFontFamily = defaultAppearance.uiFontFamily
    }
    if (!config.appearance.codeFontFamily) {
      config.appearance.codeFontFamily = defaultAppearance.codeFontFamily
    }
    if (!config.appearance.uiFontSize) {
      config.appearance.uiFontSize = defaultAppearance.uiFontSize
    }
    if (!config.appearance.codeFontSize) {
      config.appearance.codeFontSize = defaultAppearance.codeFontSize
    }
    if (!config.appearance.iconSize) {
      config.appearance.iconSize = defaultAppearance.iconSize
    }
    if (!config.appearance.theme) {
      config.appearance.theme = defaultAppearance.theme
    }
    if (!config.ai.commitPrompt) {
      config.ai.commitPrompt = 'You are a helpful assistant that generates concise commit messages for code changes. Output ONLY the commit message, no explanation.'
    }
    if (!config.ai.reviewPrompt) {
      config.ai.reviewPrompt = 'You are a senior code reviewer. Review the following code changes and provide constructive feedback on potential issues, bugs, and improvements. Be concise.'
    }
  }
})

function resetAppearance() {
  Object.assign(config.appearance, defaultAppearance)
}

async function detectSvnPath() {
  isDetecting.value = true
  detectStatus.value = t('settings.detectingStatus')
  try {
    const result = await invoke<string>('svn_detect_executable')
    config.svn.executable = result
    detectStatus.value = '✓ ' + t('settings.detectSuccess')
  } catch (error) {
    detectStatus.value = `✗ ${t('settings.detectFailed')}: ${error}`
  } finally {
    isDetecting.value = false
  }
}

async function selectSvnPath() {
  try {
    const selected = await open({
      title: t('settings.selectSvnPath'),
      filters: [
        { name: t('settings.executable'), extensions: ['exe', 'bat', 'cmd'] },
        { name: t('settings.allFiles'), extensions: ['*'] },
      ],
    })
    if (selected) {
      config.svn.executable = selected.toString()
      detectStatus.value = ''
    }
  } catch (error) {
    useToastStore().error(t('toast.pathNotFound'))
  }
}

async function selectEditorPath() {
  try {
    const selected = await open({
      title: t('settings.selectEditor'),
      filters: [
        { name: t('settings.executable'), extensions: ['exe', 'bat', 'cmd'] },
        { name: t('settings.allFiles'), extensions: ['*'] },
      ],
    })
    if (selected) {
      config.externalEditor = selected.toString()
    }
  } catch (error) {
    useToastStore().error(t('toast.pathNotFound'))
  }
}

function applyConfig() {
  if (!configStore.config) return
  Object.assign(configStore.config, JSON.parse(JSON.stringify(config)))
}

function apply() {
  applyConfig()
}

function saveAndClose() {
  applyConfig()
  configStore.saveConfig()
  emit('close')
}

let applyTimer: ReturnType<typeof setTimeout> | null = null
watch(() => config.behavior.autoRefreshSecs, () => {
  if (applyTimer) clearTimeout(applyTimer)
  applyTimer = setTimeout(() => applyConfig(), 500)
})
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.settings-modal {
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  width: 640px;
  height: 520px;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-xl);
  animation: scaleIn 0.2s ease;
}

@keyframes scaleIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--color-border);
}

.settings-title {
  margin: 0;
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-text-primary);
}

.settings-body {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 360px;
}

.settings-tabs {
  width: 140px;
  border-right: 1px solid var(--color-border);
  padding: var(--space-2) var(--space-1);
  flex-shrink: 0;
}

.settings-tabs button {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: none;
  background: transparent;
  text-align: left;
  cursor: pointer;
  font-size: var(--text-base);
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.settings-tabs button:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.settings-tabs button.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
  font-weight: 500;
}

.settings-content {
  flex: 1;
  padding: var(--space-5);
  overflow: auto;
}

.tab-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.tab-panel--hook {
  height: 500px;
  gap: 0;
}

.setting-row {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.setting-label {
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--color-text-primary);
}

.setting-row-inline {
  flex-direction: row;
  align-items: center;
  gap: var(--space-3);
}

.setting-row-inline label {
  flex: 1;
}

.radio-group {
  display: flex;
  gap: var(--space-4);
}

.radio-option {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-weight: 400;
  cursor: pointer;
  color: var(--color-text-primary);
  font-size: var(--text-base);
}

.input-inline {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}

.input-inline .input-mid {
  flex: 1;
  min-width: 0;
  width: auto;
}

.input-inline .input-small {
  width: 90px;
  min-width: 90px;
  flex-shrink: 0;
}

.input-with-btn {
  display: flex;
  gap: var(--space-2);
}

.input-with-btn input {
  flex: 1;
}

.detect-status {
  font-size: var(--text-sm);
  color: var(--color-success);
}

.detect-status.error {
  color: var(--color-danger);
}

.settings-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--color-border);
}

.checkbox {
  appearance: none;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--color-border-input);
  border-radius: var(--radius-sm);
  background: var(--color-bg-primary);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.checkbox:checked {
  background: var(--color-accent);
  border-color: var(--color-accent);
}

.checkbox:checked::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 5px;
  height: 9px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.checkbox:hover {
  border-color: var(--color-accent);
}

.radio {
  appearance: none;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--color-border-input);
  border-radius: 50%;
  background: var(--color-bg-primary);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.radio:checked {
  border-color: var(--color-accent);
}

.radio:checked::after {
  content: '';
  position: absolute;
  left: 3px;
  top: 3px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-accent);
}
</style>
