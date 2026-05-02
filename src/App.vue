<template>
  <div class="app-container" :data-theme="theme">
    <GlobalTabBar
      :tabs="tabs"
      :activeTabIndex="activeTabIndex"
      @openSettings="showSettings = true"
      @switchTab="switchTab"
      @closeTab="closeTab"
      @addTab="showAddRepo = true"
    />
    <Toolbar
      v-if="tabs.length > 0"
      :loading="currentTabStore?.loading ?? false"
      @pull="handlePull"
      @refresh="handleRefresh"
    />
    <div class="main-content" v-if="tabs.length > 0">
      <IconNavBar
        :activeView="currentTabStore?.activeView ?? 'log'"
        @switchView="switchView"
      />
      <div class="view-area">
        <LogView
          v-if="currentTabStore && currentTabStore.activeView === 'log'"
          :store="currentTabStore"
          @viewDiff="handleViewDiff"
          @aiReview="handleAiReviewRevision"
        />
        <LocalChangesView
          v-if="currentTabStore && currentTabStore.activeView === 'localChanges'"
          :store="currentTabStore"
          @refresh="handleRefresh"
        />
        <FileBrowserView
          v-if="currentTabStore && currentTabStore.activeView === 'fileBrowser'"
          :store="currentTabStore"
        />
        <ShelveView
          v-if="currentTabStore && currentTabStore.activeView === 'shelve'"
          :store="currentTabStore"
        />
      </div>
    </div>
    <div class="empty-state" v-else>
      <div class="empty-content">
        <p class="empty-title">SourceSvn</p>
        <p class="empty-hint">点击上方 "+ 新页签" 打开一个仓库</p>
      </div>
    </div>
    <SettingsPage v-if="showSettings" @close="showSettings = false" />
    <AddRepoDialog
      :visible="showAddRepo"
      :recentRepos="recentRepos"
      @close="showAddRepo = false"
      @openRepo="openRepo"
    />
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
    <PullResultModal
      :visible="showPullResult"
      :result="pullResult"
      @close="showPullResult = false"
    />
    <Toast />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import Toast from './components/Toast.vue'
import { invoke } from '@tauri-apps/api/core'
import { useConfigStore } from './stores/configStore'
import { useTabStore } from './stores/tabStore'
import type { TabInfo, RepoEntry } from './types/config'
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
import AddRepoDialog from './components/AddRepoDialog.vue'
import PullResultModal from './components/PullResultModal.vue'
import { useToastStore } from './stores/toastStore'
import type { UpdateResult } from './types/svn'

type TabStoreInstance = ReturnType<ReturnType<typeof useTabStore>>

const configStore = useConfigStore()
const tabs = ref<TabInfo[]>([])
const activeTabIndex = ref(0)
const showSettings = ref(false)
const showAddRepo = ref(false)
const tabStores = ref<Record<string, TabStoreInstance>>({})
const showDiff = ref(false)
const diffFilePath = ref('')
const diffText = ref('')
const showAiReview = ref(false)
const aiReviewContent = ref('')
const aiReviewLoading = ref(false)
const showPullResult = ref(false)
const pullResult = ref<UpdateResult | null>(null)

const recentRepos = computed<RepoEntry[]>(() => {
  return configStore.config?.session.recentRepos ?? []
})

