<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="context-menu-overlay"
      @click="$emit('close')"
      @contextmenu.prevent="$emit('close')"
    >
      <div
        ref="menuRef"
        class="context-menu"
        :style="menuStyle"
        @click.stop
      >
        <template v-for="(item, idx) in items" :key="idx">
          <div v-if="item.divider" class="context-menu-divider" />
          <div
            v-else
            class="context-menu-item"
            :class="{ disabled: item.disabled }"
            @click="handleClick(item)"
          >
            <component
              :is="item.icon"
              v-if="item.icon"
              class="context-menu-icon"
              :size="14"
            />
            <span class="context-menu-label">{{ item.label }}</span>
            <span v-if="item.shortcut" class="context-menu-shortcut">{{ item.shortcut }}</span>
          </div>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, type Component } from 'vue'

export interface MenuItem {
  label?: string
  icon?: Component
  action?: () => void
  disabled?: boolean
  divider?: boolean
  shortcut?: string
}

const props = defineProps<{
  items: MenuItem[]
  visible: boolean
  x: number
  y: number
}>()

const emit = defineEmits<{
  close: []
}>()

const menuRef = ref<HTMLDivElement | null>(null)

const menuStyle = computed(() => ({
  left: `${props.x}px`,
  top: `${props.y}px`,
}))

watch(() => props.visible, async (val) => {
  if (val) {
    await nextTick()
    adjustPosition()
  }
})

function adjustPosition() {
  if (!menuRef.value) return
  const menu = menuRef.value
  const rect = menu.getBoundingClientRect()
  const vw = window.innerWidth
  const vh = window.innerHeight

  let left = props.x
  let top = props.y

  if (left + rect.width > vw) {
    left = vw - rect.width - 4
  }
  if (top + rect.height > vh) {
    top = vh - rect.height - 4
  }
  if (left < 0) left = 4
  if (top < 0) top = 4

  menu.style.left = `${left}px`
  menu.style.top = `${top}px`
}

function handleClick(item: MenuItem) {
  if (item.disabled) return
  item.action?.()
  emit('close')
}
</script>

<style>
.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 9999;
}

.context-menu {
  position: fixed;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-1) 0;
  min-width: 180px;
  box-shadow: var(--shadow-lg);
  z-index: 10000;
  user-select: none;
  animation: scaleIn 0.15s ease;
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

.context-menu-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-base);
  color: var(--color-text-primary);
  cursor: pointer;
  white-space: nowrap;
  transition: background var(--transition-fast);
}

.context-menu-item:hover:not(.disabled) {
  background: var(--color-bg-hover);
}

.context-menu-item.disabled {
  color: var(--color-text-muted);
  cursor: default;
}

.context-menu-icon {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.context-menu-item.disabled .context-menu-icon {
  color: var(--color-text-muted);
}

.context-menu-label {
  flex: 1;
}

.context-menu-shortcut {
  color: var(--color-text-muted);
  font-size: var(--text-sm);
  margin-left: var(--space-4);
  font-family: var(--font-mono);
}

.context-menu-divider {
  height: 1px;
  background: var(--color-border);
  margin: var(--space-1) var(--space-2);
}
</style>
