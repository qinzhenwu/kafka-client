<!-- src/components/common/AboutDialog.vue -->
<script setup lang="ts">
import { X } from 'lucide-vue-next'
import appIcon from '@/assets/icon.png'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const appVersion = '0.2.11'

const handleClose = () => {
  emit('update:show', false)
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="about-overlay" @click="handleClose">
      <div class="about-dialog" @click.stop>
        <div class="dialog-header">
          <span class="dialog-title">Kafka Client</span>
          <X :size="18" class="dialog-close" @click="handleClose" />
        </div>
        <div class="dialog-content">
          <div class="app-icon">
            <img :src="appIcon" alt="Kafka Client" class="icon-image" />
          </div>
          <div class="app-name">Kafka Client</div>
          <div class="app-version">Version {{ appVersion }}</div>
          <div class="app-desc">
            A cross-platform Kafka desktop client<br>
            built with Tauri + Vue 3
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn btn-primary" @click="handleClose">
            OK
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.about-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
}

.about-dialog {
  width: 320px;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.dialog-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.dialog-close {
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.dialog-close:hover {
  color: var(--text-primary);
}

.dialog-content {
  padding: 32px 20px;
  text-align: center;
}

.app-icon {
  margin-bottom: 16px;
}

.icon-image {
  width: 80px;
  height: 80px;
  border-radius: 16px;
}

.app-name {
  font-size: 18px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.app-version {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 16px;
}

.app-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.dialog-footer {
  display: flex;
  justify-content: center;
  padding: 12px 20px 20px;
}

.btn {
  padding: 8px 32px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
}

.btn-primary {
  background: var(--accent);
  color: white;
}

.btn-primary:hover {
  background: var(--accent-hover);
}
</style>