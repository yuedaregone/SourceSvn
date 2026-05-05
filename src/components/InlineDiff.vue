<template>
  <div v-if="diffText" class="inline-diff">
    <div class="diff-toolbar">
      TOOLBAR
      <button class="toolbar-btn" :disabled="hunkCount === 0 || currentHunkIndex <= 0" @click="prevHunk" :title="t('diffViewer.prevChange')">
        <ChevronUp :size="14" />
      </button>
      <button class="toolbar-btn" :disabled="hunkCount === 0 || currentHunkIndex >= hunkCount - 1" @click="nextHunk" :title="t('diffViewer.nextChange')">
        <ChevronDown :size="14" />
      </button>
      <span v-if="hunkCount > 0" class="hunk-counter">{{ currentHunkIndex + 1 }}/{{ hunkCount }}</span>
      <span v-else class="hunk-counter">0/0</span>
      <span class="toolbar-sep"></span>
      <button class="toolbar-btn" :class="{ active: collapsed }" @click="collapsed = !collapsed" :title="t('diffViewer.collapseUnchanged')">
        <AlignJustify :size="14" />
      </button>
      <template v-if="collapsed">
        <span class="toolbar-sep"></span>
        <span class="context-label">{{ t('diffViewer.context') }}</span>
        <button v-for="n in [0, 1, 2, 3, 5]" :key="n" class="context-btn" :class="{ active: contextLevel === n }" @click="contextLevel = n">{{ n }}</button>
      </template>
      <span class="toolbar-sep"></span>
      <button class="toolbar-btn" @click="copyDiff" :title="t('diffViewer.copyDiff')">
        <Copy :size="14" />
      </button>
      <span class="toolbar-stats">
        <span class="stat-add">+{{ stats.added }}</span>
        <span class="stat-del">-{{ stats.removed }}</span>
      </span>
    </div>
    <div class="side-by-side">
      <div class="side-col">
        <div class="side-header">{{ t('diffViewer.original') }}</div>
        <div class="side-lines" ref="leftScroll">
          <template v-for="(line, i) in displayLines.left" :key="'l' + i">
            <div v-if="line.prefix === '...'" class="side-line line-hidden">
              <span class="line-no"></span>
              <pre class="line-text">{{ line.text }}</pre>
            </div>
            <div v-else :class="['side-line', lineClass(line)]" :ref="el => setLineRef('left', i, line, el)">
              <span class="line-no">{{ line.oldNo ?? '' }}</span>
              <pre class="line-text">{{ line.prefix === '-' || line.prefix === ' ' ? line.text : '' }}</pre>
            </div>
          </template>
        </div>
      </div>
      <div class="side-col">
        <div class="side-header">{{ t('diffViewer.modified') }}</div>
        <div class="side-lines" ref="rightScroll">
          <template v-for="(line, i) in displayLines.right" :key="'r' + i">
            <div v-if="line.prefix === '...'" class="side-line line-hidden">
              <span class="line-no"></span>
              <pre class="line-text">{{ line.text }}</pre>
            </div>
            <div v-else :class="['side-line', lineClass(line)]" :ref="el => setLineRef('right', i, line, el)">
              <span class="line-no">{{ line.newNo ?? '' }}</span>
              <pre class="line-text">{{ line.prefix === '+' || line.prefix === ' ' ? line.text : '' }}</pre>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
  <div v-else class="inline-diff-empty">{{ emptyHint || t('common.clickToViewDiff') }}</div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { ChevronUp, ChevronDown, AlignJustify, Copy } from 'lucide-vue-next'
import { useToastStore } from '../stores/toastStore'
import { t } from '../locales'

const props = defineProps<{
  diffText: string
  emptyHint?: string
  showToolbar?: boolean
}>()

interface DiffLine {
  prefix: string
  text: string
  oldNo: number | null
  newNo: number | null
}

const collapsed = ref(false)
const contextLevel = ref(3)
const currentHunkIndex = ref(0)
const leftScroll = ref<HTMLElement | null>(null)
const rightScroll = ref<HTMLElement | null>(null)

// Store refs to hunk line elements for scrolling
const hunkElements: Record<string, HTMLElement> = {}

function setLineRef(side: string, index: number, line: DiffLine, el: unknown) {
  if (line.prefix === '@') {
    hunkElements[`${side}-${index}`] = el as HTMLElement
  }
}

