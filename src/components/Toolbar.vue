<template>
  <div class="toolbar">
    <!-- 左侧：操作按钮 -->
    <div class="toolbar-section">
      <button @click="$emit('pull')" :disabled="busy" class="toolbar-btn" :title="t('toolbar.pull')">
        <Download :size="15" />
        <span class="btn-label">{{ t('toolbar.pull') }}</span>
      </button>
      <div class="toolbar-divider" />
      <button @click="$emit('cleanup')" :disabled="busy" class="toolbar-btn" :title="t('toolbar.cleanup')">
        <Brush :size="15" />
      </button>
      <div class="cleanup-dropdown-wrapper" ref="dropdownRef">
        <button @click="showPopover = !showPopover" :disabled="busy" class="toolbar-btn dropdown-trigger" :title="t('toolbar.cleanupOptions')">
          <ChevronDown :size="12" />
        </button>
        <CleanupPopover
          v-if="showPopover"
          @close="showPopover = false"
          @execute="handleExecute"
        />
      </div>
      <div class="toolbar-divider" />
      <button @click="$emit('refresh')" :disabled="busy" class="toolbar-btn" :title="t('toolbar.refresh')">
        <RefreshCw :size="15" :class="{ 'spin': busy }" />
      </button>
    </div>

    <!-- 中间：视图切换 -->
    <div class="toolbar-center">
      <button
        v-for="item in navItems"
        :key="item.view"
        class="view-tab"
        :class="{ active: activeView === item.view }"
        @click="$emit('switchView', item.view)"
        :title="`${item.label} (${item.shortcut})`"
      >
        <component :is="item.icon" :size="14" />
        <span>{{ item.label }}</span>
      </button>
    </div>

    <!-- 右侧：状态 + 设置 -->
    <div class="toolbar-section">
      <div class="status-indicator" :class="{ busy }">
        <div class="status-dot" />
        <span class="status-text">{{ busy ? t('toolbar.processing') : t('toolbar.ready') }}</span>
      </div>
      <div class="toolbar-divider" />
      <button class="toolbar-btn" @click="$emit('openSettings')" :title="t('globalTabBar.settings')">
        <Settings :size="15" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Download, RefreshCw, Brush, ChevronDown, Settings, ClipboardList, FileEdit, FolderTree, Package } from 'lucide-vue-next'
import CleanupPopover from './CleanupPopover.vue'
import type { ActiveView } from '../types/svn'
import { t } from '../locales'

defineProps<{
  busy: boolean
  activeView: ActiveView
}>()

const emit = defineEmits<{
  pull: []
  refresh: []
  cleanup: []
  cleanupOptions: []
  switchView: [view: ActiveView]
  openSettings: []
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

const navItems = computed(() => [
  { view: 'log' as ActiveView, icon: ClipboardList, label: t('tabs.log'), shortcut: 'Alt+1' },
  { view: 'localChanges' as ActiveView, icon: FileEdit, label: t('tabs.localChanges'), shortcut: 'Alt+2' },
  { view: 'fileBrowser' as ActiveView, icon: FolderTree, label: t('tabs.fileBrowser'), shortcut: 'Alt+3' },
  { view: 'shelve' as ActiveView, icon: Package, label: t('tabs.shelve'), shortcut: 'Alt+4' },
])
</script>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  background: var(--color-bg-primary);
  border-bottom: 1px solid var(--color-border);
  padding: 0 var(--space-3);
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.toolbar-center {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--color-bg-secondary);
  padding: var(--space-1);
  border-radius: var(--radius-md);
}

.view-tab {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  height: 24px;
  padding: 0 var(--space-2);
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--text-xs);
  font-weight: 500;
  white-space: nowrap;
  transition: all var(--transition-fast);
}

.view-tab:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-hover);
}

.view-tab.active {
  color: var(--color-accent);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-sm);
}

.toolbar-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-1);
  height: 28px;
  padding: 0 var(--space-2);
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-sm);
  transition: all var(--transition-fast);
}

.toolbar-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.toolbar-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.toolbar-btn .btn-label {
  font-size: var(--text-sm);
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: var(--color-border);
  margin: 0 var(--space-1);
}

.dropdown-trigger {
  padding: 0 var(--space-1);
  width: 24px;
}

.cleanup-dropdown-wrapper {
  position: relative;
}

/* 状态指示器 */
.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-2);
  height: 28px;
  border-radius: var(--radius-md);
  font-size: var(--text-xs);
  color: var(--color-text-muted);
  transition: all var(--transition-fast);
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--color-success);
  flex-shrink: 0;
  transition: background var(--transition-fast);
}

.status-indicator.busy .status-dot {
  background: var(--color-accent);
  animation: pulse 1.2s ease-in-out infinite;
}

.status-indicator.busy {
  color: var(--color-accent);
}

.status-text {
  white-space: nowrap;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.85); }
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
