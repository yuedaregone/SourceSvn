import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FileStatus, LogEntry, DirEntry, ShelveInfo, ActiveView, WcLogResult } from '../types/svn'
import { useToastStore } from './toastStore'
import { t } from '../locales'

export const useTabStore = (id: string) => defineStore(`tab-${id}`, () => {
  const repoPath = ref('')
  const activeView = ref<ActiveView>('log')
  const logEntries = ref<LogEntry[]>([])
  const wcRevision = ref(0)
  const root = ref('')
  const fileTree = ref<DirEntry[]>([])
  const localChanges = ref<FileStatus[]>([])
  const shelves = ref<ShelveInfo[]>([])
  const logPage = ref(1)
  const hasMoreLogs = ref(true)
  const logLoading = ref(false)
  const changesLoading = ref(false)
  const fileBrowserLoading = ref(false)
  const shelvesLoading = ref(false)
  const commitHistory = ref<string[]>([])

  const STORAGE_KEY = 'svn-commit-history'

  function loadCommitHistory() {
    try {
      const all = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}')
      commitHistory.value = all[repoPath.value] || []
    } catch {
      commitHistory.value = []
    }
  }

  function saveCommitHistory() {
    try {
      const all = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}')
      all[repoPath.value] = commitHistory.value
      localStorage.setItem(STORAGE_KEY, JSON.stringify(all))
    } catch {
      // ignore
    }
  }

  function addCommitMessage(msg: string) {
    if (!msg.trim()) return
    // 去重：如果已存在则移到最前
    const idx = commitHistory.value.indexOf(msg)
    if (idx >= 0) commitHistory.value.splice(idx, 1)
    commitHistory.value.unshift(msg)
    // 最多保留 20 条
    if (commitHistory.value.length > 20) commitHistory.value.pop()
    saveCommitHistory()
  }

  async function refreshLog(limit?: number) {
    logLoading.value = true
    try {
      const result = await invoke<WcLogResult>('svn_log_server', {
        path: repoPath.value,
        limit: limit ?? 100,
      })
      logEntries.value = result.entries
      wcRevision.value = result.wcRevision
      root.value = result.root
    } catch (e) {
      try {
        logEntries.value = await invoke<LogEntry[]>('svn_log', {
          path: repoPath.value,
          limit: limit ?? 100,
        })
        console.log('[refreshLog] svn_log fallback returned:', logEntries.value.length, 'entries')
      } catch (e2) {
        console.error('Failed to refresh log (fallback):', e2)
        useToastStore().error(t('toast.refreshFailed') + ': ' + (e2 as Error).message)
      }
    } finally {
      logLoading.value = false
    }
  }

  async function refreshLocalChanges() {
    changesLoading.value = true
    try {
      localChanges.value = await invoke<FileStatus[]>('svn_status', {
        path: repoPath.value,
      })
    } catch (e) {
      console.error('Failed to refresh local changes:', e)
      useToastStore().error(t('toast.refreshFailed') + ': ' + (e as Error).message)
    } finally {
      changesLoading.value = false
    }
  }

  async function refreshFileBrowser(path?: string) {
    fileBrowserLoading.value = true
    try {
      fileTree.value = await invoke<DirEntry[]>('svn_list', {
        path: path ?? repoPath.value,
        recursive: false,
      })
    } catch (e) {
      console.error('Failed to refresh file browser:', e)
      useToastStore().error(t('toast.refreshFailed') + ': ' + (e as Error).message)
    } finally {
      fileBrowserLoading.value = false
    }
  }

  async function refreshShelves() {
    shelvesLoading.value = true
    try {
      shelves.value = await invoke<ShelveInfo[]>('shelve_list', {
        path: repoPath.value,
      })
    } catch (e) {
      console.error('Failed to refresh shelves:', e)
      useToastStore().error(t('toast.refreshFailed') + ': ' + (e as Error).message)
    } finally {
      shelvesLoading.value = false
    }
  }

  return {
    repoPath,
    activeView,
    logEntries,
    wcRevision,
    root,
    fileTree,
    localChanges,
    shelves,
    logPage,
    hasMoreLogs,
    logLoading,
    changesLoading,
    fileBrowserLoading,
    shelvesLoading,
    commitHistory,
    refreshLog,
    refreshLocalChanges,
    refreshFileBrowser,
    refreshShelves,
    loadCommitHistory,
    addCommitMessage,
  }
})
