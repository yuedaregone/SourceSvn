<template>
  <div v-if="visible" class="diff-overlay" @mousedown.self="overlayMousedown = true" @click.self="onOverlayClick">
    <div class="diff-modal">
      <div class="diff-header">
        <span class="diff-filename">{{ t('diffViewer.file') }}: {{ filePath }}</span>
        <div class="diff-mode-toggle">
          <button :class="{ active: mode === 'unified' }" @click="mode = 'unified'">{{ t('diffViewer.unifiedView') }}</button>
          <button :class="{ active: mode === 'side_by_side' }" @click="mode = 'side_by_side'">{{ t('diffViewer.sideBySideView') }}</button>
        </div>
        <button class="close-btn" @click="$emit('close')">&times;</button>
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
        <button @click="copyDiff" class="footer-btn">{{ t('diffViewer.copyDiff') }}</button>
        <button @click="$emit('aiReview', diffText)" class="footer-btn ai">{{ t('diffViewer.aiReview') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useToastStore } from '../stores/toastStore'
import { t } from '../locales'

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
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}
.diff-modal {
  background: var(--bg-primary);
  border-radius: 8px;
  width: 85%;
  height: 85%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow);
}
.diff-header {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  gap: 12px;
}
.diff-filename {
  font-size: 13px;
  font-family: monospace;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-primary);
}
.diff-mode-toggle {
  display: flex;
  gap: 4px;
}
.diff-mode-toggle button {
  padding: 4px 10px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.diff-mode-toggle button.active {
  background: var(--accent-color);
  color: #fff;
  border-color: var(--accent-color);
}
.close-btn {
  border: none;
  background: transparent;
  font-size: 22px;
  cursor: pointer;
  color: var(--text-muted);
}
.close-btn:hover {
  color: var(--text-primary);
}
.diff-content {
  flex: 1;
  overflow: auto;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  background: var(--bg-secondary);
}
.diff-table {
  width: 100%;
  border-collapse: collapse;
}
.diff-table td {
  padding: 0 8px;
  white-space: pre;
  vertical-align: top;
  line-height: 1.6;
}
.line-no {
  width: 50px;
  text-align: right;
  color: var(--text-muted);
  user-select: none;
  padding-right: 8px;
  border-right: 1px solid var(--border-color);
}
.line-prefix {
  width: 20px;
  text-align: center;
  user-select: none;
  color: var(--text-muted);
}
.line-text {
  white-space: pre;
  margin: 0;
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
.side-by-side {
  display: flex;
  height: 100%;
  overflow: hidden;
}
.side-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  overflow: hidden;
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
}
.side-line .line-text {
  flex: 1;
  padding: 0 8px;
}
.diff-footer {
  display: flex;
  gap: 8px;
  padding: 10px 16px;
  border-top: 1px solid var(--border-color);
}
.footer-btn {
  padding: 6px 14px;
  border: 1px solid var(--border-input);
  background: var(--bg-primary);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.footer-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.footer-btn.ai {
  border-color: var(--purple-color);
  color: var(--purple-color);
}
.footer-btn.ai:hover {
  background: var(--bg-active);
}
</style>
