<template>
  <div class="app-container">
    <GlobalTabBar
      :tabs="tabs"
      :activeTabIndex="activeTabIndex"
      @openSettings="showSettings = true"
      @switchTab="switchTab"
      @closeTab="closeTab"
      @addTab="addTab"
    />
    <Toolbar
      v-if="tabs.length > 0"
      :loading="currentTabStore?.loading ?? false"
      @pull="handlePull"
      @commit="handleCommit"
      @refresh="handleRefresh"
    />
    <div class="main-content" v-if="tabs.length > 0">
      <IconNavBar
        :activeView="currentTabStore?.activeView ?? 'log'"
        @switchView="switchView"
      />
      <div class="view-area">
        <LogView v-if="currentTabStore?.activeView === 'log'" :store="currentTabStore!" />
        <LocalChangesView v-if="currentTabStore?.activeView === 'localChanges'" :store="currentTabStore!" />
        <FileBrowserView v-if="currentTabStore?.activeView === 'fileBrowser'" :store="currentTabStore!" />
        <ShelveView v-if="currentTabStore?.activeView === 'shelve'" :store="currentTabStore!" />
      </div>
    </div>
    <div class="empty-state" v-else>
      <p>点击 "+ 新页签" 打开一个仓库</p>
    </div>
    <SettingsPage v-if="showSettings" @close="showSettings = false" />
    <DiffViewer
      :visible="showDiff"
      :filePath="diffFilePath"
      :diffText="diffText"
      @close="showDiff = false"
      @aiReview="handleAiReview"
    />
    <AiReviewPanel
      :visible="showAiReview"
      :content="aiReviewContent"
      :loading="aiReviewLoading"
      @close="showAiReview = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useConfigStore } from './stores/configStore'
import { useTabStore } from './stores/tabStore'
import type { TabInfo } from './types/config'
import type { ActiveView } from './types/svn'
import GlobalTabBar from './components/GlobalTabBar.vue'
import IconNavBar from './components/IconNavBar.vue'
import Toolbar from './components/Toolbar.vue'
import LogView from './views/LogView.vue'
import LocalChangesView from './views/LocalChangesView.vue'
import FileBrowserView from './views/FileBrowserView.vue'
import ShelveView from './views/ShelveView.vue'
import SettingsPage from './views/SettingsPage.vue'
import DiffViewer from './components/DiffViewer.vue'
import AiReviewPanel from './components/AiReviewPanel.vue'

type TabStoreInstance = ReturnType<ReturnType<typeof useTabStore>>

const configStore = useConfigStore()
const tabs = ref<TabInfo[]>([])
const activeTabIndex = ref(0)
const showSettings = ref(false)
const tabStores = ref<Record<string, TabStoreInstance>>({})
const showDiff = ref(false)
const diffFilePath = ref('')
const diffText = ref('')
const showAiReview = ref(false)
const aiReviewContent = ref('')
const aiReviewLoading = ref(false)

const currentTabStore = computed(() => {
  if (tabs.value.length === 0) return null
  const tab = tabs.value[activeTabIndex.value]
  if (!tab) return null
  const key = `${activeTabIndex.value}`
  if (!tabStores.value[key]) {
    const store = useTabStore(key)()
    store.repoPath = tab.repoPath
    store.activeView = tab.activeView
    tabStores.value[key] = store
  }
  return tabStores.value[key]
})

onMounted(async () => {
  await configStore.loadConfig()
  const config = configStore.config
  if (config?.session.openTabs) {
    tabs.value = config.session.openTabs
    activeTabIndex.value = config.session.activeTabIndex || 0
  }
})

function switchTab(index: number) {
  activeTabIndex.value = index
}

function closeTab(index: number) {
  const key = `${index}`
  if (tabStores.value[key]) {
    delete tabStores.value[key]
  }
  tabs.value.splice(index, 1)
  if (activeTabIndex.value >= tabs.value.length) {
    activeTabIndex.value = Math.max(0, tabs.value.length - 1)
  }
  saveSession()
}

function addTab() {
  const path = prompt('请输入仓库工作副本路径:')
  if (!path) return
  tabs.value.push({ repoPath: path, activeView: 'log' })
  activeTabIndex.value = tabs.value.length - 1
  saveSession()
}

function switchView(view: ActiveView) {
  if (!currentTabStore.value) return
  currentTabStore.value.activeView = view
  const tab = tabs.value[activeTabIndex.value]
  if (tab) {
    tab.activeView = view
    saveSession()
  }
  refreshCurrentView()
}

function refreshCurrentView() {
  if (!currentTabStore.value) return
  const view = currentTabStore.value.activeView
  if (view === 'log') currentTabStore.value.refreshLog()
  else if (view === 'localChanges') currentTabStore.value.refreshLocalChanges()
  else if (view === 'fileBrowser') currentTabStore.value.refreshFileBrowser()
  else if (view === 'shelve') currentTabStore.value.refreshShelves()
}

function handlePull() {
  if (!currentTabStore.value) return
  invoke('svn_update', { path: currentTabStore.value.repoPath })
    .then(() => refreshCurrentView())
    .catch((e) => console.error('Pull failed:', e))
}

function handleCommit() {
  // TODO: open commit dialog
}

function handleRefresh() {
  refreshCurrentView()
}

function saveSession() {
  if (!configStore.config) return
  configStore.config.session.openTabs = tabs.value
  configStore.config.session.activeTabIndex = activeTabIndex.value
  configStore.saveConfig()
}

async function handleAiReview(diff: string) {
  if (aiReviewLoading.value) return
  showAiReview.value = true
  aiReviewContent.value = ''
  aiReviewLoading.value = true
  try {
    const { listen } = await import('@tauri-apps/api/event')
    let unlisten: (() => void) | null = null
    const timeout = setTimeout(() => {
      aiReviewContent.value += '\n\n[AI 审查超时]'
      aiReviewLoading.value = false
      unlisten?.()
    }, 120000)
    unlisten = await listen<{ content: string; done: boolean }>('review_chunk', (event) => {
      aiReviewContent.value += event.payload.content
      if (event.payload.done) {
        clearTimeout(timeout)
        aiReviewLoading.value = false
        unlisten?.()
      }
    })
    await invoke('review_changes', { diff })
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    aiReviewContent.value = `AI 审查失败: ${msg}`
    aiReviewLoading.value = false
  }
}
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.view-area {
  flex: 1;
  overflow: auto;
  padding: 12px;
}
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
}
</style>
