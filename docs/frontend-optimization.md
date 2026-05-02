# 前端代码优化文档

> 审查日期：2026-05-01
> 审查范围：src/ 目录下所有前端代码

---

## 一、高优先级问题

### 1.1 类型安全问题

**位置**：`src/App.vue:61`

**问题描述**：
使用非空断言操作符 `!` 可能导致运行时错误。

```vue
<LogView
  v-if="currentTabStore?.activeView === 'log'"
  :store="currentTabStore!"
  ...
/>
```

**风险**：当 `currentTabStore` 为 `null` 时会崩溃。

**修复建议**：
```vue
<LogView
  v-if="currentTabStore && currentTabStore.activeView === 'log'"
  :store="currentTabStore"
  ...
/>
```

---

### 1.2 内存泄漏风险

**位置**：`src/App.vue:100-107`

**问题描述**：
`visibilitychange` 事件监听器在组件卸载时未移除。

```typescript
onMounted(async () => {
  // ...
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
      refreshCurrentView()
    }
  })
  // ...
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  stopAutoRefresh()
  // 缺少 visibilitychange 的移除
})
```

**修复建议**：
```typescript
const handleVisibilityChange = () => {
  if (document.visibilityState === 'visible') {
    refreshCurrentView()
  }
}

onMounted(async () => {
  // ...
  document.addEventListener('visibilitychange', handleVisibilityChange)
  // ...
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  stopAutoRefresh()
})
```

---

### 1.3 Store 设计问题

**位置**：`src/App.vue:76-85` 和 `src/stores/tabStore.ts`

**问题描述**：
动态创建 store 实例存储在普通对象中，关闭 tab 时可能未完全清理。

```typescript
const tabStores = ref<Record<string, TabStoreInstance>>({})

const currentTabStore = computed(() => {
  // ...
  if (!tabStores.value[key]) {
    const store = useTabStore(key)()
    // ...
  }
  return tabStores.value[key]
})
```

**风险**：
1. 内存泄漏风险
2. 状态管理复杂

**修复建议**：
1. 方案一：使用单一 store 管理所有 tab 数据
2. 方案二：确保 `closeTab` 函数正确清理 store

```typescript
function closeTab(index: number) {
  const key = `${index}`
  const store = tabStores.value[key]
  if (store) {
    // 清理 store 状态
    store.$dispose()
    delete tabStores.value[key]
  }
  // ... 其他逻辑
}
```

---

## 二、中优先级问题

### 2.1 错误处理不完善

**位置**：多处文件

**问题描述**：
错误只输出到控制台，用户无法感知操作失败。

```typescript
} catch (e) {
  console.error('Failed to refresh log:', e)
}
```

**修复建议**：
1. 创建全局 Toast 组件
2. 在 catch 块中显示错误提示

```typescript
// 创建 src/components/Toast.vue
// 创建 src/stores/toastStore.ts
// 使用方式：
} catch (e) {
  console.error('Failed to refresh log:', e)
  toastStore.showError('刷新日志失败: ' + e)
}
```

---

### 2.2 图标风格不一致

**位置**：`src/components/GlobalTabBar.vue:4`

**问题描述**：
设置按钮使用 emoji，与其他组件的 Lucide 图标风格不一致。

```vue
<button class="settings-btn" @click="$emit('openSettings')" title="设置">⚙</button>
```

**修复建议**：
```vue
<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
</script>

<template>
  <button class="settings-btn" @click="$emit('openSettings')" title="设置">
    <Settings :size="16" />
  </button>
</template>
```

---

### 2.3 AddRepoDialog 缺少浏览按钮

**位置**：`src/components/AddRepoDialog.vue`

**问题描述**：
路径输入框没有浏览按钮，用户需要手动输入路径。

```vue
<input v-model="localPath" placeholder="C:\path\to\working\copy" @keyup.enter="openLocal" />
```

**修复建议**：
```vue
<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog'

async function browseLocalPath() {
  const selected = await open({
    title: '选择工作副本目录',
    directory: true,
  })
  if (selected) {
    localPath.value = selected.toString()
  }
}
</script>

<template>
  <div class="input-with-btn">
    <input v-model="localPath" placeholder="C:\path\to\working\copy" @keyup.enter="openLocal" />
    <button @click="browseLocalPath" class="browse-btn">浏览...</button>
  </div>
</template>
```

---

### 2.4 硬编码字符串

**位置**：多处文件

**问题描述**：
中文字符串硬编码在代码中，难以支持多语言。

```vue
placeholder="提交信息..."
title="AI 生成注释"
<span>全选</span>
```

**修复建议**：
1. 创建 `src/locales/zh-CN.ts`
2. 使用 Vue I18n 或简单的翻译函数

```typescript
// src/locales/zh-CN.ts
export default {
  common: {
    submit: '提交',
    cancel: '取消',
    refresh: '刷新',
    selectAll: '全选',
  },
  localChanges: {
    commitMessage: '提交信息...',
    aiGenerate: 'AI 生成注释',
  },
  // ...
}
```

---

## 三、低优先级问题

### 3.1 重复代码

**位置**：`src/App.vue:240-247`

