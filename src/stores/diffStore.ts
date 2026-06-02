import { defineStore } from 'pinia'
import { ref } from 'vue'

const BINARY_PREFIX = '\x00BINARY:'
const BINARY_SUFFIX = '\x00'

/** 判断字符串是否是二进制占位符，返回哈希值或 null */
function parseBinaryPlaceholder(s: string): string | null {
  if (s.startsWith(BINARY_PREFIX) && s.endsWith(BINARY_SUFFIX)) {
    return s.slice(BINARY_PREFIX.length, s.length - BINARY_SUFFIX.length)
  }
  return null
}

export const useDiffStore = defineStore('diff', () => {
  const visible = ref(false)
  const filePath = ref('')
  const diffText = ref('')
  // 完整文件内容
  const oldContent = ref<string | undefined>(undefined)
  const newContent = ref<string | undefined>(undefined)
  // 二进制文件标志
  const isBinary = ref(false)
  const binaryIdentical = ref<boolean | null>(null) // null 表示只有单侧

  function open(path: string, text: string) {
    filePath.value = path
    diffText.value = text
    oldContent.value = undefined
    newContent.value = undefined
    isBinary.value = false
    binaryIdentical.value = null
    visible.value = true
  }

  // 传入完整内容，自动检测二进制
  function openWithContent(path: string, oldStr: string | undefined, newStr: string | undefined) {
    filePath.value = path
    diffText.value = ''

    const oldHash = oldStr !== undefined ? parseBinaryPlaceholder(oldStr) : null
    const newHash = newStr !== undefined ? parseBinaryPlaceholder(newStr) : null
    const oldIsBinary = oldStr !== undefined && oldHash !== null
    const newIsBinary = newStr !== undefined && newHash !== null

    if (oldIsBinary || newIsBinary) {
      isBinary.value = true
      if (oldIsBinary && newIsBinary) {
        // 两侧都有二进制内容，通过 FNV 哈希判断是否相同
        binaryIdentical.value = oldHash === newHash
      } else {
        // 一侧是二进制一侧不是（或一侧不存在），视为不同
        binaryIdentical.value = false
      }
      oldContent.value = undefined
      newContent.value = undefined
    } else {
      isBinary.value = false
      binaryIdentical.value = null
      oldContent.value = oldStr
      newContent.value = newStr
    }

    visible.value = true
  }

  function close() {
    visible.value = false
  }

  return {
    visible, filePath, diffText,
    oldContent, newContent,
    isBinary, binaryIdentical,
    open, openWithContent, close,
  }
})
