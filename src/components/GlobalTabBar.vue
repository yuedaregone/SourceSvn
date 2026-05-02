<template>
  <div class="tab-bar">
    <button class="settings-btn" @click="$emit('openSettings')" title="设置">
      <Settings :size="16" />
    </button>
    <div class="tabs">
      <div
        v-for="(tab, index) in tabs"
        :key="index"
        class="tab"
        :class="{ active: index === activeTabIndex }"
        @click="$emit('switchTab', index)"
        @dblclick="$emit('closeTab', index)"
      >
        <span class="tab-title">{{ getTabTitle(tab.repoPath) }}</span>
        <button class="tab-close" @click.stop="$emit('closeTab', index)">&times;</button>
      </div>
    </div>
    <button class="add-tab-btn" @click="$emit('addTab')" title="打开仓库">+ 新页签</button>
  </div>
</template>

<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import type { TabInfo } from '../types/config'

defineProps<{
  tabs: TabInfo[]
  activeTabIndex: number
}>()

defineEmits<{
  openSettings: []
  switchTab: [index: number]
  closeTab: [index: number]
  addTab: []
}>()

function getTabTitle(path: string) {
  return path.split(/[/\\]/).pop() || path
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  height: 36px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 4px;
  gap: 2px;
}
.settings-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 16px;
  border-radius: 4px;
  color: var(--text-primary);
}
.settings-btn:hover {
  background: var(--bg-hover);
}
.tabs {
  display: flex;
  flex: 1;
  overflow-x: auto;
  gap: 2px;
}
.tab {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  white-space: nowrap;
  font-size: 12px;
  color: var(--text-secondary);
}
.tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent-color);
}
.tab-close {
  margin-left: 4px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-muted);
}
.tab-close:hover {
  color: var(--text-primary);
}
.add-tab-btn {
  border: 1px dashed var(--border-input);
  background: transparent;
  padding: 4px 8px;
  cursor: pointer;
  font-size: 12px;
  border-radius: 4px;
  color: var(--text-secondary);
}
.add-tab-btn:hover {
  background: var(--bg-hover);
}
</style>
