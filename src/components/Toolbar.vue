<template>
  <div class="toolbar">
    <div class="toolbar-left">
      <button @click="$emit('pull')" :disabled="loading" class="icon-btn" :title="t('toolbar.pull')">
        <Download :size="16" />
      </button>
      <button @click="$emit('cleanup')" :disabled="loading" class="icon-btn" :title="t('toolbar.cleanup')">
        <Brush :size="16" />
      </button>
      <div class="cleanup-dropdown-wrapper" ref="dropdownRef">
        <button @click="showPopover = !showPopover" :disabled="loading" class="icon-btn dropdown-btn" :title="t('toolbar.cleanupOptions')">
          <Brush :size="14" />
          <ChevronDown :size="8" />
        </button>
        <CleanupPopover
          v-if="showPopover"
          @close="showPopover = false"
          @execute="handleExecute"
        />
      </div>
      <button @click="$emit('refresh')" :disabled="loading" class="icon-btn" :title="t('toolbar.refresh')">
        <RefreshCw :size="16" />
      </button>
    </div>
    <span v-if="loading" class="loading-indicator">{{ t('toolbar.processing') }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Download, RefreshCw, Brush, ChevronDown } from 'lucide-vue-next'
import CleanupPopover from './CleanupPopover.vue'
import { t } from '../locales'

defineProps<{
  loading: boolean
}>()

const emit = defineEmits<{
  pull: []
  refresh: []
  cleanup: []
  cleanupOptions: []
}>()

const showPopover = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

function handleClickOutside(e: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    showPopover.value = false
  }
}

function handleExecute() {
  showPopover.value = false
  emit('cleanupOptions')
}

onMounted(() => document.addEventListener('click', handleClickOutside))
onUnmounted(() => document.removeEventListener('click', handleClickOutside))
</script>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 var(--spacing-md);
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}
.toolbar button {
  padding: 4px 12px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
}
.toolbar button:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
  background: var(--bg-active);
}
.toolbar .icon-btn {
  padding: 4px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}
.toolbar button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.loading-indicator {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