const parsedLines = computed(() => {
  const lines = props.diffText.split('\n')
  const start = lines.findIndex((l) => l.startsWith('@@'))
  if (start < 0) return []

  const result: DiffLine[] = []
  let oldNo = 0
  let newNo = 0

  for (let i = start; i < lines.length; i++) {
    const line = lines[i]
    const hunkMatch = line.match(/^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/)
    if (hunkMatch) {
      oldNo = parseInt(hunkMatch[1], 10)
      newNo = parseInt(hunkMatch[2], 10)
      result.push({ prefix: '@', text: line.slice(line.indexOf('@@', 2) + 2), oldNo: null, newNo: null })
      continue
    }
    if (line === '\\ No newline at end of file') continue
    if (line.startsWith('+')) {
      result.push({ prefix: '+', text: line.slice(1), oldNo: null, newNo: newNo++ })
    } else if (line.startsWith('-')) {
      result.push({ prefix: '-', text: line.slice(1), oldNo: oldNo++, newNo: null })
    } else {
      result.push({ prefix: ' ', text: line.startsWith(' ') ? line.slice(1) : line, oldNo: oldNo++, newNo: newNo++ })
    }
  }
  return result
})

const stats = computed(() => {
  let added = 0
  let removed = 0
  for (const line of parsedLines.value) {
    if (line.prefix === '+') added++
    else if (line.prefix === '-') removed++
  }
  return { added, removed }
})

const hunkParsedIndices = computed(() => {
  const indices: number[] = []
  for (let i = 0; i < parsedLines.value.length; i++) {
    if (parsedLines.value[i].prefix === '@') indices.push(i)
  }
  return indices
})

const hunkCount = computed(() => hunkParsedIndices.value.length)

function buildSideBySide(lines: DiffLine[]) {
  const left: DiffLine[] = []
  const right: DiffLine[] = []
  for (const line of lines) {
    if (line.prefix === '+') {
      right.push(line)
      left.push({ prefix: ' ', text: '', oldNo: null, newNo: null })
    } else if (line.prefix === '-') {
      left.push(line)
      right.push({ prefix: ' ', text: '', newNo: null, oldNo: null })
    } else if (line.prefix === '@') {
      left.push(line)
      right.push({ ...line })
    } else if (line.prefix === '...') {
      left.push(line)
      right.push({ ...line })
    } else {
      left.push(line)
      right.push({ ...line })
    }
  }
  return { left, right }
}

const fullSideBySide = computed(() => buildSideBySide(parsedLines.value))

const collapsedSideBySide = computed(() => {
  const lines = parsedLines.value
  if (lines.length === 0) return { left: [] as DiffLine[], right: [] as DiffLine[] }

  const visible = new Set<number>()
  const hunkIndices = hunkParsedIndices.value
  const ctx = contextLevel.value

  for (const hi of hunkIndices) {
    visible.add(hi)
    // find the extent of this hunk (consecutive +/- lines after the @@ line)
    let hunkEnd = hi
    for (let j = hi + 1; j < lines.length; j++) {
      if (lines[j].prefix === '+' || lines[j].prefix === '-' || lines[j].prefix === ' ') {
        hunkEnd = j
      } else {
        break
      }
    }
    // add context before
    for (let j = Math.max(0, hi - ctx); j < hi; j++) visible.add(j)
    // add hunk body + context after
    for (let j = hi; j <= Math.min(lines.length - 1, hunkEnd + ctx); j++) visible.add(j)
  }

  const filtered: DiffLine[] = []
  let hiddenStart = -1

  for (let i = 0; i < lines.length; i++) {
    if (visible.has(i)) {
      if (hiddenStart >= 0) {
        const count = i - hiddenStart
        filtered.push({ prefix: '...', text: `... ${count} unchanged line${count > 1 ? 's' : ''} ...`, oldNo: null, newNo: null })
        hiddenStart = -1
      }
      filtered.push(lines[i])
    } else {
      if (hiddenStart < 0) hiddenStart = i
    }
  }
  if (hiddenStart >= 0) {
    const count = lines.length - hiddenStart
    filtered.push({ prefix: '...', text: `... ${count} unchanged line${count > 1 ? 's' : ''} ...`, oldNo: null, newNo: null })
  }

  return buildSideBySide(filtered)
})

const displayLines = computed(() => {
  return collapsed.value ? collapsedSideBySide.value : fullSideBySide.value
})

function lineClass(line: DiffLine) {
  if (line.prefix === '+') return 'line-add'
  if (line.prefix === '-') return 'line-del'
  if (line.prefix === '@') return 'line-hunk'
  return ''
}

