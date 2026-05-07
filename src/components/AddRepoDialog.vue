<template>
  <div v-if="visible" class="dialog-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
    <div class="dialog">
      <div class="dialog-header">
        <h3 class="dialog-title">{{ t('addRepo.title') }}</h3>
        <button class="btn btn-icon btn-ghost" @click="$emit('close')">
          <X :size="16" />
        </button>
      </div>
      <div class="dialog-body">
        <div class="mode-tabs">
          <button
            :class="{ active: mode === 'local' }"
            @click="mode = 'local'"
            class="mode-tab"
          >
            <FolderOpen :size="16" />
            <span>{{ t('addRepo.openWorkingCopy') }}</span>
          </button>
          <button
            :class="{ active: mode === 'checkout' }"
            @click="mode = 'checkout'"
            class="mode-tab"
          >
            <Download :size="16" />
            <span>{{ t('addRepo.checkoutRepo') }}</span>
          </button>
        </div>

        <div v-if="mode === 'local'" class="form-section">
          <label class="form-label">{{ t('addRepo.workingCopyPath') }}</label>
          <div class="input-with-btn">
            <input v-model="localPath" :placeholder="t('addRepo.pathPlaceholder')" @keyup.enter="openLocal" class="input" />
            <button @click="browseLocalPath" class="btn btn-secondary" :title="t('addRepo.browse')">
              <FolderOpen :size="16" />
            </button>
          </div>
        </div>

        <div v-if="mode === 'checkout'" class="form-section">
          <label class="form-label">{{ t('addRepo.repoUrl') }}</label>
          <input v-model="repoUrl" placeholder="https://svn.example.com/repo/trunk" class="input" />
          <label class="form-label">{{ t('addRepo.checkoutTo') }}</label>
          <input v-model="checkoutDest" placeholder="C:\path\to\destination" class="input" />
        </div>

        <div v-if="recentRepos.length > 0" class="recent-section">
          <h4 class="recent-title">{{ t('addRepo.recentRepos') }}</h4>
          <div
            v-for="repo in recentRepos"
            :key="repo.path"
            class="recent-item"
            @click="selectRecent(repo.path)"
          >
            <FolderOpen :size="14" class="recent-icon" />
            <span class="recent-path">{{ repo.path }}</span>
            <span class="recent-date">{{ formatDate(repo.lastOpened) }}</span>
          </div>
        </div>

        <div v-if="errorMessage" class="error-message">
          <AlertCircle :size="14" />
          <span>{{ errorMessage }}</span>
        </div>
      </div>
      <div class="dialog-footer">
        <button @click="$emit('close')" class="btn btn-secondary">{{ t('common.cancel') }}</button>
        <button
          v-if="mode === 'local'"
          @click="openLocal"
          :disabled="!localPath.trim()"
          class="btn btn-primary"
        >
          <FolderOpen :size="14" />
          <span>{{ t('addRepo.open') }}</span>
        </button>
        <button
          v-if="mode === 'checkout'"
          @click="doCheckout"
          :disabled="!repoUrl.trim() || !checkoutDest.trim()"
          class="btn btn-primary"
        >
          <Download :size="14" />
          <span>{{ t('addRepo.checkout') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, Download, X, AlertCircle } from 'lucide-vue-next'
import type { RepoEntry } from '../types/config'
import { t } from '../locales'

defineProps<{
  visible: boolean
  recentRepos: RepoEntry[]
}>()

const emit = defineEmits<{
  close: []
  openRepo: [path: string]
}>()

const overlayMousedown = ref(false)
function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  emit('close')
}

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

async function browseLocalPath() {
  const selected = await open({
    title: t('addRepo.selectDirectory'),
    directory: true,
  })
  if (selected) {
    localPath.value = selected.toString()
  }
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
    errorMessage.value = `${t('addRepo.checkoutFailed')}: ${e}`
  }
}
</script>

<style scoped>
.dialog-overlay {
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

.dialog {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  width: 480px;
  max-height: 80vh;
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

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--color-border);
}

.dialog-title {
  margin: 0;
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-text-primary);
}

.dialog-body {
  flex: 1;
  padding: var(--space-5);
  overflow: auto;
}

.mode-tabs {
  display: flex;
  gap: var(--space-2);
  margin-bottom: var(--space-4);
}

.mode-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--color-border-input);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-base);
  transition: all var(--transition-fast);
}

.mode-tab:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.mode-tab.active {
  background: var(--color-accent);
  color: var(--color-text-inverse);
  border-color: var(--color-accent);
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-bottom: var(--space-4);
}

.form-label {
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--color-text-primary);
}

.input-with-btn {
  display: flex;
  gap: var(--space-2);
}

.input-with-btn input {
  flex: 1;
}

.recent-section {
  margin-top: var(--space-3);
}

.recent-title {
  margin: 0 0 var(--space-2);
  font-size: var(--text-base);
  color: var(--color-text-secondary);
  font-weight: 500;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-md);
  cursor: pointer;
  margin-bottom: var(--space-1);
  font-size: var(--text-base);
  transition: all var(--transition-fast);
}

.recent-item:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-border-input);
}

.recent-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.recent-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-primary);
}

.recent-date {
  font-size: var(--text-sm);
  color: var(--color-text-muted);
  margin-left: var(--space-3);
  flex-shrink: 0;
}

.error-message {
  margin-top: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--color-danger-muted);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-md);
  color: var(--color-danger);
  font-size: var(--text-sm);
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-5);
  border-top: 1px solid var(--color-border);
}
</style>
