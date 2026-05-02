import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { FileStatus, LogEntry, DirEntry, ShelveInfo, ActiveView, WcLogResult } from '../types/svn'

export const useTabStore = (id: string) =>
  defineStore(`tab-${id}`, {
    state: () => ({
      repoPath: '',
      activeView: 'log' as ActiveView,
      logEntries: [] as LogEntry[],
      wcRevision: 0,
      fileTree: [] as DirEntry[],
      localChanges: [] as FileStatus[],
      shelves: [] as ShelveInfo[],
      logPage: 1,
      hasMoreLogs: true,
      loading: false,
    }),
    actions: {
      async refreshLog(limit?: number) {
        this.loading = true
        try {
          const result = await invoke<WcLogResult>('svn_log_server', {
            path: this.repoPath,
            limit: limit ?? 100,
          })
          this.logEntries = result.entries
          this.wcRevision = result.wcRevision
        } catch (e) {
          console.error('Failed to refresh log:', e)
        } finally {
          this.loading = false
        }
      },
      async refreshLocalChanges() {
        this.loading = true
        try {
          this.localChanges = await invoke<FileStatus[]>('svn_status', {
            path: this.repoPath,
          })
        } catch (e) {
          console.error('Failed to refresh local changes:', e)
        } finally {
          this.loading = false
        }
      },
      async refreshFileBrowser(path?: string) {
        this.loading = true
        try {
          this.fileTree = await invoke<DirEntry[]>('svn_list', {
            path: path ?? this.repoPath,
            recursive: false,
          })
        } catch (e) {
          console.error('Failed to refresh file browser:', e)
        } finally {
          this.loading = false
        }
      },
      async refreshShelves() {
        this.loading = true
        try {
          this.shelves = await invoke<ShelveInfo[]>('shelve_list', {
            path: this.repoPath,
          })
        } catch (e) {
          console.error('Failed to refresh shelves:', e)
        } finally {
          this.loading = false
        }
      },
    },
  })
