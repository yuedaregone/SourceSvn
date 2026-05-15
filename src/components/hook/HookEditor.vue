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
        <label class="form-label">类型</label>
        <select v-model="formData.hook_type" class="input">
          <option value="PreCommit">PreCommit - 提交前</option>
          <option value="PostCommit">PostCommit - 提交后</option>
          <option value="PreUpdate">PreUpdate - 更新前</option>
          <option value="PostUpdate">PostUpdate - 更新后</option>
          <option value="StatusChange">StatusChange - 状态变更</option>
          <option value="ConflictDetected">ConflictDetected - 冲突检测</option>
          <option value="PreCheckout">PreCheckout - 检出前</option>
          <option value="PostCheckout">PostCheckout - 检出后</option>
          <option value="PreMerge">PreMerge - 合并前</option>
          <option value="PostMerge">PostMerge - 合并后</option>
        </select>
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
          />
          <span>启用</span>
        </label>
      </div>
    </div>
    <div class="editor-actions">
      <button @click="$emit('cancel')" class="btn btn-secondary">
        取消
      </button>
      <button @click="handleSave" class="btn btn-primary" :disabled="!isValid">
        保存
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { FolderOpen } from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'
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
  hook_type: 'PostCommit',
  script_path: '',
  enabled: true
})

const isValid = computed(() => {
  return formData.value.name.trim() !== '' && formData.value.script_path.trim() !== ''
})

watch(() => props.handler, (newHandler) => {
  if (newHandler) {
    formData.value = { ...newHandler }
  } else {
    formData.value = {
      name: '',
      hook_type: 'PostCommit',
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
  padding: 16px;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.editor-header h3 {
  margin: 0;
  font-size: 16px;
}

.editor-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  font-size: 14px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.checkbox-label input {
  width: auto;
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input .input {
  flex: 1;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px;
  border-top: 1px solid var(--border-color, #e0e0e0);
}
</style>
