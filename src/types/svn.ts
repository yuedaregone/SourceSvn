export type FileStatusType = 'modified' | 'added' | 'deleted' | 'unversioned' | 'missing' | 'conflicted'

export interface FileStatus {
  path: string
  status: FileStatusType
  isDirectory: boolean
  copied?: boolean
}

export interface LogEntry {
  revision: number
  author: string
  date: string
  message: string
  changedPaths?: ChangedPath[]
}

export interface WcLogResult {
  entries: LogEntry[]
  wcRevision: number
}

export interface ChangedPath {
  path: string
  action: 'A' | 'M' | 'D' | 'R'
  copyFromPath?: string
  copyFromRev?: number
}

export interface RepoInfo {
  url: string
  root: string
  revision: number
  lastChangedRev: number
  lastChangedDate: string
  lastChangedAuthor: string
}

export interface DirEntry {
  name: string
  kind: 'file' | 'dir'
  size?: number
  revision: number
  author: string
  date: string
}

export interface ShelveInfo {
  name: string
  date: string
}

export type DiffTarget =
  | { type: 'file'; data: { path: string; revision?: string } }
  | { type: 'fileAtRevision'; data: { path: string; baseRevision: string; revision: string } }
  | { type: 'revisions'; data: { oldRev: string; newRev: string } }

export interface CommitResult {
  revision: number
  success: boolean
  errors?: string[]
}

export interface UpdateFileItem {
  path: string
  status: 'A' | 'U' | 'M' | 'C'
  author: string
}

export interface UpdateResult {
  revision: number
  files: UpdateFileItem[]
}

export type SvnUpdateEvent =
  | { type: 'file'; status: string; path: string }
  | { type: 'done'; revision: number }
  | { type: 'error'; message: string }
  | { type: 'upToDate'; revision: number }

export type ActiveView = 'log' | 'localChanges' | 'fileBrowser' | 'shelve'

export interface BlameEntry {
  revision: number
  author: string
  lineNumber: number
}
