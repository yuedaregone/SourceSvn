<template>
  <div class="hook-list">
    <div class="hook-list-header">
      <input
        v-model="searchQuery"
        placeholder="搜索hook..."
        class="input"
      />
      <button @click="$emit('add')" class="btn btn-primary">
        <Plus :size="14" />
        <span>添加Hook</span>
      </button>
    </div>
    <div class="hook-items">
      <div
        v-for="handler in filteredHandlers"
        :key="handler.name"
        :class="['hook-item', { active: selectedName === handler.name }]"
        @click="$emit('select', handler.name)"
      >
        <div class="hook-item-info">
          <span class="hook-name">{{ handler.name }}</span>
        </div>
        <div class="hook-item-actions">
          <button
            @click.stop="$emit('toggle', handler.name)"
            :class="['btn btn-sm', handler.enabled ? 'btn-primary' : 'btn-ghost']"
          >
            {{ handler.enabled ? '启用' : '禁用' }}
          </button>
          <button
            @click.stop="$emit('delete', handler.name)"
            class="btn btn-sm btn-danger"
          >
            <Trash2 :size="14" />
          </button>
        </div>
      </div>
      <div v-if="filteredHandlers.length === 0" class="hook-empty">
        <span v-if="searchQuery">未找到匹配的hook</span>
        <span v-else>暂无配置的hook</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { Plus, Trash2 } from 'lucide-vue-next'
import type { HookHandlerConfig } from '../../stores/hook'

const props = defineProps<{
  handlers: HookHandlerConfig[]
  selectedName: string | null
}>()

defineEmits<{
  add: []
  select: [name: string]
  toggle: [name: string]
  delete: [name: string]
}>()

const searchQuery = ref('')

const filteredHandlers = computed(() => {
  if (!searchQuery.value) return props.handlers
  const query = searchQuery.value.toLowerCase()
  return props.handlers.filter(
    h => h.name.toLowerCase().includes(query)
  )
})
</script>

<style scoped>
.hook-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 220px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border);
}

.hook-list-header {
  display: flex;
  gap: var(--space-2);
  padding: var(--space-3);
  border-bottom: 1px solid var(--color-border);
}

.hook-items {
  flex: 1;
  overflow-y: auto;
}

.hook-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3);
  border-bottom: 1px solid var(--color-border-light);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.hook-item:hover {
  background: var(--color-bg-hover);
}

.hook-item.active {
  background: var(--color-bg-active);
}

.hook-item-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.hook-name {
  font-weight: 500;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hook-type {
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
}

.hook-item-actions {
  display: flex;
  gap: var(--space-1);
  flex-shrink: 0;
}

.hook-empty {
  padding: var(--space-6);
  text-align: center;
  color: var(--color-text-muted);
}
</style>
