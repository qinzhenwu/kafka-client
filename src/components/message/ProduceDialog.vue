<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import {
  NInput, NInputNumber, NSelect, useMessage
} from 'naive-ui'
import { useMessageStore, type ProduceRequest } from '@/stores/message'
import { useClusterStore } from '@/stores/cluster'
import { useI18n } from 'vue-i18n'
import IconButton from '@/components/common/IconButton.vue'
import { Send, X, Trash2, GripHorizontal } from 'lucide-vue-next'

const props = defineProps<{
  show: boolean
  topicName: string
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  success: []
}>()

const { t } = useI18n()
const messageStore = useMessageStore()
const clusterStore = useClusterStore()
const message = useMessage()

const formValue = ref<ProduceRequest>({
  topic: props.topicName,
  partition: undefined,
  key: '',
  value: '',
  headers: {},
})

const headerKey = ref('')
const headerValue = ref('')
const messageFormat = ref<'json' | 'text'>('text')

// Drag state
const dialogPosition = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })
const dialogRef = ref<HTMLElement | null>(null)

// Reset form when dialog opens
watch(() => props.show, (show) => {
  if (show) {
    formValue.value = {
      topic: props.topicName,
      partition: undefined,
      key: '',
      value: '',
      headers: {},
    }
    headerKey.value = ''
    headerValue.value = ''
    messageFormat.value = 'text'
    // Reset position
    dialogPosition.value = { x: 0, y: 0 }
  }
})

// Update topic when it changes
watch(() => props.topicName, (topicName) => {
  if (props.show) {
    formValue.value.topic = topicName
  }
})

const addHeader = () => {
  if (headerKey.value && headerValue.value) {
    formValue.value.headers[headerKey.value] = headerValue.value
    headerKey.value = ''
    headerValue.value = ''
  }
}

const removeHeader = (key: string) => {
  delete formValue.value.headers[key]
}

const formatJson = () => {
  try {
    const parsed = JSON.parse(formValue.value.value)
    formValue.value.value = JSON.stringify(parsed, null, 2)
  } catch {
    // Not valid JSON, ignore
  }
}

const handleProduce = async () => {
  if (!clusterStore.activeClusterId) return

  try {
    await messageStore.produceMessage(clusterStore.activeClusterId, {
      ...formValue.value,
      key: formValue.value.key || undefined,
      partition: formValue.value.partition,
    })
    message.success(t('message.produceSuccess', 'Message produced successfully'))
    emit('success')
    // Don't clear form after send - per user request
  } catch (e: unknown) {
    message.error(t('message.produceError', 'Failed to produce message') + ': ' + String(e))
  }
}

const handleClear = () => {
  formValue.value = {
    topic: props.topicName,
    partition: undefined,
    key: '',
    value: '',
    headers: {},
  }
  headerKey.value = ''
  headerValue.value = ''
  messageFormat.value = 'text'
}

const handleClose = () => {
  messageStore.closeProduceDialog()
}

// Drag handlers
const handleMouseDown = (e: MouseEvent) => {
  isDragging.value = true
  dragStart.value = {
    x: e.clientX - dialogPosition.value.x,
    y: e.clientY - dialogPosition.value.y
  }
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return
  dialogPosition.value = {
    x: e.clientX - dragStart.value.x,
    y: e.clientY - dragStart.value.y
  }
}

const handleMouseUp = () => {
  isDragging.value = false
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
}

onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
})
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay">
      <div
        ref="dialogRef"
        class="modal-container produce-dialog"
        :style="{ transform: `translate(${dialogPosition.x}px, ${dialogPosition.y}px)` }"
        @click.stop
      >
        <!-- Header -->
        <div class="modal-header" @mousedown="handleMouseDown">
          <GripHorizontal :size="16" class="drag-handle" />
          <span class="header-title">
            <Send :size="18" :stroke-width="1.5" class="header-icon" />
            {{ t('message.produce') }} - {{ topicName }}
          </span>
          <X :size="18" class="header-close" @click.stop="handleClose" />
        </div>

        <!-- Content -->
        <div class="modal-content">
          <!-- Topic (readonly) -->
          <div class="form-row">
            <label class="form-label">{{ t('message.topic', 'Topic') }}</label>
            <div class="form-value readonly">{{ formValue.topic }}</div>
          </div>

          <!-- Partition -->
          <div class="form-row">
            <label class="form-label">{{ t('message.partition', 'Partition') }}</label>
            <n-input-number
              v-model:value="formValue.partition"
              :min="0"
              :placeholder="t('message.auto', 'Auto')"
              clearable
              size="small"
              style="width: 120px"
            />
          </div>

          <!-- Key -->
          <div class="form-row">
            <label class="form-label">{{ t('message.key', 'Key') }}</label>
            <n-input
              v-model:value="formValue.key"
              :placeholder="t('message.keyPlaceholder', 'Message key (optional)')"
              size="small"
            />
          </div>

          <!-- Format -->
          <div class="form-row">
            <label class="form-label">{{ t('message.format', 'Format') }}</label>
            <div class="format-group">
              <n-select
                v-model:value="messageFormat"
                :options="[
                  { label: 'Plain Text', value: 'text' },
                  { label: 'JSON', value: 'json' },
                ]"
                size="small"
                style="width: 120px"
              />
              <IconButton
                v-if="messageFormat === 'json'"
                icon="clipboard"
                :tooltip="t('message.formatJson', 'Format JSON')"
                size="small"
                @click="formatJson"
              />
            </div>
          </div>

          <!-- Value -->
          <div class="form-row form-row-value">
            <label class="form-label">{{ t('message.value', 'Value') }}</label>
            <n-input
              v-model:value="formValue.value"
              type="textarea"
              :rows="10"
              :placeholder="t('message.valuePlaceholder', 'Message value')"
            />
          </div>

          <!-- Headers -->
          <div class="headers-section">
            <div class="section-title">{{ t('message.headers', 'Headers') }} ({{ t('common.optional', 'Optional') }})</div>

            <!-- Existing headers -->
            <div v-if="Object.keys(formValue.headers).length > 0" class="headers-list">
              <div v-for="(value, key) in formValue.headers" :key="key" class="header-item">
                <span class="header-key">{{ key }}</span>
                <span class="header-value">{{ value }}</span>
                <X :size="14" class="remove-header-icon" @click="removeHeader(key as string)" />
              </div>
            </div>

            <!-- Add header -->
            <div class="add-header-row">
              <n-input
                v-model:value="headerKey"
                :placeholder="t('message.headerKey', 'Key')"
                size="small"
                style="width: 150px"
              />
              <n-input
                v-model:value="headerValue"
                :placeholder="t('message.headerValue', 'Value')"
                size="small"
                style="flex: 1"
              />
              <IconButton
                icon="add"
                :tooltip="t('message.addHeader', 'Add')"
                size="small"
                @click="addHeader"
              />
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="modal-footer">
          <div class="footer-left">
            <span class="footer-hint">{{ t('tab.sendToTopic', { topic: topicName }) }}</span>
          </div>
          <div class="footer-actions">
            <button class="action-btn danger" @click="handleClear">
              <Trash2 :size="14" />
              {{ t('message.clear', 'Clear') }}
            </button>
            <button class="action-btn primary" @click="handleProduce">
              <Send :size="14" />
              {{ t('message.send', 'Send') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  pointer-events: auto;
  width: 600px;
  max-width: 95vw;
  max-height: 90vh;
  background: var(--glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--glass-border);
  flex-shrink: 0;
  cursor: move;
  user-select: none;
}

.drag-handle {
  color: var(--text-muted);
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
  flex: 1;
}

.header-icon {
  color: var(--text-primary);
}

.header-close {
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.header-close:hover {
  color: var(--text-primary);
}

.modal-content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.form-row-value {
  align-items: flex-start;
}

.form-label {
  width: 80px;
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
  padding-top: 6px;
}

.form-value {
  font-size: 13px;
  color: var(--text-primary);
}

.form-value.readonly {
  color: var(--text-muted);
  padding: 6px 0;
}

.format-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.headers-section {
  margin-top: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--glass-border);
}

.section-title {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

.headers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.header-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--glass-border);
  border-radius: 6px;
}

.header-key {
  font-size: 13px;
  color: var(--accent);
  font-weight: 500;
  min-width: 80px;
}

.header-value {
  font-size: 13px;
  color: var(--text-secondary);
  flex: 1;
}

.remove-header-icon {
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.remove-header-icon:hover {
  color: var(--error);
}

.add-header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid var(--glass-border);
  flex-shrink: 0;
}

.footer-left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.footer-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn.primary {
  background: var(--accent);
  color: white;
}

.action-btn.primary:hover {
  background: var(--accent-hover);
}

.action-btn.danger {
  color: var(--error);
}

.action-btn.danger:hover {
  background: var(--error-bg);
}

/* Light Mode */
:root[data-theme="light"] .modal-container {
  background: rgba(255, 255, 255, 0.75);
  border: 1px solid rgba(59, 130, 246, 0.2);
  box-shadow: 0 8px 32px rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .modal-header {
  border-bottom: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .drag-handle {
  color: #94a3b8;
}

:root[data-theme="light"] .header-title {
  color: #1e293b;
}

:root[data-theme="light"] .header-icon {
  color: #3b82f6;
}

:root[data-theme="light"] .header-close {
  color: #64748b;
}

:root[data-theme="light"] .header-close:hover {
  color: #1e293b;
}

:root[data-theme="light"] .form-label {
  color: #64748b;
}

:root[data-theme="light"] .form-value {
  color: #1e293b;
}

:root[data-theme="light"] .form-value.readonly {
  color: #64748b;
}

:root[data-theme="light"] .headers-section {
  border-top: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .section-title {
  color: #64748b;
}

:root[data-theme="light"] .header-item {
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .header-key {
  color: #3b82f6;
}

:root[data-theme="light"] .header-value {
  color: #475569;
}

:root[data-theme="light"] .modal-footer {
  border-top: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .footer-hint {
  color: #64748b;
}

:root[data-theme="light"] .action-btn {
  background: rgba(59, 130, 246, 0.1);
  color: #475569;
}

:root[data-theme="light"] .action-btn:hover {
  background: rgba(59, 130, 246, 0.2);
  color: #1e293b;
}

:root[data-theme="light"] .action-btn.primary {
  background: #3b82f6;
  color: white;
}

:root[data-theme="light"] .action-btn.primary:hover {
  background: #2563eb;
}

:root[data-theme="light"] .action-btn.danger {
  color: #ef4444;
}

:root[data-theme="light"] .action-btn.danger:hover {
  background: rgba(239, 68, 68, 0.1);
}
</style>