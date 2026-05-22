<template>
  <!-- 面板模式 -->
  <Teleport v-if="mode === 'panel'" to="body">
    <Transition name="slide">
      <div v-if="panelVisible" class="diff-panel-overlay" @mousedown.self="onOverlayMousedown" @click.self="onOverlayClick">
        <div class="diff-panel" :style="{ width: panelWidth + 'px' }">
          <div class="resize-handle" @mousedown="onResizeStart" @touchstart="onResizeStart"></div>
          <div class="panel-header">
            <div class="panel-header-left">
              <FileIcon :size="16" class="panel-icon" />
              <span class="panel-filename" :title="panelFilePath">{{ panelFilename }}</span>
            </div>
            <div class="panel-actions">
              <!-- 双排/单排切换 -->
              <button class="btn btn-icon btn-ghost" :class="{ active: outputFormat === 'side-by-side' }" @click="outputFormat = 'side-by-side'" :title="t('diffViewer.sideBySide')">
                <Columns2 :size="14" />
              </button>
              <button class="btn btn-icon btn-ghost" :class="{ active: outputFormat === 'line-by-line' }" @click="outputFormat = 'line-by-line'" :title="t('diffViewer.lineByLine')">
                <AlignLeft :size="14" />
              </button>
              <!-- 显示模式切换 -->
              <button class="btn btn-icon btn-ghost" :class="{ active: showFullContent }" @click="showFullContent = !showFullContent" :title="showFullContent ? t('diffViewer.showNearby') : t('diffViewer.showFull')">
                <FileText :size="14" />
              </button>
              <!-- 自动换行 -->
              <button class="btn btn-icon btn-ghost" :class="{ active: wordWrap }" @click="wordWrap = !wordWrap" :title="t('diffViewer.wordWrap')">
                <WrapText :size="14" />
              </button>
              <button class="btn btn-icon btn-ghost" @click="copyDiff" :title="t('diffViewer.copyDiff')">
                <Copy :size="14" />
              </button>
              <button class="btn btn-icon btn-ghost" @click="closePanel">
                <X :size="16" />
              </button>
            </div>
          </div>
          <div class="panel-content">
            <div class="code-diff-viewer">
              <div v-if="panelFilename && panelOldString !== undefined && panelNewString !== undefined" class="diff-header">
                <span class="diff-filename">{{ panelFilename }}</span>
                <span class="diff-nav">
                  <button class="nav-btn" :disabled="hunkCount === 0" @click="prevHunk" :title="t('diffViewer.prevChange')">
                    <ChevronUp :size="14" />
                  </button>
                  <button class="nav-btn" :disabled="hunkCount === 0" @click="nextHunk" :title="t('diffViewer.nextChange')">
                    <ChevronDown :size="14" />
                  </button>
                  <span v-if="hunkCount > 0" class="nav-counter">{{ currentHunkIndex + 1 }}/{{ hunkCount }}</span>
                  <span v-else class="nav-counter">0/0</span>
                </span>
                <span class="diff-stat">
                  <span class="diff-stat-added">+{{ diffStat.addNum }}</span>
                  <span class="diff-stat-deleted">-{{ diffStat.delNum }}</span>
                </span>
              </div>
              <div v-if="panelOldString !== undefined && panelNewString !== undefined" ref="scrollAreaRef" class="diff-scroll-area" :class="{ 'word-wrap': wordWrap }">
                <CodeDiff
                  :key="showFullContent ? 'full' : 'nearby'"
                  :old-string="panelOldString"
                  :new-string="panelNewString"
                  :language="language"
                  :output-format="outputFormat"
                  :trim="false"
                  :no-diff-line-feed="true"
                  hide-header
                  theme="dark"
                  diff-style="word"
                  :context="showFullContent ? 1000000 : 10"
                  @diff="onDiff"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <!-- 内嵌模式 -->
  <div v-else class="code-diff-viewer">
    <div v-if="filename && oldString !== undefined && newString !== undefined" class="diff-header">
      <span class="diff-filename">{{ filename }}</span>
      <span class="diff-nav">
        <button class="nav-btn" :disabled="hunkCount === 0" @click="prevHunk" :title="t('diffViewer.prevChange')">
          <ChevronUp :size="14" />
        </button>
        <button class="nav-btn" :disabled="hunkCount === 0" @click="nextHunk" :title="t('diffViewer.nextChange')">
          <ChevronDown :size="14" />
        </button>
        <span v-if="hunkCount > 0" class="nav-counter">{{ currentHunkIndex + 1 }}/{{ hunkCount }}</span>
        <span v-else class="nav-counter">0/0</span>
      </span>
      <span class="diff-stat">
        <span class="diff-stat-added">+{{ diffStat.addNum }}</span>
        <span class="diff-stat-deleted">-{{ diffStat.delNum }}</span>
      </span>
    </div>
    <div v-if="oldString !== undefined && newString !== undefined" ref="scrollAreaRef" class="diff-scroll-area">
      <CodeDiff
        :old-string="oldString"
        :new-string="newString"
        :language="language"
        output-format="side-by-side"
        :trim="false"
        :no-diff-line-feed="true"
        hide-header
        theme="dark"
        diff-style="word"
        :context="10"
        @diff="onDiff"
      />
    </div>
    <div v-else class="code-diff-empty">{{ emptyHint || t('common.clickToViewDiff') }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, nextTick } from 'vue'