function scrollToHunk(index: number) {
  if (index < 0 || index >= hunkParsedIndices.value.length) return
  currentHunkIndex.value = index
  const parsedIdx = hunkParsedIndices.value[index]

  nextTick(() => {
    // In non-collapsed mode, find the hunk element by matching parsed index
    // We need to find which display-line index corresponds to this parsed index
    const lines = displayLines.value.left
    let parsedCount = 0
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].prefix === '...') continue
      if (parsedCount === parsedIdx) {
        const el = leftScroll.value?.querySelectorAll('.side-line')[i] as HTMLElement | undefined
        if (el) {
          el.scrollIntoView({ behavior: 'smooth', block: 'center' })
          // flash highlight
          el.classList.add('flash')
          setTimeout(() => el.classList.remove('flash'), 800)
        }
        return
      }
      parsedCount++
    }
  })
}

function prevHunk() {
  if (currentHunkIndex.value > 0) {
    scrollToHunk(currentHunkIndex.value - 1)
  }
}

function nextHunk() {
  if (currentHunkIndex.value < hunkCount.value - 1) {
    scrollToHunk(currentHunkIndex.value + 1)
  }
}

async function copyDiff() {
  try {
    await navigator.clipboard.writeText(props.diffText)
    useToastStore().success(t('diffViewer.copySuccess'))
  } catch {
    useToastStore().error(t('diffViewer.copyFailed'))
  }
}

// Reset hunk index when diff changes
watch(() => props.diffText, () => {
  currentHunkIndex.value = 0
})
</script>

<style scoped>
.inline-diff {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  background: var(--bg-secondary);
  overflow: hidden;
}
.inline-diff-empty {
  color: var(--text-muted);
  text-align: center;
  margin-top: 40px;
  font-size: 13px;
}
.diff-toolbar {
  display: flex !important;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  border-bottom: 2px solid var(--accent-color);
  background: var(--bg-tertiary, #f5f5f5);
  flex-shrink: 0;
  min-height: 32px;
  z-index: 1;
}
.toolbar-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  color: var(--text-secondary);
}
.toolbar-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.toolbar-btn:disabled {
  opacity: 0.3;
  cursor: default;
}
.toolbar-btn.active {
  background: var(--bg-active);
  color: var(--accent-color);
}
.toolbar-sep {
  width: 1px;
  height: 16px;
  background: var(--border-light);
  margin: 0 4px;
}
.hunk-counter {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 28px;
  text-align: center;
  font-family: inherit;
}
.context-label {
  font-size: 11px;
  color: var(--text-muted);
  margin-right: 2px;
}
.context-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  border: 1px solid var(--border-light);
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  font-size: 11px;
  color: var(--text-secondary);
  padding: 0 4px;
}
.context-btn:hover {
  border-color: var(--accent-color);
}
.context-btn.active {
  background: var(--accent-color);
  color: #fff;
  border-color: var(--accent-color);
}
.toolbar-stats {
  margin-left: auto;
  display: flex;
  gap: 8px;
  font-size: 12px;
  font-family: inherit;
}
.stat-add {
  color: var(--diff-add-text, var(--success-color));
  font-weight: 500;
}
.stat-del {
  color: var(--diff-del-text, var(--danger-color));
  font-weight: 500;
}
.side-by-side {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.side-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  overflow: hidden;
  min-width: 0;
}
.side-col:last-child {
  border-right: none;
}
.side-header {
  padding: 6px 12px;
  background: var(--bg-tertiary);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}
.side-lines {
  flex: 1;
  overflow: auto;
}
.side-line {
  display: flex;
  min-height: 22px;
  line-height: 1.6;
}
.side-line .line-no {
  min-width: 40px;
  padding: 0 6px;
  text-align: right;
  color: var(--text-muted);
  user-select: none;
  border-right: 1px solid var(--border-color);
  flex-shrink: 0;
}
.side-line .line-text {
  flex: 1;
  padding: 0 8px;
  margin: 0;
  white-space: pre;
  overflow: hidden;
}
.line-add {
  background: var(--diff-add-bg);
}
.line-del {
  background: var(--diff-del-bg);
}
.line-hunk {
  background: var(--diff-hunk-bg);
  color: var(--text-secondary);
}
.line-add .line-text {
  color: var(--diff-add-text);
}
.line-del .line-text {
  color: var(--diff-del-text);
}
.line-hidden {
  background: var(--bg-tertiary);
}
.line-hidden .line-text {
  color: var(--text-muted);
  font-size: 11px;
  text-align: center;
  font-style: italic;
}
@keyframes flash-highlight {
  0% { background: var(--accent-color); color: #fff; }
  100% { background: transparent; }
}
.side-line.flash {
  animation: flash-highlight 0.8s ease-out;
}
</style>
