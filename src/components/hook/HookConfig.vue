<template>
  <div class="hook-config">
    <div class="config-header">
      <h2>Hook配置</h2>
      <div class="config-actions">
        <label class="form-label checkbox-label">
          <input
            type="checkbox"
            v-model="config.enabled"
            @change="saveConfig"
          />
          <span>启用Hook系统</span>
        </label>
      </div>
    </div>
    <div class="config-content">
      <HookList
        :handlers="config.handlers"
        :selected-name="selectedName"
        @add="handleAdd"
        @select="handleSelect"
        @toggle="handleToggle"
        @delete="handleDelete"
      />
      <HookEditor
        v-if="selectedHandler || isNew"
        :handler="selectedHandler"
        :is-new="isNew"
        @save="handleSave"
        @cancel="handleCancel"
      />
      <div v-else class="editor-placeholder">
        <div class="placeholder-content">
          <Settings :size="48" />
          <p>选择一个hook进行编辑，或点击"添加Hook"创建新hook</p>
        </div>
      </div>
    </div>
    <div v-if="hookStore.error" class="config-error">
      <AlertCircle :size="14" />
      <span>{{ hookStore.error }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Settings, AlertCircle } from 'lucide-vue-next'
import { useHookStore } from '../../stores/hook'
import HookList from './HookList.vue'
import HookEditor from './HookEditor.vue'
import type { HookHandlerConfig } from '../../stores/hook'

const hookStore = useHookStore()

const selectedName = ref<string | null>(null)
const isNew = ref(false)

const config = computed(() => hookStore.config)

const selectedHandler = computed(() => {
  if (!selectedName.value) return null
  return config.value.handlers.find(h => h.name === selectedName.value) || null
})

onMounted(() => {
  hookStore.loadConfig()
})

function handleAdd() {
  selectedName.value = null
  isNew.value = true
}

function handleSelect(name: string) {
  selectedName.value = name
  isNew.value = false
}

async function handleToggle(name: string) {
  const handler = config.value.handlers.find(h => h.name === name)
  if (handler) {
    await hookStore.updateHandler(name, { ...handler, enabled: !handler.enabled })
  }
}

async function handleDelete(name: string) {
  if (confirm(`确定要删除hook "${name}" 吗？`)) {
    await hookStore.removeHandler(name)
    if (selectedName.value === name) {
      selectedName.value = null
      isNew.value = false
    }
  }
}

async function handleSave(handler: HookHandlerConfig) {
  if (isNew.value) {
    await hookStore.addHandler(handler)
  } else {
    await hookStore.updateHandler(handler.name, handler)
  }
  isNew.value = false
  selectedName.value = handler.name
}

function handleCancel() {
  isNew.value = false
  if (!selectedName.value) {
    selectedName.value = null
  }
}

async function saveConfig() {
  await hookStore.saveConfig()
}
</script>

<style scoped>
.hook-config {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.config-header h2 {
  margin: 0;
  font-size: 18px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-weight: normal;
}

.config-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.editor-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary, #999);
}

.placeholder-content {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.placeholder-content p {
  margin: 0;
  max-width: 240px;
  line-height: 1.5;
}

.config-error {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--danger-bg, #fef2f2);
  color: var(--danger-color, #dc2626);
  border-top: 1px solid var(--danger-border, #fecaca);
  font-size: 14px;
}
</style>
