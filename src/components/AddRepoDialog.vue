<template>
  <div v-if="visible" class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <h3>打开仓库</h3>
        <button class="close-btn" @click="$emit('close')">&times;</button>
      </div>
      <div class="dialog-body">
        <div class="mode-tabs">
          <button :class="{ active: mode === 'local' }" @click="mode = 'local'">打开工作副本</button>
          <button :class="{ active: mode === 'checkout' }" @click="mode = 'checkout'">检出仓库</button>
        </div>

        <div v-if="mode === 'local'" class="form-section">
          <label>工作副本路径</label>
          <div class="input-with-btn">
            <input v-model="localPath" placeholder="C:\path\to\working\copy" @keyup.enter="openLocal" />
          </div>
        </div>

        <div v-if="mode === 'checkout'" class="form-section">
          <label>仓库 URL</label>
          <input v-model="repoUrl" placeholder="https://svn.example.com/repo/trunk" />
          <label>检出到</label>
          <input v-model="checkoutDest" placeholder="C:\path\to\destination" />
        </div>

        <div v-if="recentRepos.length > 0" class="recent-section">
          <h4>最近打开</h4>
          <div
            v-for="repo in recentRepos"
            :key="repo.path"
            class="recent-item"
            @click="selectRecent(repo.path)"
          >
            <span class="recent-path">{{ repo.path }}</span>
            <span class="recent-date">{{ formatDate(repo.lastOpened) }}</span>
          </div>
        </div>

        <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
      </div>
      <div class="dialog-footer">
        <button @click="$emit('close')" class="cancel-btn">取消</button>
        <button
          v-if="mode === 'local'"
          @click="openLocal"
          :disabled="!localPath.trim()"
          class="confirm-btn"
        >
          打开
        </button>
        <button
          v-if="mode === 'checkout'"
          @click="doCheckout"
          :disabled="!repoUrl.trim() || !checkoutDest.trim()"
          class="confirm-btn"
        >
          检出
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { RepoEntry } from '../types/config'

defineProps<{
  visible: boolean
  recentRepos: RepoEntry[]
}>()

const emit = defineEmits<{
  close: []
  openRepo: [path: string]
}>()

const mode = ref<'local' | 'checkout'>('local')
const localPath = ref('')
const repoUrl = ref('')
const checkoutDest = ref('')
const errorMessage = ref('')

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN')
  } catch {
    return ''
  }
}

function selectRecent(path: string) {
  localPath.value = path
  mode.value = 'local'
}

function openLocal() {
  if (!localPath.value.trim()) return
  emit('openRepo', localPath.value.trim())
}

async function doCheckout() {
  if (!repoUrl.value.trim() || !checkoutDest.value.trim()) return
  errorMessage.value = ''
  try {
    await invoke('svn_checkout', {
      url: repoUrl.value.trim(),
      dest: checkoutDest.value.trim(),
    })
    emit('openRepo', checkoutDest.value.trim())
  } catch (e) {
    errorMessage.value = `检出失败: ${e}`
  }
}
</script>

<style scoped>
.dialog-overlay {
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
.dialog {
  background: var(--bg-primary);
  border-radius: 8px;
  width: 480px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow);
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}
.dialog-header h3 {
  margin: 0;
  font-size: 15px;
  color: var(--text-primary);
}
.close-btn {
  border: none;
  background: transparent;
  font-size: 22px;
  cursor: pointer;
  color: var(--text-muted);
}
.close-btn:hover {
  color: var(--text-primary);
}
.dialog-body {
  flex: 1;
  padding: 16px 20px;
  overflow: auto;
}
.mode-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
}
.mode-tabs button {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.mode-tabs button.active {
  background: var(--accent-color);
  color: #fff;
  border-color: var(--accent-color);
}
.form-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
}
.form-section label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}
.form-section input {
  width: 100%;
  padding: 7px 10px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}
.form-section input:focus {
  border-color: var(--accent-color);
  outline: none;
}
.input-with-btn {
  display: flex;
  gap: 8px;
}
.input-with-btn input {
  flex: 1;
}
.recent-section {
  margin-top: 8px;
}
.recent-section h4 {
  margin: 0 0 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
.recent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 10px;
  border: 1px solid var(--border-light);
  border-radius: 4px;
  cursor: pointer;
  margin-bottom: 4px;
  font-size: 13px;
}
.recent-item:hover {
  background: var(--bg-hover);
  border-color: var(--border-input);
}
.recent-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-primary);
}
.recent-date {
  font-size: 11px;
  color: var(--text-muted);
  margin-left: 12px;
  flex-shrink: 0;
}
.error-message {
  margin-top: 8px;
  padding: 6px 10px;
  background: var(--diff-del-bg);
  border: 1px solid var(--danger-color);
  border-radius: 4px;
  color: var(--danger-color);
  font-size: 12px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
}
.cancel-btn {
  padding: 6px 18px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.confirm-btn {
  padding: 6px 18px;
  border: 1px solid var(--accent-color);
  background: var(--accent-color);
  color: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.confirm-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.confirm-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}
</style>
