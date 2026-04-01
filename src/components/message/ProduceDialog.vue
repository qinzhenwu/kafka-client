<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NInput, NInputNumber, NSelect, useMessage
} from 'naive-ui'
import { useMessageStore, type ProduceRequest } from '@/stores/message'
import { useClusterStore } from '@/stores/cluster'
import { useI18n } from 'vue-i18n'
import IconButton from '@/components/common/IconButton.vue'
import { Send, X, Trash2 } from 'lucide-vue-next'

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
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click="handleClose">
      <div class="modal-container produce-dialog" @click.stop>
        <!-- Header -->
        <div class="modal-header">
          <span class="header-title">
            <Send :size="18" :stroke-width="1.5" class="header-icon" />
            {{ t('message.produce') }} - {{ topicName }}
          </span>
          <X :size="16" class="header-close" @click="handleClose" />
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
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  width: 600px;
  max-width: 95vw;
  max-height: 90vh;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.header-icon {
  color: var(--text-primary);
}

.header-close {
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
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
  border-top: 1px solid var(--border);
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
  background: var(--bg-tertiary);
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
  border-top: 1px solid var(--border);
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
</style>