<template>
  <div class="code-diff-viewer">
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
import { CodeDiff, hljs } from 'v-code-diff'
import { ChevronUp, ChevronDown } from 'lucide-vue-next'
import { t } from '../locales'

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

const props = defineProps<{
  oldString?: string
  newString?: string
  filename?: string
  emptyHint?: string
}>()

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
  if (!props.filename) return 'plaintext'
  const ext = props.filename.split('.').pop()?.toLowerCase() || ''
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
</style>
