<template>
  <div class="icon-nav-bar">
    <button
      v-for="item in navItems"
      :key="item.view"
      class="nav-item"
      :class="{ active: activeView === item.view }"
      @click="$emit('switchView', item.view)"
      :title="`${item.label} (${item.shortcut})`"
    >
      <component :is="item.icon" :size="18" class="nav-icon" />
      <span class="nav-label">{{ item.label }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ClipboardList, FileEdit, FolderTree, Package } from 'lucide-vue-next'
import type { ActiveView } from '../types/svn'
import { t } from '../locales'

defineProps<{
  activeView: ActiveView
}>()

defineEmits<{
  switchView: [view: ActiveView]
}>()

const navItems = computed(() => [
  { view: 'log' as ActiveView, icon: ClipboardList, label: t('tabs.log'), shortcut: 'Ctrl+1' },
  { view: 'localChanges' as ActiveView, icon: FileEdit, label: t('tabs.localChanges'), shortcut: 'Ctrl+2' },
  { view: 'fileBrowser' as ActiveView, icon: FolderTree, label: t('tabs.fileBrowser'), shortcut: 'Ctrl+3' },
  { view: 'shelve' as ActiveView, icon: Package, label: t('tabs.shelve'), shortcut: 'Ctrl+4' },
])
</script>

<style scoped>
.icon-nav-bar {
  display: flex;
  flex-direction: column;
  width: 56px;
  background: var(--color-bg-secondary);
  border-right: 1px solid var(--color-border);
  padding: var(--space-2) var(--space-1);
  gap: var(--space-1);
}

.nav-item {
  width: 100%;
  height: 44px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  position: relative;
  transition: all var(--transition-fast);
  color: var(--color-text-muted);
  border-radius: var(--radius-md);
}

.nav-item:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.nav-item.active {
  background: var(--color-accent-muted);
  color: var(--color-accent);
}

.nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 24px;
  background: var(--color-accent);
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
}

.nav-icon {
  flex-shrink: 0;
}

.nav-label {
  font-size: 10px;
  font-weight: 500;
  line-height: 1;
  white-space: nowrap;
}
</style>
