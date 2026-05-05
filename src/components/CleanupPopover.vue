<template>
  <div class="cleanup-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
    <div class="cleanup-popover">
      <div class="popover-header">{{ t('cleanup.title') }}</div>
      <div class="popover-body">
        <label
          v-for="opt in options"
          :key="opt.key"
          class="option-item"
        >
          <input
            type="checkbox"
            :checked="opt.value"
            @change="toggle(opt.key, $event)"
          />
          <span class="option-label">{{ t(`cleanup.${opt.key}`) }}</span>
          <span class="option-flag">{{ opt.flag }}</span>
        </label>
      </div>
      <div class="popover-footer">
        <button class="btn-execute" @click="$emit('execute')">{{ t('cleanup.execute') }}</button>
        <button class="btn-cancel" @click="$emit('close')">{{ t('common.cancel') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useConfigStore } from '../stores/configStore'
import type { CleanupConfig } from '../types/config'
import { t } from '../locales'

const emit = defineEmits<{ close: [], execute: [] }>()
const overlayMousedown = ref(false)
function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  emit('close')
}

const configStore = useConfigStore()

const optionKeys: (keyof CleanupConfig)[] = [
  'vacuumPristines',
  'vacuumPrunables',
  'includeExternals',
  'removeUnversionedTrees',
  'removeIgnoredTrees',
  'dropDavCache',
]

const flags: Record<keyof CleanupConfig, string> = {
  vacuumPristines: '--vacuum-pristines',
  vacuumPrunables: '--vacuum-prunables',
  includeExternals: '--include-externals',
  removeUnversionedTrees: '--remove-unversioned-trees',
  removeIgnoredTrees: '--remove-ignored-trees',
  dropDavCache: '--drop-dav-cache',
}

const options = computed(() => {
  const cfg = configStore.config?.cleanup
  return optionKeys.map((key) => ({
    key,
    flag: flags[key],
    value: cfg ? cfg[key] : false,
  }))
})

function toggle(key: keyof CleanupConfig, event: Event) {
  const checked = (event.target as HTMLInputElement).checked
  if (!configStore.config) return
  configStore.config.cleanup[key] = checked
  configStore.saveConfig()
}
</script>

<style scoped>
.cleanup-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 60px;
}
.cleanup-popover {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  width: 340px;
}
.popover-header {
  padding: 10px 14px;
  font-size: 13px;
  font-weight: 600;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
}
.popover-body {
  padding: 8px 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.option-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
}
.option-item input[type='checkbox'] {
  margin: 0;
  cursor: pointer;
}
.option-label {
  flex: 1;
}
.option-flag {
  font-family: monospace;
  font-size: 11px;
  color: var(--text-muted);
}
.popover-footer {
  padding: 10px 14px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}
.btn-execute {
  padding: 4px 14px;
  border: 1px solid var(--accent-color);
  background: var(--accent-color);
  color: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.btn-execute:hover {
  opacity: 0.9;
}
.btn-cancel {
  padding: 4px 14px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.btn-cancel:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
</style>
