<template>
  <div class="tab-bar">
    <button class="settings-btn" @click="$emit('openSettings')" :title="t('globalTabBar.settings')">
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
        @contextmenu.prevent="openContextMenu($event, index)"
      >
        <span class="tab-title">{{ getTabTitle(tab.repoPath) }}</span>
        <button class="tab-close" @click.stop="$emit('closeTab', index)">&times;</button>
      </div>
      <button class="add-tab-btn" @click="$emit('addTab')" :title="t('globalTabBar.addRepo')"><Plus :size="16" /></button>
    </div>
    <ContextMenu
      :visible="ctxMenu.visible"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      :items="ctxMenuItems"
      @close="ctxMenu.visible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Settings, Plus, Copy } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import type { TabInfo } from '../types/config'
import type { MenuItem } from './ContextMenu.vue'
import ContextMenu from './ContextMenu.vue'
import { t } from '../locales'

const props = defineProps<{
  tabs: TabInfo[]
  activeTabIndex: number
}>()

const emit = defineEmits<{
  openSettings: []
  switchTab: [index: number]
  closeTab: [index: number]
  closeOtherTabs: [index: number]
  closeTabsToRight: [index: number]
  addTab: []
}>()

function getTabTitle(path: string) {
  return path.split(/[/\\]/).pop() || path
}

const ctxMenu = ref({ visible: false, x: 0, y: 0, tabIndex: -1 })

function openContextMenu(e: MouseEvent, index: number) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, tabIndex: index }
}

const ctxMenuItems = computed<MenuItem[]>(() => {
  const { tabIndex } = ctxMenu.value
  if (tabIndex < 0 || tabIndex >= props.tabs.length) return []
  const tab = props.tabs[tabIndex]
  const toast = useToastStore()

  return [
    {
      label: t('contextMenu.closeTab'),
      action: () => { emit('closeTab', tabIndex) },
    },
    {
      label: t('contextMenu.closeOtherTabs'),
      disabled: props.tabs.length <= 1,
      action: () => { emit('closeOtherTabs', tabIndex) },
    },
    {
      label: t('contextMenu.closeTabsToRight'),
      disabled: tabIndex >= props.tabs.length - 1,
      action: () => { emit('closeTabsToRight', tabIndex) },
    },
    { divider: true },
    {
      label: t('contextMenu.copyRepoPath'),
      icon: Copy,
      action: () => {
        navigator.clipboard.writeText(tab.repoPath)
        toast.success(t('contextMenu.copySuccess'))
      },
    },
    {
      label: t('contextMenu.openInExplorer'),
      action: async () => {
        try { await invoke('open_in_system', { path: tab.repoPath }) } catch (e) { toast.error(String(e)) }
      },
    },
  ]
})
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
  width: 28px;
  height: 28px;
  padding: 0;
  cursor: pointer;
  border-radius: 4px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}
.add-tab-btn:hover {
  background: var(--bg-hover);
}
</style>