import { CodeDiff } from 'v-code-diff'
import vCodeDiff from 'v-code-diff'
const hljs = vCodeDiff.hljs
import { ChevronUp, ChevronDown, X, File as FileIcon, Copy, Columns2, AlignLeft, FileText, WrapText } from 'lucide-vue-next'
import { t } from '../locales'
import { useDiffStore } from '../stores/diffStore'
import { useToastStore } from '../stores/toastStore'

// 注册额外的语言包
import csharp from 'highlight.js/lib/languages/csharp'
import cpp from 'highlight.js/lib/languages/cpp'
import css from 'highlight.js/lib/languages/css'
import scss from 'highlight.js/lib/languages/scss'
import go from 'highlight.js/lib/languages/go'
import rust from 'highlight.js/lib/languages/rust'
import kotlin from 'highlight.js/lib/languages/kotlin'
import typescript from 'highlight.js/lib/languages/typescript'
import php from 'highlight.js/lib/languages/php'
import ruby from 'highlight.js/lib/languages/ruby'
import markdown from 'highlight.js/lib/languages/markdown'
import ini from 'highlight.js/lib/languages/ini'
import powershell from 'highlight.js/lib/languages/powershell'
import lua from 'highlight.js/lib/languages/lua'
import perl from 'highlight.js/lib/languages/perl'
import swift from 'highlight.js/lib/languages/swift'
import dockerfile from 'highlight.js/lib/languages/dockerfile'
import makefile from 'highlight.js/lib/languages/makefile'

hljs.registerLanguage('csharp', csharp)
hljs.registerLanguage('cpp', cpp)
hljs.registerLanguage('css', css)
hljs.registerLanguage('scss', scss)
hljs.registerLanguage('go', go)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('kotlin', kotlin)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('php', php)
hljs.registerLanguage('ruby', ruby)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('ini', ini)
hljs.registerLanguage('powershell', powershell)
hljs.registerLanguage('lua', lua)
hljs.registerLanguage('perl', perl)
hljs.registerLanguage('swift', swift)
hljs.registerLanguage('dockerfile', dockerfile)
hljs.registerLanguage('makefile', makefile)

const props = withDefaults(defineProps<{
  mode?: 'inline' | 'panel'
  oldString?: string
  newString?: string
  filename?: string
  emptyHint?: string
}>(), {
  mode: 'inline'
})

const diffStore = useDiffStore()
const toastStore = useToastStore()

// 面板模式：显示选项
const outputFormat = ref<'side-by-side' | 'line-by-line'>('side-by-side')
const showFullContent = ref(false)
const wordWrap = ref(false)

// 面板模式相关
const panelVisible = computed(() => props.mode === 'panel' && diffStore.visible)
const panelFilePath = computed(() => diffStore.filePath)
const panelFilename = computed(() => {
  return diffStore.filePath.split(/[/\\]/).pop() || diffStore.filePath
})

const panelOldString = computed(() => {
  // 优先使用完整内容
  if (diffStore.oldContent !== undefined) return diffStore.oldContent
  return extractContent(diffStore.diffText, 'old')
})

const panelNewString = computed(() => {
  // 优先使用完整内容
  if (diffStore.newContent !== undefined) return diffStore.newContent
  return extractContent(diffStore.diffText, 'new')
})

function extractContent(diffText: string, side: 'old' | 'new'): string {
  if (!diffText) return ''
  const lines = diffText.split('\n')
  const result: string[] = []

  for (const line of lines) {
    // 跳过diff头部信息
    if (line.startsWith('Index:')) continue
    if (line.startsWith('===')) continue
    if (line.startsWith('---')) continue
    if (line.startsWith('+++')) continue
    if (line.startsWith('@@')) continue

    if (side === 'old') {
      if (!line.startsWith('+')) {
        result.push(line.startsWith('-') ? line.slice(1) : line)
      }
    } else {
      if (!line.startsWith('-')) {
        result.push(line.startsWith('+') ? line.slice(1) : line)
      }
    }
  }

  return result.join('\n')
}

