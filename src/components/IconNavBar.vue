<template>
  <div class="icon-nav-bar">
    <button
      v-for="item in navItems"
      :key="item.view"
      class="nav-item"
      :class="{ active: activeView === item.view }"
      @click="$emit('switchView', item.view)"
      :title="item.label"
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
  { view: 'log' as ActiveView, icon: '📋', label: '日志' },
  { view: 'localChanges' as ActiveView, icon: '📝', label: '本地修改' },
  { view: 'fileBrowser' as ActiveView, icon: '📂', label: '文件浏览' },
  { view: 'shelve' as ActiveView, icon: '📦', label: 'Shelve' },
]
</script>

<style scoped>
.icon-nav-bar {
  display: flex;
  flex-direction: column;
  width: 48px;
  background: #fafafa;
  border-right: 1px solid #e8e8e8;
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
}
.nav-item:hover {
  background: #f0f0f0;
}
.nav-item.active {
  background: #e6f7ff;
  border-left: 3px solid #1890ff;
}
</style>
