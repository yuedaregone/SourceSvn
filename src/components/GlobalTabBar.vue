<template>
  <div class="tab-bar">
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
        <button class="tab-close" @click.stop="$emit('closeTab', index)">
          <X :size="12" />
        </button>
      </div>
      <button class="add-tab-btn" @click="$emit('addTab')" :title="t('globalTabBar.addRepo')">
        <Plus :size="14" />
      </button>
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
import { Plus, Copy, X } from 'lucide-vue-next'
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
  height: 38px;
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
  padding: 0 var(--space-2);
  gap: var(--space-1);
}

.tabs {
  display: flex;
  flex: 1;
  overflow-x: auto;
  gap: var(--space-1);
  padding: var(--space-1) 0;
}

.tab {
  display: flex;
  align-items: center;
  padding: var(--space-1) var(--space-3);
  background: var(--color-bg-tertiary);
  border-radius: var(--radius-md) var(--radius-md) 0 0;
  cursor: pointer;
  white-space: nowrap;
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
  gap: var(--space-2);
}

.tab:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.tab.active {
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  border-bottom: 2px solid var(--color-accent);
}

.tab-close {
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  opacity: 0;
}

.tab:hover .tab-close {
  opacity: 1;
}

.tab-close:hover {
  color: var(--color-danger);
  background: var(--color-danger-muted);
}

.add-tab-btn {
  border: 1px dashed var(--color-border-input);
  background: transparent;
  width: 28px;
  height: 28px;
  padding: 0;
  cursor: pointer;
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
}

.add-tab-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-accent);
  color: var(--color-accent);
}
</style>
