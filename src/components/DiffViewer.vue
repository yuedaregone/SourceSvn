<template>
  <div v-if="visible" class="diff-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
    <div class="diff-modal">
      <div class="diff-header">
        <div class="diff-header-left">
          <FileIcon :size="16" class="file-icon" />
          <span class="diff-filename">{{ t('diffViewer.file') }}: {{ filePath }}</span>
        </div>
        <div class="diff-mode-toggle">
          <button :class="{ active: mode === 'unified' }" @click="mode = 'unified'" class="mode-btn">
            <AlignLeft :size="14" />
            <span>{{ t('diffViewer.unifiedView') }}</span>
          </button>
          <button :class="{ active: mode === 'side_by_side' }" @click="mode = 'side_by_side'" class="mode-btn">
            <Columns :size="14" />
            <span>{{ t('diffViewer.sideBySideView') }}</span>
          </button>
        </div>
        <button class="btn btn-icon btn-ghost" @click="$emit('close')">
          <X :size="16" />
        </button>
      </div>
      <div class="diff-content">
        <template v-if="mode === 'unified'">
          <table class="diff-table">
            <tbody>
              <tr v-for="(line, i) in parsedLines" :key="i" :class="lineClass(line)">
                <td class="line-no old">{{ line.oldNo ?? '' }}</td>
                <td class="line-no new">{{ line.newNo ?? '' }}</td>
                <td class="line-prefix">{{ line.prefix }}</td>
                <td class="line-text"><pre>{{ line.text }}</pre></td>
              </tr>
            </tbody>
          </table>
        </template>
        <template v-else>
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
                  <pre class="line-text">{{ line.prefix === '-' ? line.text : '' }}</pre>
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
                  <pre class="line-text">{{ line.prefix === '+' ? line.text : '' }}</pre>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
      <div class="diff-footer">
        <button @click="copyDiff" class="btn btn-secondary">
          <Copy :size="14" />
          <span>{{ t('diffViewer.copyDiff') }}</span>
        </button>
        <button @click="$emit('aiReview', diffText)" class="btn btn-secondary ai-btn">
          <Sparkles :size="14" />
          <span>{{ t('diffViewer.aiReview') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useToastStore } from '../stores/toastStore'
import { t } from '../locales'
import { X, File as FileIcon, AlignLeft, Columns, Copy, Sparkles } from 'lucide-vue-next'

const overlayMousedown = ref(false)
function onOverlayClick() {
  if (!overlayMousedown.value) return
  overlayMousedown.value = false
  emit('close')
}

const props = defineProps<{
  visible: boolean
  filePath: string
  diffText: string
}>()

const emit = defineEmits<{
  close: []
  aiReview: [diff: string]
}>()

const mode = ref<'unified' | 'side_by_side'>('unified')

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
      right.push({ prefix: ' ', text: '', oldNo: null, newNo: null })
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

async function copyDiff() {
  try {
    await navigator.clipboard.writeText(props.diffText)
    useToastStore().success(t('diffViewer.copySuccess'))
  } catch (e) {
    useToastStore().error(t('diffViewer.copyFailed'))
  }
}
</script>

<style scoped>
.diff-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-overlay);
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.diff-modal {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  width: 85%;
  height: 85%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-xl);
  animation: scaleIn 0.2s ease;
}

@keyframes scaleIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.diff-header {
  display: flex;
  align-items: center;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border);
  gap: var(--space-3);
  flex-shrink: 0;
}

.diff-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex: 1;
  overflow: hidden;
}

.file-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.diff-filename {
  font-size: var(--text-base);
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-primary);
}

.diff-mode-toggle {
  display: flex;
  gap: var(--space-1);
  background: var(--color-bg-secondary);
  padding: var(--space-1);
  border-radius: var(--radius-md);
}

.mode-btn {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--text-sm);
  transition: all var(--transition-fast);
}

.mode-btn:hover {
  color: var(--color-text-primary);
}

.mode-btn.active {
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  box-shadow: var(--shadow-sm);
}

.diff-content {
  flex: 1;
  overflow: auto;
  font-family: var(--font-code);
  font-size: var(--text-base);
  background: var(--color-bg-primary);
}

.diff-table {
  width: 100%;
  border-collapse: collapse;
}

.diff-table td {
  padding: 0 var(--space-2);
  white-space: pre;
  vertical-align: top;
  line-height: 1.6;
}

.line-no {
  width: 50px;
  text-align: right;
  color: var(--color-text-muted);
  user-select: none;
  padding-right: var(--space-2);
  border-right: 1px solid var(--color-border);
  font-size: var(--text-sm);
}

.line-prefix {
  width: 20px;
  text-align: center;
  user-select: none;
  color: var(--color-text-muted);
}

.line-text {
  white-space: pre;
  margin: 0;
}

.line-add {
  background: var(--color-diff-add-bg);
}

.line-del {
  background: var(--color-diff-del-bg);
}

.line-hunk {
  background: var(--color-diff-hunk-bg);
  color: var(--color-text-secondary);
}

.line-add .line-text {
  color: var(--color-diff-add-text);
}

.line-del .line-text {
  color: var(--color-diff-del-text);
}

.side-by-side {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.side-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-border);
  overflow: hidden;
}

.side-col:last-child {
  border-right: none;
}

.side-header {
  padding: var(--space-2) var(--space-3);
  background: var(--color-bg-secondary);
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  border-bottom: 1px solid var(--color-border);
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
  padding: 0 var(--space-2);
}

.side-line .line-text {
  flex: 1;
  padding: 0 var(--space-2);
}

.diff-footer {
  display: flex;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--color-border);
}

.ai-btn {
  color: var(--color-purple);
  border-color: var(--color-purple-muted);
}

.ai-btn:hover {
  background: var(--color-purple-muted);
  border-color: var(--color-purple);
}
</style>