const extLangMap: Record<string, string> = {
  js: 'javascript', jsx: 'javascript', mjs: 'javascript', cjs: 'javascript',
  ts: 'typescript', tsx: 'typescript',
  json: 'json', xml: 'xml', html: 'xml', htm: 'xml', svg: 'xml',
  vue: 'html', css: 'css', scss: 'scss', less: 'css', sass: 'scss',
  yaml: 'yaml', yml: 'yaml', toml: 'ini',
  py: 'python', pyw: 'python',
  java: 'java', kt: 'kotlin',
  c: 'cpp', h: 'cpp', cpp: 'cpp', hpp: 'cpp', cc: 'cpp', cs: 'csharp',
  go: 'go', rs: 'rust', swift: 'swift',
  php: 'php', rb: 'ruby',
  sql: 'sql',
  sh: 'bash', bash: 'bash', zsh: 'bash', ps1: 'powershell',
  md: 'markdown', markdown: 'markdown',
  lua: 'lua', pl: 'perl', perl: 'perl',
  dockerfile: 'dockerfile', makefile: 'makefile',
}

const language = computed(() => {
  const fname = props.mode === 'panel' ? panelFilePath.value : props.filename
  if (!fname) return 'plaintext'
  const ext = fname.split('.').pop()?.toLowerCase() || ''
  return extLangMap[ext] || 'plaintext'
})

const diffStat = ref({ addNum: 0, delNum: 0 })
const scrollAreaRef = ref<HTMLElement | null>(null)
const currentHunkIndex = ref(-1)
const hunkCount = ref(0)

function onDiff(result: { stat: { addNum: number; delNum: number; isChanged: boolean } }) {
  diffStat.value = result.stat
  currentHunkIndex.value = -1
  nextTick(() => {
    hunkCount.value = getHunks().length
  })
}

/** 获取所有差异块：连续的增/删行组成一个 hunk，返回每个 hunk 的首行元素 */
function getHunks(): HTMLElement[] {
  if (!scrollAreaRef.value) return []
  const rows = scrollAreaRef.value.querySelectorAll('tr')
  const hunks: HTMLElement[] = []
  let inHunk = false
  for (const row of rows) {
    const isChange = row.querySelector('.blob-code-addition, .blob-code-deletion')
    if (isChange) {
      if (!inHunk) {
        hunks.push(row as HTMLElement)
        inHunk = true
      }
    } else {
      inHunk = false
    }
  }
  return hunks
}

function scrollToHunk(hunkEl: HTMLElement) {
  // 清除旧高亮
  scrollAreaRef.value?.querySelectorAll('.current-hunk-start').forEach(el => el.classList.remove('current-hunk-start'))
  hunkEl.classList.add('current-hunk-start')
  hunkEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

function prevHunk() {
  const hunks = getHunks()
  if (hunks.length === 0) return
  if (currentHunkIndex.value <= 0) {
    currentHunkIndex.value = hunks.length - 1
  } else {
    currentHunkIndex.value--
  }
  scrollToHunk(hunks[currentHunkIndex.value])
}

function nextHunk() {
  const hunks = getHunks()
  if (hunks.length === 0) return
  if (currentHunkIndex.value >= hunks.length - 1) {
    currentHunkIndex.value = 0
  } else {
    currentHunkIndex.value++
  }
  scrollToHunk(hunks[currentHunkIndex.value])
}

// 面板模式：关闭
function closePanel() {
  diffStore.close()
}

// 面板模式：复制diff
async function copyDiff() {
  try {
    const textToCopy = diffStore.diffText || `${diffStore.oldContent}\n---\n${diffStore.newContent}`
    await navigator.clipboard.writeText(textToCopy)
    toastStore.success(t('diffViewer.copySuccess'))
  } catch {
    toastStore.error(t('diffViewer.copyFailed'))
  }
}

// 面板模式：点击overlay关闭
const overlayMousedown = ref(false)

function onOverlayMousedown() {
  overlayMousedown.value = true
}

function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  closePanel()
}

// 面板模式：调整宽度
const panelWidth = ref(800)
const MIN_WIDTH = 400
const MAX_WIDTH_RATIO = 0.85

let resizing = false
let startX = 0
let startWidth = 0

function onResizeStart(e: MouseEvent | TouchEvent) {
  e.preventDefault()
  resizing = true
  startX = 'touches' in e ? e.touches[0].clientX : e.clientX
  startWidth = panelWidth.value
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', onResizeEnd)
  document.addEventListener('touchmove', onResizeMove)
  document.addEventListener('touchend', onResizeEnd)
}

