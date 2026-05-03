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
  const fileTree = ref<DirEntry[]>([])
  const localChanges = ref<FileStatus[]>([])
  const shelves = ref<ShelveInfo[]>([])
  const logPage = ref(1)
  const hasMoreLogs = ref(true)
  const loading = ref(false)

  async function refreshLog(limit?: number) {
    loading.value = true
    try {
      const result = await invoke<WcLogResult>('svn_log_server', {
        path: repoPath.value,
        limit: limit ?? 100,
      })
      logEntries.value = result.entries
      wcRevision.value = result.wcRevision
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
      loading.value = false
    }
  }

  async function refreshLocalChanges() {
    loading.value = true
    try {
      localChanges.value = await invoke<FileStatus[]>('svn_status', {
        path: repoPath.value,
      })
    } catch (e) {
      console.error('Failed to refresh local changes:', e)
      useToastStore().error(t('toast.refreshFailed') + ': ' + (e as Error).message)
    } finally {
      loading.value = false
    }
  }

  async function refreshFileBrowser(path?: string) {
    loading.value = true
    try {
      fileTree.value = await invoke<DirEntry[]>('svn_list', {
        path: path ?? repoPath.value,
        recursive: false,
      })
    } catch (e) {
      console.error('Failed to refresh file browser:', e)
      useToastStore().error(t('toast.refreshFailed') + ': ' + (e as Error).message)
    } finally {
      loading.value = false
    }
  }

  async function refreshShelves() {
    loading.value = true
    try {
      shelves.value = await invoke<ShelveInfo[]>('shelve_list', {
        path: repoPath.value,
      })
    } catch (e) {
      console.error('Failed to refresh shelves:', e)
      useToastStore().error(t('toast.refreshFailed') + ': ' + (e as Error).message)
    } finally {
      loading.value = false
    }
  }

  return {
    repoPath,
    activeView,
    logEntries,
    wcRevision,
    fileTree,
    localChanges,
    shelves,
    logPage,
    hasMoreLogs,
    loading,
    refreshLog,
    refreshLocalChanges,
    refreshFileBrowser,
    refreshShelves,
  }
})
