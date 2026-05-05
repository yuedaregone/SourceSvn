<template>
  <div class="app-container" @contextmenu.prevent>
    <GlobalTabBar
      :tabs="tabs"
      :activeTabIndex="activeTabIndex"
      @openSettings="showSettings = true"
      @switchTab="switchTab"
      @closeTab="closeTab"
      @closeOtherTabs="closeOtherTabs"
      @closeTabsToRight="closeTabsToRight"
      @addTab="showAddRepo = true"
    />
    <Toolbar
      v-if="tabs.length > 0"
      :loading="(currentTabStore?.logLoading || currentTabStore?.changesLoading || currentTabStore?.fileBrowserLoading || currentTabStore?.shelvesLoading || false) || cleanupLoading"
      @pull="handlePull"
      @cleanup="handleCleanup"
      @cleanupOptions="handleCleanup"
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
          :repoPath="currentTabStore.repoPath"
          :logEntries="currentTabStore.logEntries"
          :wcRevision="currentTabStore.wcRevision"
          :root="currentTabStore.root"
          :loading="currentTabStore.logLoading"
          @refreshLog="currentTabStore.refreshLog"
        />
        <LocalChangesView
          v-if="currentTabStore && currentTabStore.activeView === 'localChanges'"
          :repoPath="currentTabStore.repoPath"
          :localChanges="currentTabStore.localChanges"
          :loading="currentTabStore.changesLoading"
          @refresh="handleRefresh"
          @refreshLocalChanges="currentTabStore.refreshLocalChanges"
        />
        <FileBrowserView
          v-if="currentTabStore && currentTabStore.activeView === 'fileBrowser'"
          :repoPath="currentTabStore.repoPath"
          :fileTree="currentTabStore.fileTree"
          :loading="currentTabStore.fileBrowserLoading"
          @refreshFileBrowser="currentTabStore.refreshFileBrowser"
          @viewHistory="handleViewHistory"
          @aiReview="handleAiReviewFromBrowser"
        />
        <ShelveView
          v-if="currentTabStore && currentTabStore.activeView === 'shelve'"
          :repoPath="currentTabStore.repoPath"
          :shelves="currentTabStore.shelves"
          :loading="currentTabStore.shelvesLoading"
          @refreshShelves="currentTabStore.refreshShelves"
        />
      </div>
    </div>
    <div class="empty-state" v-else>
      <div class="empty-content">
        <div class="empty-icon">
          <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21 16-3.056-3.056a2.503 2.503 0 0 0-3.536 0l-2.38 2.379-1.144-1.143a2.5 2.5 0 0 0-3.535 0L3 16"/><path d="m16 6-4-4-4 4"/><path d="M12 12h8"/></svg>
        </div>
        <p class="empty-title">SourceSvn</p>
        <p class="empty-hint">{{ t('common.addRepoHint') }}</p>
        <button class="empty-action" @click="showAddRepo = true">
          {{ t('globalTabBar.addRepo') }}
        </button>
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
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
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
import type { UpdateResult, SvnUpdateEvent } from './types/svn'
import { t } from './locales'

type TabStoreInstance = ReturnType<ReturnType<typeof useTabStore>>

const configStore = useConfigStore()
const tabs = ref<TabInfo[]>([])
const activeTabIndex = ref(0)
const showSettings = ref(false)
const showAddRepo = ref(false)
const tabStores = ref<Record<string, TabStoreInstance>>({})
let tabIdCounter = 0
const showDiff = ref(false)
const diffFilePath = ref('')
const diffText = ref('')
const showAiReview = ref(false)
const aiReviewContent = ref('')
const aiReviewLoading = ref(false)
const showPullResult = ref(false)
const pullResult = ref<UpdateResult | null>(null)
const cleanupLoading = ref(false)

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

watch(theme, (val) => {
  document.documentElement.setAttribute('data-theme', val)
}, { immediate: true })

function getOrCreateTabStore(tab: TabInfo) {
  if (!tabStores.value[tab.id]) {
    const store = useTabStore(tab.id)()
    store.repoPath = tab.repoPath
    store.activeView = tab.activeView
    tabStores.value[tab.id] = store
  }
  return tabStores.value[tab.id]
}

const currentTab = computed(() => {
  if (tabs.value.length === 0) return null
  return tabs.value[activeTabIndex.value] ?? null
})