function onResizeMove(e: MouseEvent | TouchEvent) {
  if (!resizing) return
  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX
  const delta = startX - clientX
  const maxWidth = window.innerWidth * MAX_WIDTH_RATIO
  const newWidth = Math.min(maxWidth, Math.max(MIN_WIDTH, startWidth + delta))
  panelWidth.value = newWidth
}

function onResizeEnd() {
  resizing = false
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
  document.removeEventListener('touchmove', onResizeMove)
  document.removeEventListener('touchend', onResizeEnd)
}
</script>

<style scoped>
.code-diff-viewer {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.diff-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: var(--color-bg-tertiary);
  border-bottom: 2px solid var(--color-accent);
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
}
.diff-filename {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.diff-nav {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: 8px;
}
.nav-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  color: var(--color-text-secondary);
}
.nav-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}
.nav-btn:disabled {
  opacity: 0.3;
  cursor: default;
}
.nav-counter {
  font-size: 11px;
  color: var(--color-text-muted);
  min-width: 28px;
  text-align: center;
}
.diff-stat {
  display: flex;
  gap: 8px;
  font-weight: 500;
  margin-left: auto;
}
.diff-stat-added {
  color: var(--color-diff-add-text, var(--color-success));
}
.diff-stat-deleted {
  color: var(--color-diff-del-text, var(--color-danger));
}
.diff-scroll-area :deep(.current-hunk-start td) {
  box-shadow: inset 0 2px 0 var(--color-accent);
}
.diff-scroll-area {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
/* 自动换行模式 */
.diff-scroll-area.word-wrap :deep(.code-diff-view .diff-table .blob-code),
.diff-scroll-area.word-wrap :deep(.code-diff-view .diff-table .blob-code-inner) {
  white-space: pre-wrap !important;
  word-break: break-all;
  overflow: visible;
  text-overflow: unset;
}
.code-diff-empty {
  color: var(--color-text-muted);
  text-align: center;
  margin-top: 40px;
  font-size: 13px;
}
/* 覆盖 v-code-diff 默认样式以适配项目主题 */
.diff-scroll-area :deep(.code-diff-view) {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  background: var(--color-bg-secondary);
  border: none;
  border-radius: 0;
  max-height: none;
  margin: 0;
  padding: 0;
}
/* 默认不换行，超出部分裁掉 */
.diff-scroll-area :deep(.code-diff-view .diff-table .blob-code),
.diff-scroll-area :deep(.code-diff-view .diff-table .blob-code-inner) {
  white-space: pre !important;
  overflow: hidden;
  text-overflow: ellipsis;
  word-wrap: normal;
}
/* 双排模式下每个单元格占50%宽度 */
.diff-scroll-area :deep(.code-diff-view .split-view td) {
  width: 50%;
  max-width: 0;
}
.diff-scroll-area :deep(.blob-code-addition) {
  background: var(--diff-add-bg);
}
.diff-scroll-area :deep(.blob-code-deletion) {
  background: var(--diff-del-bg);
}
.diff-scroll-area :deep(.blob-code-context) {
  background: var(--color-bg-secondary);
}
.diff-scroll-area :deep(.blob-num) {
  color: var(--color-text-muted);
  border-right: 1px solid var(--color-border);
}
.diff-scroll-area :deep(.split-view) {
  border: none;
}
.diff-scroll-area :deep(table) {
  border-collapse: collapse;
}
.diff-scroll-area :deep(td) {
  padding: 0;
}

/* 面板模式样式 */
.diff-panel-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  z-index: var(--z-overlay);
  display: flex;
  justify-content: flex-end;
}

.diff-panel {
  height: 100%;
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--color-border);
  position: relative;
}

.resize-handle {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  z-index: 10;
}

.resize-handle:hover,
.resize-handle:active {
  background: var(--color-accent);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.panel-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  overflow: hidden;
  flex: 1;
}

.panel-icon {
  color: var(--color-accent);
  flex-shrink: 0;
}

.panel-filename {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.panel-actions {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  flex-shrink: 0;
}

.panel-actions .btn-ghost.active {
  color: var(--color-accent);
  background: var(--color-bg-hover);
}

.panel-content {
  flex: 1;
  overflow: hidden;
  display: flex;
}

.slide-enter-active,
.slide-leave-active {
  transition: opacity 0.25s ease;
}

.slide-enter-active .diff-panel,
.slide-leave-active .diff-panel {
  transition: transform 0.25s ease;
}

.slide-enter-from .diff-panel,
.slide-leave-to .diff-panel {
  transform: translateX(100%);
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
}

.slide-enter-to,
.slide-leave-from {
  opacity: 1;
}
</style>
