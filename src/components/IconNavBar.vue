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
      <component :is="item.icon" :size="20" class="nav-icon" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ClipboardList, FileEdit, FolderTree, Package } from 'lucide-vue-next'
import type { ActiveView } from '../types/svn'
import { t } from '../locales'

defineProps<{
  activeView: ActiveView
}>()

defineEmits<{
  switchView: [view: ActiveView]
}>()

const navItems = [
  { view: 'log' as ActiveView, icon: ClipboardList, label: t('tabs.log'), shortcut: 'Ctrl+1' },
  { view: 'localChanges' as ActiveView, icon: FileEdit, label: t('tabs.localChanges'), shortcut: 'Ctrl+2' },
  { view: 'fileBrowser' as ActiveView, icon: FolderTree, label: t('tabs.fileBrowser'), shortcut: 'Ctrl+3' },
  { view: 'shelve' as ActiveView, icon: Package, label: t('tabs.shelve'), shortcut: 'Ctrl+4' },
]
</script>

<style scoped>
.icon-nav-bar {
  display: flex;
  flex-direction: column;
  width: 48px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  padding: var(--spacing-sm) 0;
}
.nav-item {
  width: 48px;
  height: 48px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: all 0.2s ease;
  color: var(--text-secondary);
  border-radius: 8px;
}
.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.nav-item.active {
  background: var(--bg-active);
  color: var(--accent-color);
  box-shadow: inset 0 0 0 1px var(--accent-color);
}
.nav-icon {
  flex-shrink: 0;
}
</style>
