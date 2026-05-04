<template>
  <div v-if="diffText" class="inline-diff">
    <div class="side-by-side">
      <div class="side-col">
        <div class="side-header">{{ t('diffViewer.original') }}</div>
        <div class="side-lines">
          <div
            v-for="(line, i) in sideBySide.left"
            :key="'l' + i"
            :class="['side-line', lineClass(line)]"
          >
            <span class="line-no">{{ line.oldNo ?? '' }}</span>
            <pre class="line-text">{{ line.prefix === '-' || line.prefix === ' ' ? line.text : '' }}</pre>
          </div>
        </div>
      </div>
      <div class="side-col">
        <div class="side-header">{{ t('diffViewer.modified') }}</div>
        <div class="side-lines">
          <div
            v-for="(line, i) in sideBySide.right"
            :key="'r' + i"
            :class="['side-line', lineClass(line)]"
          >
            <span class="line-no">{{ line.newNo ?? '' }}</span>
            <pre class="line-text">{{ line.prefix === '+' || line.prefix === ' ' ? line.text : '' }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div v-else class="inline-diff-empty">{{ emptyHint || t('common.clickToViewDiff') }}</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { t } from '../locales'

const props = defineProps<{
  diffText: string
  emptyHint?: string
}>()

interface DiffLine {
  prefix: string
  text: string
  oldNo: number | null
  newNo: number | null
}

const parsedLines = computed(() => {
  const lines = props.diffText.split('\n')
  const result: DiffLine[] = []
  let oldNo = 0
  let newNo = 0

  for (const line of lines) {
    const hunkMatch = line.match(/^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/)
    if (hunkMatch) {
      oldNo = parseInt(hunkMatch[1], 10)
      newNo = parseInt(hunkMatch[2], 10)
      result.push({ prefix: '@', text: line.slice(line.indexOf('@@', 2) + 2), oldNo: null, newNo: null })
      continue
    }
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

const sideBySide = computed(() => {
  const left: DiffLine[] = []
  const right: DiffLine[] = []
  for (const line of parsedLines.value) {
    if (line.prefix === '+') {
      right.push(line)
      left.push({ prefix: ' ', text: '', oldNo: null, newNo: null })
    } else if (line.prefix === '-') {
      left.push(line)
      right.push({ prefix: ' ', text: '', newNo: null, oldNo: null })
    } else if (line.prefix === '@') {
      left.push(line)
      right.push({ ...line })
    } else {
      left.push(line)
      right.push({ ...line })
    }
  }
  return { left, right }
})

function lineClass(line: DiffLine) {
  if (line.prefix === '+') return 'line-add'
  if (line.prefix === '-') return 'line-del'
  if (line.prefix === '@') return 'line-hunk'
  return ''
}
</script>

<style scoped>
.inline-diff {
  flex: 1;
  overflow: hidden;
  display: flex;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  background: var(--bg-secondary);
}
.inline-diff-empty {
  color: var(--text-muted);
  text-align: center;
  margin-top: 40px;
  font-size: 13px;
}
.side-by-side {
  display: flex;
  width: 100%;
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
</style>
