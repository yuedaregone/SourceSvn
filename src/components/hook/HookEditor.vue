<template>
  <div class="hook-editor">
    <div class="editor-header">
      <h3>{{ isNew ? '添加Hook' : '编辑Hook' }}</h3>
    </div>
    <div class="editor-content">
      <div class="form-group">
        <label class="form-label">名称</label>
        <input
          v-model="formData.name"
          :disabled="!isNew"
          placeholder="输入hook名称"
          class="input"
        />
      </div>
      <div class="form-group">
        <label class="form-label">脚本路径</label>
        <div class="path-input">
          <input
            v-model="formData.script_path"
            placeholder="选择脚本文件 (.js/.ts/.exe)"
            class="input"
          />
          <button @click="selectFile" class="btn btn-secondary">
            <FolderOpen :size="14" />
            <span>选择</span>
          </button>
        </div>
      </div>
      <div class="form-group">
        <label class="form-label checkbox-label">
          <input
            type="checkbox"
            v-model="formData.enabled"
            class="checkbox"
          />
          <span>启用</span>
        </label>
      </div>
    </div>
    <div class="editor-actions">
      <button @click="handleTest" class="btn btn-secondary" :disabled="!canTest || testing">
        {{ testing ? '测试中...' : '测试' }}
      </button>
      <button @click="$emit('cancel')" class="btn btn-secondary">
        取消
      </button>
      <button @click="handleSave" class="btn btn-primary" :disabled="!isValid">
        保存
      </button>
    </div>
    <div v-if="testResult" :class="['test-result', testResult.success ? 'success' : 'error']">
      {{ testResult.message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { FolderOpen } from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import type { HookHandlerConfig } from '../../stores/hook'

const props = defineProps<{
  handler: HookHandlerConfig | null
  isNew: boolean
}>()

const emit = defineEmits<{
  save: [handler: HookHandlerConfig]
  cancel: []
}>()

const formData = ref<HookHandlerConfig>({
  name: '',
  script_path: '',
  enabled: true
})

const isValid = computed(() => {
  return formData.value.name.trim() !== '' && formData.value.script_path.trim() !== ''
})

const canTest = computed(() => {
  return formData.value.script_path.trim() !== ''
})

const testing = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)

watch(() => props.handler, (newHandler) => {
  if (newHandler) {
    formData.value = { ...newHandler }
  } else {
    formData.value = {
      name: '',
      script_path: '',
      enabled: true
    }
  }
}, { immediate: true })

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [{
      name: '脚本文件',
      extensions: ['js', 'ts', 'exe', 'bat', 'sh']
    }]
  })
  if (selected) {
    formData.value.script_path = selected
  }
}

function handleSave() {
  if (isValid.value) {
    emit('save', { ...formData.value })
  }
}

async function handleTest() {
  testing.value = true
  testResult.value = null
  try {
    const result = await invoke('hook_emit', {
      scriptPath: formData.value.script_path,
      hookType: 'PostCommit',
      repoPath: '.'
    })
    testResult.value = { success: true, message: `执行成功: ${JSON.stringify(result)}` }
  } catch (e) {
    testResult.value = { success: false, message: `执行失败: ${String(e)}` }
  } finally {
    testing.value = false
  }
}
</script>

<style scoped>
.hook-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  flex: 1;
  min-width: 0;
}

.editor-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--color-border);
}

.editor-header h3 {
  margin: 0;
  font-size: var(--text-lg);
  color: var(--color-text-primary);
}

.editor-content {
  flex: 1;
  padding: var(--space-4);
  overflow-y: auto;
}

.form-group {
  margin-bottom: var(--space-4);
}

.form-label {
  display: block;
  margin-bottom: var(--space-2);
  font-weight: 500;
  font-size: var(--text-base);
  color: var(--color-text-primary);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
}

.checkbox-label input {
  width: auto;
}

.path-input {
  display: flex;
  gap: var(--space-2);
}

.path-input .input {
  flex: 1;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4);
  border-top: 1px solid var(--color-border);
}

.test-result {
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-sm);
  border-top: 1px solid var(--color-border);
}

.test-result.success {
  background: var(--color-success-muted, #f0fdf4);
  color: var(--color-success, #16a34a);
}

.test-result.error {
  background: var(--color-danger-muted, #fef2f2);
  color: var(--color-danger, #dc2626);
}
</style>
