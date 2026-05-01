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
      <span class="nav-icon">{{ item.icon }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { ActiveView } from '../types/svn'

defineProps<{
  activeView: ActiveView
}>()

defineEmits<{
  switchView: [view: ActiveView]
}>()

const navItems = [
  { view: 'log' as ActiveView, icon: '📋', label: '日志', shortcut: 'Ctrl+1' },
  { view: 'localChanges' as ActiveView, icon: '📝', label: '本地修改', shortcut: 'Ctrl+2' },
  { view: 'fileBrowser' as ActiveView, icon: '📂', label: '文件浏览', shortcut: 'Ctrl+3' },
  { view: 'shelve' as ActiveView, icon: '📦', label: 'Shelve', shortcut: 'Ctrl+4' },
]
</script>

<style scoped>
.icon-nav-bar {
  display: flex;
  flex-direction: column;
  width: 48px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  padding: 8px 0;
}
.nav-item {
  width: 48px;
  height: 48px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: background 0.15s;
}
.nav-item:hover {
  background: var(--bg-hover);
}
.nav-item.active {
  background: var(--bg-active);
  border-left: 3px solid var(--accent-color);
}
</style>
