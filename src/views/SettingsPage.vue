<template>
  <div v-if="true" class="settings-overlay" @click.self="$emit('close')">
    <div class="settings-modal">
      <div class="settings-header">
        <h3>设置</h3>
        <button @click="$emit('close')">×</button>
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
          <div v-if="activeTab === 'general'">
            <label>主题</label>
            <select v-model="config.appearance.theme">
              <option value="light">亮色</option>
              <option value="dark">深色</option>
            </select>
            <label>UI 字体大小</label>
            <input type="number" v-model.number="config.appearance.uiFontSize" />
            <label>代码字体</label>
            <input v-model="config.appearance.codeFontFamily" />
          </div>
          <div v-if="activeTab === 'svn'">
            <label>SVN 可执行文件路径</label>
            <input v-model="config.svn.executable" placeholder="自动检测" />
          </div>
          <div v-if="activeTab === 'ai'">
            <label>API 端点</label>
            <input v-model="config.ai.endpoint" />
            <label>API 密钥</label>
            <input v-model="config.ai.apiKey" type="password" />
            <label>模型</label>
            <input v-model="config.ai.model" />
            <label>超时（秒）</label>
            <input type="number" v-model.number="config.ai.timeoutSecs" />
          </div>
          <div v-if="activeTab === 'advanced'">
            <label>SVN 超时（秒）</label>
            <input type="number" v-model.number="config.advanced.svnTimeoutSecs" />
            <label>日志级别</label>
            <select v-model="config.advanced.logLevel">
              <option value="error">Error</option>
              <option value="warn">Warn</option>
              <option value="info">Info</option>
              <option value="debug">Debug</option>
            </select>
          </div>
        </div>
      </div>
      <div class="settings-footer">
        <button @click="$emit('close')" class="cancel-btn">取消</button>
        <button @click="save" class="save-btn">保存</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useConfigStore } from '../stores/configStore'

defineEmits<{
  close: []
}>()

const configStore = useConfigStore()
const activeTab = ref('general')

const tabs = [
  { key: 'general', label: '通用' },
  { key: 'svn', label: 'SVN' },
  { key: 'ai', label: 'AI' },
  { key: 'advanced', label: '高级' },
]

const config = reactive({
  appearance: { theme: 'light', uiFontSize: 14, codeFontFamily: 'monospace' },
  svn: { executable: '' },
  ai: { endpoint: 'https://api.openai.com/v1', apiKey: '', model: 'gpt-4o-mini', timeoutSecs: 30 },
  advanced: { svnTimeoutSecs: 60, logLevel: 'warn' },
})

onMounted(() => {
  if (configStore.config) {
    Object.assign(config.appearance, configStore.config.appearance)
    Object.assign(config.svn, configStore.config.svn)
    Object.assign(config.ai, configStore.config.ai)
    Object.assign(config.advanced, configStore.config.advanced)
  }
})

function save() {
  if (!configStore.config) return
  Object.assign(configStore.config.appearance, config.appearance)
  Object.assign(configStore.config.svn, config.svn)
  Object.assign(configStore.config.ai, config.ai)
  Object.assign(configStore.config.advanced, config.advanced)
  configStore.saveConfig()
}
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}
.settings-modal {
  background: #fff;
  border-radius: 8px;
  width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid #e8e8e8;
}
.settings-header h3 { margin: 0; }
.settings-header button {
  border: none;
  background: transparent;
  font-size: 20px;
  cursor: pointer;
}
.settings-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.settings-tabs {
  width: 120px;
  border-right: 1px solid #e8e8e8;
  padding: 8px 0;
}
.settings-tabs button {
  display: block;
  width: 100%;
  padding: 8px 16px;
  border: none;
  background: transparent;
  text-align: left;
  cursor: pointer;
  font-size: 13px;
}
.settings-tabs button.active {
  background: #e6f7ff;
  color: #1890ff;
}
.settings-content {
  flex: 1;
  padding: 16px;
  overflow: auto;
}
.settings-content label {
  display: block;
  margin-top: 12px;
  margin-bottom: 4px;
  font-size: 13px;
  font-weight: 500;
}
.settings-content input,
.settings-content select {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}
.settings-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid #e8e8e8;
}
.settings-footer button {
  padding: 6px 16px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.save-btn {
  background: #1890ff !important;
  color: #fff !important;
  border-color: #1890ff !important;
}
</style>
