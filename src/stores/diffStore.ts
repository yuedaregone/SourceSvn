import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useDiffStore = defineStore('diff', () => {
  const visible = ref(false)
  const filePath = ref('')
  const diffText = ref('')
  // 新增：完整文件内容
  const oldContent = ref<string | undefined>(undefined)
  const newContent = ref<string | undefined>(undefined)

  function open(path: string, text: string) {
    filePath.value = path
    diffText.value = text
    oldContent.value = undefined
    newContent.value = undefined
    visible.value = true
  }

  // 新增：传入完整内容
  function openWithContent(path: string, oldStr: string | undefined, newStr: string | undefined) {
    filePath.value = path
    diffText.value = ''
    oldContent.value = oldStr
    newContent.value = newStr
    visible.value = true
  }

  function close() {
    visible.value = false
  }

  return { visible, filePath, diffText, oldContent, newContent, open, openWithContent, close }
})
