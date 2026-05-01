<template>
  <div class="settings-overlay" @click.self="$emit('close')">
    <div class="settings-modal">
      <div class="settings-header">
        <h3>设置</h3>
        <button class="close-btn" @click="$emit('close')">&times;</button>
      </div>
      <div class="settings-body">
        <div class="settings-tabs">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </div>
        <div class="settings-content">
          <!-- General -->
          <div v-if="activeTab === 'general'" class="tab-panel">
            <div class="setting-row">
              <label>主题</label>
              <div class="radio-group">
                <label><input type="radio" v-model="config.appearance.theme" value="light" /> 亮色</label>
                <label><input type="radio" v-model="config.appearance.theme" value="dark" /> 深色</label>
                <label><input type="radio" v-model="config.appearance.theme" value="system" /> 跟随系统</label>
              </div>
            </div>
            <div class="setting-row">
              <label>UI 字体</label>
              <div class="input-inline">
                <input v-model="config.appearance.uiFontFamily" class="input-mid" />
                <select v-model.number="config.appearance.uiFontSize" class="input-small">
                  <option v-for="s in fontSizeOptions" :key="s" :value="s">{{ s }}px</option>
                </select>
              </div>
            </div>
            <div class="setting-row">
              <label>代码字体</label>
              <div class="input-inline">
                <input v-model="config.appearance.codeFontFamily" class="input-mid" />
                <select v-model.number="config.appearance.codeFontSize" class="input-small">
                  <option v-for="s in fontSizeOptions" :key="s" :value="s">{{ s }}px</option>
                </select>
              </div>
            </div>
            <div class="setting-row">
              <label>图标大小</label>
              <select v-model.number="config.appearance.iconSize" class="input-small">
                <option :value="16">16px</option>
                <option :value="20">20px</option>
                <option :value="24">24px</option>
                <option :value="32">32px</option>
              </select>
            </div>
            <div class="setting-row">
              <button @click="resetAppearance" class="reset-btn">恢复默认</button>
            </div>
          </div>

          <!-- SVN -->
          <div v-if="activeTab === 'svn'" class="tab-panel">
            <div class="setting-row">
              <label>SVN 可执行文件路径</label>
              <div class="input-with-btn">
                <input v-model="config.svn.executable" placeholder="留空则自动检测" />
                <button @click="detectSvnPath" :disabled="isDetecting" class="detect-btn">
                  {{ isDetecting ? '检测中...' : '自动检测' }}
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
              <label>Provider</label>
              <select v-model="config.ai.provider">
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
                <option value="custom">自定义</option>
              </select>
            </div>
            <div class="setting-row">
              <label>API 端点</label>
              <input v-model="config.ai.endpoint" placeholder="https://api.openai.com/v1" />
            </div>
            <div class="setting-row">
              <label>API 密钥</label>
              <input v-model="config.ai.apiKey" type="password" placeholder="sk-..." />
            </div>
            <div class="setting-row">
              <label>模型</label>
              <input v-model="config.ai.model" placeholder="gpt-4o-mini" />
            </div>
            <div class="setting-row">
              <label>超时（秒）</label>
              <input type="number" v-model.number="config.ai.timeoutSecs" min="5" max="300" />
            </div>
          </div>

          <!-- Advanced -->
          <div v-if="activeTab === 'advanced'" class="tab-panel">
            <div class="setting-row">
              <label>SVN 超时（秒）</label>
              <input type="number" v-model.number="config.advanced.svnTimeoutSecs" min="10" max="600" />
            </div>
            <div class="setting-row">
              <label>日志级别</label>
              <select v-model="config.advanced.logLevel">
                <option value="error">Error</option>
                <option value="warn">Warn</option>
                <option value="info">Info</option>
                <option value="debug">Debug</option>
              </select>
            </div>
            <div class="setting-row">
              <label>提交前确认</label>
              <input type="checkbox" v-model="config.behavior.confirmBeforeCommit" />
            </div>
            <div class="setting-row">
              <label>Revert 前确认</label>
              <input type="checkbox" v-model="config.behavior.confirmBeforeRevert" />
            </div>
            <div class="setting-row">
              <label>自动刷新间隔（秒，0=关闭）</label>
              <input type="number" v-model.number="config.behavior.autoRefreshSecs" min="0" max="300" />
            </div>
          </div>
        </div>
      </div>
      <div class="settings-footer">
        <button @click="$emit('close')" class="cancel-btn">取消</button>
        <button @click="saveAndClose" class="save-btn">保存</button>
        <button @click="apply" class="apply-btn">应用</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useConfigStore } from '../stores/configStore'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types/config'