**问题描述**：
`refreshCurrentView` 函数使用 if-else 链，代码重复。

```typescript
function refreshCurrentView() {
  if (!currentTabStore.value) return
  const view = currentTabStore.value.activeView
  if (view === 'log') currentTabStore.value.refreshLog()
  else if (view === 'localChanges') currentTabStore.value.refreshLocalChanges()
  else if (view === 'fileBrowser') currentTabStore.value.refreshFileBrowser()
  else if (view === 'shelve') currentTabStore.value.refreshShelves()
}
```

**修复建议**：
```typescript
function refreshCurrentView() {
  if (!currentTabStore.value) return
  const view = currentTabStore.value.activeView
  const refreshMap: Record<ActiveView, () => void> = {
    log: () => currentTabStore.value!.refreshLog(),
    localChanges: () => currentTabStore.value!.refreshLocalChanges(),
    fileBrowser: () => currentTabStore.value!.refreshFileBrowser(),
    shelve: () => currentTabStore.value!.refreshShelves(),
  }
  refreshMap[view]?.()
}
```

---

### 3.2 CSS 变量重复定义

**位置**：`src/App.vue` 和 `src/style.css`

**问题描述**：
主题变量在两处定义，可能导致样式冲突。

**修复建议**：
1. 统一在 `src/style.css` 中定义所有主题变量
2. 移除 `App.vue` 中的重复定义

---

### 3.3 缺少 Loading 状态反馈

**位置**：`src/views/LocalChangesView.vue` 等视图组件

**问题描述**：
刷新操作时没有明显的加载指示。

**修复建议**：
1. 在数据加载时显示 loading 动画
2. 禁用相关按钮防止重复操作

```vue
<template>
  <div class="file-list" :class="{ loading: store.loading }">
    <div v-if="store.loading" class="loading-overlay">
      <RefreshCw :size="20" class="spin" />
    </div>
    <!-- ... -->
  </div>
</template>

<style scoped>
.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  opacity: 0.8;
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
```

---

### 3.4 直接修改 Props

**位置**：`src/views/LocalChangesView.vue:189`

**问题描述**：
直接修改 props 传递的对象属性，违反单向数据流原则。

```typescript
props.store.logEntries = []
```

**修复建议**：
```typescript
// 通过 store action 清空
props.store.clearLogEntries()

// 或通过 emit 通知父组件
emit('clearLog')
```

---

### 3.5 DiffViewer 并排视图对齐问题

**位置**：`src/components/DiffViewer.vue`

**问题描述**：
并排视图的行号和内容可能不对齐，当左右行数不同时显示会错位。

**修复建议**：
优化 `sideBySide` 计算逻辑，确保左右两侧行数一致：

```typescript
const sideBySide = computed(() => {
  const left: DiffLine[] = []
  const right: DiffLine[] = []
  
  for (const line of parsedLines.value) {
    if (line.prefix === '+') {
      // 新增行：左侧空，右侧有内容
      left.push({ prefix: ' ', text: '', oldNo: null, newNo: null })
      right.push(line)
    } else if (line.prefix === '-') {
      // 删除行：左侧有内容，右侧空
      left.push(line)
      right.push({ prefix: ' ', text: '', oldNo: null, newNo: null })
    } else {
      // 普通行：两侧都有
      left.push(line)
      right.push({ ...line })
    }
  }
  return { left, right }
})
```

---

## 四、优化计划

### 第一阶段（高优先级）

| 序号 | 任务 | 预计工作量 |
|------|------|-----------|
| 1 | 修复类型安全问题 | 0.5h |
| 2 | 修复内存泄漏问题 | 1h |
| 3 | 优化 Store 设计 | 2h |

### 第二阶段（中优先级）

| 序号 | 任务 | 预计工作量 |
|------|------|-----------|
| 4 | 添加全局 Toast 组件 | 2h |
| 5 | 统一图标风格 | 0.5h |
| 6 | AddRepoDialog 添加浏览按钮 | 1h |
| 7 | 国际化支持 | 4h |

### 第三阶段（低优先级）

| 序号 | 任务 | 预计工作量 |
|------|------|-----------|
| 8 | 重构重复代码 | 1h |
| 9 | 统一 CSS 变量管理 | 1h |
| 10 | 添加 Loading 状态 | 2h |
| 11 | 修复 Props 修改问题 | 0.5h |
| 12 | 优化 DiffViewer 并排视图 | 2h |

---

## 五、相关文件清单

| 文件路径 | 问题数量 | 优先级 |
|----------|----------|--------|
| `src/App.vue` | 4 | 高 |
| `src/stores/tabStore.ts` | 1 | 高 |
| `src/components/GlobalTabBar.vue` | 1 | 中 |
| `src/components/AddRepoDialog.vue` | 1 | 中 |
| `src/components/DiffViewer.vue` | 1 | 低 |
| `src/views/LocalChangesView.vue` | 2 | 低 |
| `src/style.css` | 1 | 低 |

---

## 六、备注

- 本文档基于代码审查生成，后续优化时请根据实际情况调整
- 建议按优先级顺序进行优化
- 每完成一项优化，请在对应任务上标记完成状态