const theme = computed(() => {
  const t = configStore.config?.appearance.theme ?? 'light'
  if (t === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return t
})

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

const handleVisibilityChange = () => {
  if (document.visibilityState === 'visible') {
    refreshCurrentView()
  }
}

onMounted(async () => {
  await configStore.loadConfig()
  const config = configStore.config
  if (config?.session.openTabs) {
    tabs.value = config.session.openTabs
    activeTabIndex.value = config.session.activeTabIndex || 0
  }
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('visibilitychange', handleVisibilityChange)
  window.addEventListener('tauri://close-requested', async () => {
    await saveSession()
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    getCurrentWindow().destroy()
  })
  startAutoRefresh()
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  stopAutoRefresh()
})

let autoRefreshTimer: ReturnType<typeof setInterval> | null = null

function startAutoRefresh() {
  stopAutoRefresh()
  const secs = configStore.config?.behavior.autoRefreshSecs
  if (secs && secs > 0) {
    autoRefreshTimer = setInterval(() => {
      if (tabs.value.length > 0 && !currentTabStore.value?.loading) {
        refreshCurrentView()
      }
    }, secs * 1000)
  }
}

function stopAutoRefresh() {
  if (autoRefreshTimer) {
    clearInterval(autoRefreshTimer)
    autoRefreshTimer = null
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement) return
  const views: ActiveView[] = ['log', 'localChanges', 'fileBrowser', 'shelve']
  if (e.ctrlKey && e.key >= '1' && e.key <= '4') {
    e.preventDefault()
    switchView(views[parseInt(e.key) - 1])
  } else if (e.key === 'F5') {
    e.preventDefault()
    handleRefresh()
  } else if (e.ctrlKey && e.key === 'Enter') {
    e.preventDefault()
    handlePull()
  }
}

function openRepo(path: string) {
  showAddRepo.value = false
  tabs.value.push({ repoPath: path, activeView: 'log' })
  activeTabIndex.value = tabs.value.length - 1
  addRecentRepo(path)
  saveSession()
  refreshCurrentView()
}

function addRecentRepo(path: string) {
  if (!configStore.config) return
  const repos = configStore.config.session.recentRepos
  const existing = repos.findIndex((r) => r.path === path)
  if (existing >= 0) {
    repos[existing].lastOpened = new Date().toISOString()
  } else {
    repos.unshift({ path, lastOpened: new Date().toISOString() })
    if (repos.length > configStore.config.session.maxRecentRepos) {
      repos.pop()
    }
  }
}

function switchTab(index: number) {
  activeTabIndex.value = index
}

function closeTab(index: number) {
  const key = `${index}`
  const store = tabStores.value[key]
  if (store) {
    store.$dispose()
    delete tabStores.value[key]
  }
  tabs.value.splice(index, 1)
  if (activeTabIndex.value >= tabs.value.length) {
    activeTabIndex.value = Math.max(0, tabs.value.length - 1)
  }
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
  const refreshMap: Record<ActiveView, () => void> = {
    log: () => currentTabStore.value!.refreshLog(),
    localChanges: () => currentTabStore.value!.refreshLocalChanges(),
    fileBrowser: () => currentTabStore.value!.refreshFileBrowser(),
    shelve: () => currentTabStore.value!.refreshShelves(),
  }
  refreshMap[view]?.()
}

async function handlePull() {
  if (!currentTabStore.value) return
  try {
    const result = await invoke<UpdateResult>('svn_update', {
      path: currentTabStore.value.repoPath,
    })
    if (result.files.length === 0) {
      useToastStore().info('已是最新版本')
    } else {
      pullResult.value = result
      showPullResult.value = true
    }
    refreshCurrentView()
  } catch (e) {
    console.error('Pull failed:', e)
    useToastStore().error('拉取失败')
  }
}

function handleRefresh() {
  refreshCurrentView()
}

function handleViewDiff(revision: number) {
  if (!currentTabStore.value) return
  // Fetch diff for the revision's changed paths
  const entry = currentTabStore.value.logEntries.find((e) => e.revision === revision)
  if (!entry?.changedPaths?.length) return
  const firstPath = entry.changedPaths[0].path
  diffFilePath.value = firstPath
  invoke<string>('svn_diff', {
    path: currentTabStore.value.repoPath,
    target: { type: 'File', data: { path: firstPath, revision: String(revision) } },
  })
    .then((d) => {
      diffText.value = d
      showDiff.value = true
    })
    .catch((e) => console.error('Diff failed:', e))
}

function handleAiReviewRevision(revision: number) {
  if (!currentTabStore.value) return
  const entry = currentTabStore.value.logEntries.find((e) => e.revision === revision)
  if (!entry?.changedPaths?.length) return
  const firstPath = entry.changedPaths[0].path
  invoke<string>('svn_diff', {
    path: currentTabStore.value.repoPath,
    target: { type: 'File', data: { path: firstPath, revision: String(revision) } },
  })
    .then((d) => handleAiReview(d))
    .catch((e) => console.error('Diff failed:', e))
}

async function saveSession() {
  if (!configStore.config) return
  configStore.config.session.openTabs = tabs.value
  configStore.config.session.activeTabIndex = activeTabIndex.value
  await configStore.saveConfig()
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
  background: var(--bg-primary);
  color: var(--text-primary);
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
}
.empty-content {
  text-align: center;
}
.empty-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}
.empty-hint {
  color: var(--text-muted);
  font-size: 14px;
}
</style>
