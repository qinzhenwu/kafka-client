<!-- src/components/common/IconButton.vue -->
<script setup lang="ts">
import { computed, ref } from 'vue'
import { NTooltip } from 'naive-ui'
import { icons, type IconName } from '@/config/icons'

interface Props {
  icon?: IconName
  active?: boolean
  danger?: boolean
  filled?: boolean
  tooltip?: string
  size?: 'small' | 'medium' | 'large'
  loading?: boolean
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  active: false,
  danger: false,
  filled: false,
  tooltip: '',
  size: 'medium',
  loading: false,
  disabled: false
})

const emit = defineEmits<{
  click: []
}>()

const isHovered = ref(false)

const sizeMap = {
  small: { padding: '4px', iconSize: 16 },
  medium: { padding: '10px', iconSize: 22 },
  large: { padding: '12px', iconSize: 24 }
}

const iconSize = computed(() => sizeMap[props.size].iconSize)

const IconComponent = computed(() => props.icon ? icons[props.icon] : null)

const buttonStyle = computed(() => {
  let background = 'transparent'
  if (props.active) {
    background = 'var(--accent-bg)'
  } else if (isHovered.value && !props.disabled && !props.loading) {
    background = 'var(--bg-hover)'
  }

  let color = props.danger ? 'var(--error)' : props.active ? 'var(--accent)' : 'var(--text-muted)'
  if (props.disabled || props.loading) {
    color = 'var(--text-muted)'
  }

  return {
    padding: sizeMap[props.size].padding,
    color,
    background,
    borderRadius: 'var(--border-radius)',
    cursor: props.disabled || props.loading ? 'not-allowed' : 'pointer',
    transition: 'all 0.2s ease',
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    border: 'none',
    userSelect: 'none' as const,
    opacity: props.disabled ? 0.5 : 1,
    position: 'relative' as const
  }
})

const iconStrokeWidth = computed(() => props.active ? 2.5 : 1.5)

const handleMouseEnter = () => {
  if (!props.disabled && !props.loading) {
    isHovered.value = true
  }
}

const handleMouseLeave = () => {
  isHovered.value = false
}

const handleClick = () => {
  if (!props.disabled && !props.loading) {
    emit('click')
  }
}
</script>

<template>
  <n-tooltip v-if="tooltip" :delay="500">
    <template #trigger>
      <div
        :style="buttonStyle"
        class="icon-button"
        :class="{ 'is-loading': loading, 'is-filled': danger && filled }"
        @click="handleClick"
        @mouseenter="handleMouseEnter"
        @mouseleave="handleMouseLeave"
      >
        <span v-if="loading" class="loading-spinner"></span>
        <component v-else-if="IconComponent" :is="IconComponent" :size="iconSize" :stroke-width="iconStrokeWidth" />
      </div>
    </template>
    {{ tooltip }}
  </n-tooltip>
  <div
    v-else
    :style="buttonStyle"
    class="icon-button"
    :class="{ 'is-loading': loading, 'is-active': active, 'is-filled': danger && filled }"
    @click="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <span v-if="loading" class="loading-spinner"></span>
    <component v-else-if="IconComponent" :is="IconComponent" :size="iconSize" :stroke-width="iconStrokeWidth" />
  </div>
</template>

<style scoped>
.icon-button:hover:not(.is-loading) {
  color: var(--text-secondary);
}

.icon-button.is-filled :deep(svg) {
  fill: var(--error);
}

.icon-button.is-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 3px;
  background: var(--accent);
  border-radius: 0 2px 2px 0;
}

.loading-spinner {
  width: 1em;
  height: 1em;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>