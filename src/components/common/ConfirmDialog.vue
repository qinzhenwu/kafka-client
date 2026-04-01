<script setup lang="ts">
import { X } from 'lucide-vue-next'

defineProps<{
  show: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
}>()

const emit = defineEmits<{
  close: []
  confirm: []
  cancel: []
}>()

const handleConfirm = () => {
  emit('confirm')
  emit('close')
}

const handleCancel = () => {
  emit('cancel')
  emit('close')
}

const handleOverlayClick = () => {
  handleCancel()
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="confirm-overlay" @click="handleOverlayClick">
      <div class="confirm-dialog" @click.stop>
        <div class="dialog-header">
          <span class="dialog-title">{{ title }}</span>
          <X :size="18" class="dialog-close" @click="handleCancel" />
        </div>
        <div class="dialog-content">
          <p>{{ message }}</p>
        </div>
        <div class="dialog-footer">
          <button class="btn btn-cancel" @click="handleCancel">
            {{ cancelText || '取消' }}
          </button>
          <button class="btn btn-confirm" @click="handleConfirm">
            {{ confirmText || '确认' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
}

.confirm-dialog {
  width: 360px;
  background: var(--glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--glass-border);
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
  padding: 20px;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.5;
}

.dialog-content p {
  margin: 0;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--glass-border);
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
}

.btn-cancel {
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--glass-border);
  color: var(--text-muted);
}

.btn-cancel:hover {
  background: var(--glass-bg-hover);
  color: var(--text-primary);
}

.btn-confirm {
  background: var(--accent);
  color: white;
}

.btn-confirm:hover {
  background: var(--accent-hover);
}

/* Light Mode */
:root[data-theme="light"] .confirm-overlay {
  background: rgba(0, 0, 0, 0.2);
}

:root[data-theme="light"] .confirm-dialog {
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid rgba(59, 130, 246, 0.2);
  box-shadow: 0 8px 32px rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .dialog-header {
  border-bottom: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .dialog-title {
  color: #1e293b;
}

:root[data-theme="light"] .dialog-close {
  color: #64748b;
}

:root[data-theme="light"] .dialog-close:hover {
  color: #1e293b;
}

:root[data-theme="light"] .dialog-content {
  color: #475569;
}

:root[data-theme="light"] .dialog-footer {
  border-top: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .btn-cancel {
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(59, 130, 246, 0.15);
  color: #475569;
}

:root[data-theme="light"] .btn-cancel:hover {
  background: rgba(59, 130, 246, 0.1);
  color: #1e293b;
}

:root[data-theme="light"] .btn-confirm {
  background: #3b82f6;
  color: white;
}

:root[data-theme="light"] .btn-confirm:hover {
  background: #2563eb;
}
</style>