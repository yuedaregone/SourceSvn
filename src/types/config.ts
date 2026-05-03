export interface AppConfig {
  configVersion: number
  window: WindowConfig
  appearance: AppearanceConfig
  session: SessionConfig
  svn: SvnConfig
  ai: AiConfig
  diff: DiffConfig
  log: LogConfig
  commit: CommitConfig
  fileBrowser: FileBrowserConfig
  behavior: BehaviorConfig
  advanced: AdvancedConfig
  cleanup: CleanupConfig
  externalEditor?: string
}

export interface WindowConfig {
  width: number
  height: number
  x?: number
  y?: number
  maximized: boolean
}

export interface AppearanceConfig {
  theme: string
  uiFontFamily: string
  uiFontSize: number
  codeFontFamily: string
  codeFontSize: number
  iconSize: number
}

export interface SessionConfig {
  openTabs: TabInfo[]
  activeTabIndex: number
  recentRepos: RepoEntry[]
  maxRecentRepos: number
}

export interface TabInfo {
  id: string
  repoPath: string
  activeView: 'log' | 'localChanges' | 'fileBrowser' | 'shelve'
}

export interface RepoEntry {
  path: string
  lastOpened: string
}

export interface SvnConfig {
  executable?: string
}

export interface AiConfig {
  provider: string
  endpoint: string
  apiKey: string
  model: string
  timeoutSecs: number
}

export interface DiffConfig {
  contextLines: number
  ignoreWhitespace: boolean
  viewMode: 'unified' | 'side_by_side'
}

export interface LogConfig {
  fetchLimit: number
  showChangedPaths: boolean
}

export interface CommitConfig {
  template?: string
}

export interface FileBrowserConfig {
  showHidden: boolean
}

export interface BehaviorConfig {
  confirmBeforeCommit: boolean
  confirmBeforeRevert: boolean
  autoRefreshSecs?: number
}

export interface AdvancedConfig {
  svnTimeoutSecs: number
  logLevel: 'error' | 'warn' | 'info' | 'debug'
}

export interface CleanupConfig {
  vacuumPristines: boolean
  vacuumPrunables: boolean
  includeExternals: boolean
  removeUnversionedTrees: boolean
  removeIgnoredTrees: boolean
  dropDavCache: boolean
}