const emit = defineEmits<{
  close: []
}>()

const configStore = useConfigStore()
const activeTab = ref('general')
const isDetecting = ref(false)
const detectStatus = ref('')

const tabs = [
  { key: 'general', label: '通用' },
  { key: 'svn', label: 'SVN' },
  { key: 'ai', label: 'AI' },
  { key: 'advanced', label: '高级' },
]

const fontSizeOptions = [10, 11, 12, 13, 14, 15, 16, 18, 20]

const defaultAppearance = {
  theme: 'light',
  uiFontFamily: 'Inter, -apple-system, sans-serif',
  uiFontSize: 14,
  codeFontFamily: 'Consolas, Monaco, monospace',
  codeFontSize: 13,
  iconSize: 20,
}

const config = reactive<AppConfig>({
  configVersion: 1,
  window: { width: 1200, height: 800, maximized: false },
  appearance: { ...defaultAppearance },
  session: { openTabs: [], activeTabIndex: 0, recentRepos: [], maxRecentRepos: 20 },
  svn: { executable: '' },
  ai: { provider: 'openai', endpoint: 'https://api.openai.com/v1', apiKey: '', model: 'gpt-4o-mini', timeoutSecs: 30 },
  diff: { contextLines: 3, ignoreWhitespace: false, viewMode: 'unified' },
  log: { fetchLimit: 100, showChangedPaths: true },
  commit: {},
  fileBrowser: { showHidden: false },
  behavior: { confirmBeforeCommit: true, confirmBeforeRevert: true, autoRefreshSecs: 0 },
  advanced: { svnTimeoutSecs: 60, logLevel: 'warn' },
})

onMounted(() => {
  if (configStore.config) {
    Object.assign(config, JSON.parse(JSON.stringify(configStore.config)))
  }
})

function resetAppearance() {
  Object.assign(config.appearance, defaultAppearance)
}

async function detectSvnPath() {
  isDetecting.value = true
  detectStatus.value = '正在检测...'
  try {
    const result = await invoke<string>('svn_detect_executable')
    config.svn.executable = result
    detectStatus.value = '✓ 检测成功'
  } catch (error) {
    detectStatus.value = `✗ 检测失败: ${error}`
  } finally {
    isDetecting.value = false
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
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}
.settings-modal {
  background: var(--bg-primary);
  border-radius: 8px;
  width: 640px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow);
}
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}
.settings-header h3 {
  margin: 0;
  font-size: 16px;
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
.settings-body {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 360px;
}
.settings-tabs {
  width: 120px;
  border-right: 1px solid var(--border-color);
  padding: 8px 0;
  flex-shrink: 0;
}
.settings-tabs button {
  display: block;
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: transparent;
  text-align: left;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
}
.settings-tabs button:hover {
  background: var(--bg-hover);
}
.settings-tabs button.active {
  background: var(--bg-active);
  color: var(--accent-color);
  font-weight: 500;
}
.settings-content {
  flex: 1;
  padding: 20px;
  overflow: auto;
}
.tab-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.setting-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.setting-row label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}
.setting-row input[type='text'],
.setting-row input[type='password'],
.setting-row input[type='number'],
.setting-row select {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}
.radio-group {
  display: flex;
  gap: 16px;
}
.radio-group label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-weight: 400;
  cursor: pointer;
  color: var(--text-primary);
}
.input-inline {
  display: flex;
  gap: 8px;
}
.input-mid {
  flex: 1;
}
.input-small {
  width: 80px;
}
.input-with-btn {
  display: flex;
  gap: 8px;
}
.input-with-btn input {
  flex: 1;
}
.detect-btn {
  padding: 6px 14px;
  border: 1px solid var(--accent-color);
  background: var(--bg-primary);
  color: var(--accent-color);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
}
.detect-btn:hover:not(:disabled) {
  background: var(--bg-active);
}
.detect-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.detect-status {
  font-size: 12px;
  color: var(--success-color);
}
.detect-status.error {
  color: var(--danger-color);
}
.reset-btn {
  align-self: flex-start;
  padding: 5px 14px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.reset-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.settings-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
}
.settings-footer button {
  padding: 6px 18px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.cancel-btn:hover {
  border-color: var(--text-muted);
}
.apply-btn {
  color: var(--accent-color);
  border-color: var(--accent-color);
}
.apply-btn:hover {
  background: var(--bg-active);
}
.save-btn {
  background: var(--accent-color);
  color: #fff;
  border-color: var(--accent-color);
}
.save-btn:hover {
  background: var(--accent-hover);
}
</style>