const currentTabStore = computed(() => {
  const tab = currentTab.value
  if (!tab) return null
  return tabStores.value[tab.id] ?? null
})

watch(currentTab, (tab) => {
  if (tab) getOrCreateTabStore(tab)
}, { immediate: true })

const handleVisibilityChange = () => {
  if (document.visibilityState === 'visible') {
    refreshCurrentView()
  }
}

let tauriCloseUnlisten: (() => void) | null = null

onMounted(async () => {
  await configStore.loadConfig()
  const config = configStore.config
  if (config?.session.openTabs) {
    tabs.value = config.session.openTabs.map((tab, i) => ({
      ...tab,
      id: tab.id || `tab-${i}`,
    }))
    tabIdCounter = tabs.value.length
    activeTabIndex.value = config.session.activeTabIndex || 0
  }
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('visibilitychange', handleVisibilityChange)
  const { listen } = await import('@tauri-apps/api/event')
  tauriCloseUnlisten = await listen('tauri://close-requested', async () => {
    await saveSession()
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    getCurrentWindow().destroy()
  })
  startAutoRefresh()
  refreshCurrentView()
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  tauriCloseUnlisten?.()
  stopAutoRefresh()
})

let autoRefreshTimer: ReturnType<typeof setInterval> | null = null

function startAutoRefresh() {
  stopAutoRefresh()
  const secs = configStore.config?.behavior.autoRefreshSecs
  if (secs && secs > 0) {
    autoRefreshTimer = setInterval(() => {
      const store = currentTabStore.value
      if (tabs.value.length > 0 && store && !store.logLoading && !store.changesLoading && !store.fileBrowserLoading && !store.shelvesLoading) {
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

watch(() => configStore.config?.behavior.autoRefreshSecs, () => {
  startAutoRefresh()
})

async function openRepo(path: string) {
  showAddRepo.value = false
  let resolvedPath = path.replace(/[\\\/]+$/, '')
  try {
    resolvedPath = await invoke<string>('find_svn_root', { path: resolvedPath })
  } catch (e) {
    useToastStore().error(String(e))
    return
  }
  const id = `tab-${++tabIdCounter}`
  tabs.value.push({ id, repoPath: resolvedPath, activeView: 'log' })
  activeTabIndex.value = tabs.value.length - 1
  addRecentRepo(resolvedPath)
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
  const tab = tabs.value[index]
  if (tab) {
    const store = tabStores.value[tab.id]
    if (store) {
      store.$dispose()
      delete tabStores.value[tab.id]
    }
  }
  tabs.value.splice(index, 1)
  if (activeTabIndex.value >= tabs.value.length) {
    activeTabIndex.value = Math.max(0, tabs.value.length - 1)
  }
  saveSession()
}

function closeOtherTabs(keepIndex: number) {
  const keepTab = tabs.value[keepIndex]
  if (!keepTab) return
  for (const tab of tabs.value) {
    if (tab.id !== keepTab.id) {
      const store = tabStores.value[tab.id]
      if (store) {
        store.$dispose()
        delete tabStores.value[tab.id]
      }
    }
  }
  tabs.value = [keepTab]
  activeTabIndex.value = 0
  saveSession()
}

function closeTabsToRight(fromIndex: number) {
  const toClose = tabs.value.slice(fromIndex + 1)
  for (const tab of toClose) {
    const store = tabStores.value[tab.id]
    if (store) {
      store.$dispose()
      delete tabStores.value[tab.id]
    }
  }
  tabs.value = tabs.value.slice(0, fromIndex + 1)
  if (activeTabIndex.value >= tabs.value.length) {
    activeTabIndex.value = tabs.value.length - 1
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
  const store = currentTabStore.value
  if (!store) return
  const view = store.activeView
  const refreshMap: Record<ActiveView, () => void> = {
    log: () => store.refreshLog(),
    localChanges: () => store.refreshLocalChanges(),
    fileBrowser: () => store.refreshFileBrowser(),
    shelve: () => store.refreshShelves(),
  }
  refreshMap[view]?.()
}

async function handlePull() {
  if (!currentTabStore.value) return

  pullResult.value = { revision: 0, files: [] }
  showPullResult.value = true

  try {
    const { listen } = await import('@tauri-apps/api/event')
    let unlisten: (() => void) | null = null

    unlisten = await listen<SvnUpdateEvent>('svn_update_progress', (event) => {
      const ev = event.payload
      switch (ev.type) {
        case 'file':
          pullResult.value = {
            revision: pullResult.value?.revision ?? 0,
            files: [...(pullResult.value?.files ?? []), { path: ev.path, status: ev.status as 'A' | 'U' | 'M' | 'C', author: '' }],
          }
          break
        case 'done':
          pullResult.value = { ...pullResult.value!, revision: ev.revision }
          unlisten?.()
          refreshCurrentView()
          break
        case 'error':
          showPullResult.value = false
          useToastStore().error(t('common.pullFailed') + ': ' + ev.message)
          unlisten?.()
          break
        case 'upToDate':
          showPullResult.value = false
          useToastStore().info(t('common.upToDate'))
          unlisten?.()
          refreshCurrentView()
          break
      }
    })

    await invoke('svn_update', { path: currentTabStore.value.repoPath })
  } catch (e) {
    showPullResult.value = false
    useToastStore().error(t('common.pullFailed') + ': ' + (e as Error).message)
  }
}

function hasDestructiveCleanupFlags(cfg: { removeUnversionedTrees: boolean; removeIgnoredTrees: boolean }): boolean {
  return cfg.removeUnversionedTrees || cfg.removeIgnoredTrees
}

function cleanupConfigToArgs(cfg: { vacuumPristines: boolean; vacuumPrunables: boolean; includeExternals: boolean; removeUnversionedTrees: boolean; removeIgnoredTrees: boolean; dropDavCache: boolean }): string[] {
  const args: string[] = []
  if (cfg.vacuumPristines) args.push('--vacuum-pristines')
  if (cfg.vacuumPrunables) args.push('--vacuum-prunables')
  if (cfg.includeExternals) args.push('--include-externals')
  if (cfg.removeUnversionedTrees) args.push('--remove-unversioned-trees')
  if (cfg.removeIgnoredTrees) args.push('--remove-ignored-trees')
  if (cfg.dropDavCache) args.push('--drop-dav-cache')
  return args
}

async function handleCleanup() {
  if (!currentTabStore.value || cleanupLoading.value) return
  const cfg = configStore.config?.cleanup
  if (!cfg) return
  if (hasDestructiveCleanupFlags(cfg)) {
    const confirmed = confirm(t('cleanup.destructiveConfirm'))
    if (!confirmed) return
  }
  cleanupLoading.value = true
  try {
    await configStore.saveConfig()
    const options = cleanupConfigToArgs(cfg)
    await invoke<string>('svn_cleanup', {
      path: currentTabStore.value.repoPath,
      options,
    })
    useToastStore().success(t('cleanup.success'))
    refreshCurrentView()
  } catch (e) {
    useToastStore().error(t('cleanup.failed') + ': ' + (e as Error).message)
  } finally {
    cleanupLoading.value = false
  }
}

function handleRefresh() {
  refreshCurrentView()
}

async function saveSession() {
  if (!configStore.config) return
  configStore.config.session.openTabs = tabs.value
  configStore.config.session.activeTabIndex = activeTabIndex.value
  await configStore.saveConfig()
}

function handleViewHistory(_path: string) {
  if (!currentTabStore.value) return
  switchView('log')
  currentTabStore.value.refreshLog()
}

async function handleAiReviewFromBrowser(path: string) {
  if (aiReviewLoading.value) return
  try {
    const content = await invoke<string>('svn_cat', { path })
    const diff = `--- ${path} (repository)\n+++ ${path} (repository)\n@@ -0,0 +1 @@\n${content}`
    await handleAiReview(diff)
  } catch (e) {
    useToastStore().error(String(e))
  }
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
      aiReviewContent.value += t('common.aiReviewTimeout')
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
    aiReviewContent.value = t('common.aiReviewFailed', { msg })
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
  padding: var(--spacing-md);
}
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);
}
.empty-content {
  text-align: center;
  padding: var(--spacing-xl);
}
.empty-icon {
  width: 80px;
  height: 80px;
  margin: 0 auto var(--spacing-lg);
  border-radius: 50%;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-color);
}
.empty-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: var(--spacing-sm);
}
.empty-hint {
  color: var(--text-muted);
  font-size: 14px;
  margin-bottom: var(--spacing-lg);
}
.empty-action {
  padding: var(--spacing-sm) var(--spacing-xl);
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: background 0.2s;
}
.empty-action:hover {
  background: var(--accent-hover);
}
</style>
