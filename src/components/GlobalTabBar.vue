<template>
  <div class="tab-bar">
    <button class="settings-btn" @click="$emit('openSettings')" title="设置">⚙</button>
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
        <button class="tab-close" @click.stop="$emit('closeTab', index)">×</button>
      </div>
    </div>
    <button class="add-tab-btn" @click="$emit('addTab')" title="打开仓库">+ 新页签</button>
  </div>
</template>

<script setup lang="ts">
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
  background: #f0f0f0;
  border-bottom: 1px solid #ddd;
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
}
.settings-btn:hover {
  background: #e0e0e0;
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
  background: #e8e8e8;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  white-space: nowrap;
  font-size: 12px;
}
.tab.active {
  background: #fff;
  border-bottom: 2px solid #1890ff;
}
.tab-close {
  margin-left: 4px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  color: #999;
}
.tab-close:hover {
  color: #333;
}
.add-tab-btn {
  border: 1px dashed #999;
  background: transparent;
  padding: 4px 8px;
  cursor: pointer;
  font-size: 12px;
  border-radius: 4px;
}
.add-tab-btn:hover {
  background: #e0e0e0;
}
</style>
